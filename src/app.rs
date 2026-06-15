use std::io;
use std::path::Path;
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::autopilot::{find_nearest_planet, tick_autopilot, AutopilotMode};
use crate::combat::{fire_missile, move_booms, move_missiles};
use crate::config::{read_prefs, write_prefs, Prefs};
use crate::input::InputState;
use crate::mission::{do_events, load_mission};
use crate::physics::{move_planets, move_player, move_targets};
use crate::planet_data::{init_weapons, reset_planets};
use crate::renderer::stars::all_stars;
use crate::renderer::texture::load_all_textures;
use crate::save::{push_save, read_saves, SaveSlot};
use crate::scores::{record_score, top_scores};
use crate::types::World;
use crate::ui::{
    briefing,
    console::{advance_console, console_add},
    title_screen::{render_load_menu, render_title, TITLE_OPTIONS},
};

const FRAME: Duration = Duration::from_millis(33);
const MISSIONS_DIR: &str = "missions";
const MODELS_DIR: &str = "models";
const MAPS_DIR: &str = "maps";
const START_MISSION: &str = "train01.msn";

enum Screen {
    Title(usize),
    Briefing { scores: Vec<crate::scores::ScoreEntry> },
    Playing,
    LoadMenu { slots: Vec<SaveSlot>, sel: usize },
}

pub fn run() -> io::Result<()> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut term = Terminal::new(backend)?;
    term.hide_cursor()?;

    let result = game_loop(&mut term);

    terminal::disable_raw_mode()?;
    execute!(term.backend_mut(), LeaveAlternateScreen)?;
    term.show_cursor()?;

    result
}

fn game_loop(term: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    let mut prefs = read_prefs();
    let mut world = World::new();
    init_weapons(&mut world);
    reset_planets(&mut world);
    apply_prefs(&prefs, &mut world);

    let stars = all_stars();
    let textures = load_all_textures(Path::new(MAPS_DIR));
    let mut dense_stars = prefs.dense_stars;
    let mut show_orrery = false;
    let mut show_names = false;
    let mut show_texture = false;
    let mut show_lines  = false;
    let mut paused = false;
    let mut orbit_cam = false;
    let mut autopilot = AutopilotMode::Off;
    let mut input = InputState::default();
    let mut last_tick = Instant::now();
    let mut screen = Screen::Title(0);

    loop {
        // ── Draw ──────────────────────────────────────────────────────────────
        let saves_cache: Vec<SaveSlot> = match &screen {
            Screen::Title(_) | Screen::LoadMenu { .. } => read_saves(),
            _ => Vec::new(),
        };

        term.draw(|frame| match &screen {
            Screen::Title(sel) => render_title(frame, *sel, &saves_cache),
            Screen::Briefing { scores } => briefing::render(frame, &world, scores),
            Screen::Playing => {
                crate::hud::panels::render(frame, &world, &stars, dense_stars, show_orrery, show_names, paused, show_texture, &textures, orbit_cam, autopilot, show_lines)
            }
            Screen::LoadMenu { slots, sel } => render_load_menu(frame, slots, *sel),
        })?;

        // ── Events ────────────────────────────────────────────────────────────
        let timeout = match screen {
            Screen::Playing => FRAME.saturating_sub(last_tick.elapsed()),
            _ => Duration::from_millis(50),
        };

        if event::poll(timeout)? {
            let ev = event::read()?;

            match &mut screen {
                Screen::Title(sel) => {
                    if let Some(next) =
                        handle_title_event(ev, sel, &saves_cache, &mut world, &prefs)?
                    {
                        if matches!(next, Screen::Playing) {
                            last_tick = Instant::now();
                        }
                        screen = next;
                    }
                }

                Screen::Briefing { .. } => {
                    if is_any_keypress(&ev) {
                        let msn = world.mission_file.clone();
                        push_save(&msn);
                        world.briefing.clear();
                        if msn == "win.msn" || msn == "lose.msn" {
                            screen = Screen::Title(0);
                        } else {
                            last_tick = Instant::now();
                            screen = Screen::Playing;
                        }
                    }
                }

                Screen::Playing => {
                    if handle_playing_event(ev, &mut input, &mut dense_stars, &mut show_orrery, &mut show_names, &mut show_texture, &mut show_lines, &mut orbit_cam, &mut autopilot, &mut paused, &mut world)? {
                        // Q — save prefs and return to title.
                        prefs.dense_stars = dense_stars;
                        write_prefs(&prefs);
                        push_save(&world.mission_file);
                        screen = Screen::Title(0);
                    }
                    while event::poll(Duration::ZERO)? {
                        let ev2 = event::read()?;
                        handle_playing_event(ev2, &mut input, &mut dense_stars, &mut show_orrery, &mut show_names, &mut show_texture, &mut show_lines, &mut orbit_cam, &mut autopilot, &mut paused, &mut world)?;
                    }
                }

                Screen::LoadMenu { slots, sel } => {
                    if let Some(next) =
                        handle_load_event(ev, slots, sel, &mut world, &prefs)?
                    {
                        if matches!(next, Screen::Playing) {
                            last_tick = Instant::now();
                        }
                        screen = next;
                    }
                }
            }
        }

        // ── Game tick (Playing only, not when paused) ─────────────────────────
        if matches!(screen, Screen::Playing) && !paused && last_tick.elapsed() >= FRAME {
            let dt = world.delta_t;

            if input.fire {
                player_fire(&mut world);
            }

            // Autopilot overrides manual input when engaged.
            if autopilot != AutopilotMode::Off {
                input = InputState::default();
                tick_autopilot(&mut world, autopilot, dt);
            }
            input.apply_to_player(&mut world.player);
            move_player(&mut world);
            move_targets(&mut world);
            move_missiles(&mut world);
            move_booms(&mut world);
            move_planets(&mut world);
            world.abs_t += dt;

            if !world.message.text.is_empty() {
                world.message.age += dt;
                if world.message.age > crate::constants::MSG_MAXAGE {
                    world.message.text.clear();
                }
            }

            advance_console(&mut world, dt);

            // Check for mission transitions before loading so we can record the
            // score for the mission just completed.
            let old_mission = world.mission_file.clone();
            let old_score = world.player.score;

            let mission_changed = if let Some(msn) = world.pending_mission.take() {
                let forward = msn != old_mission;
                try_load_mission(&mut world, &msn);
                if forward {
                    record_score(&old_mission, &prefs.player_name, old_score);
                }
                true
            } else if let Some(next_msn) = do_events(&mut world) {
                try_load_mission(&mut world, &next_msn);
                record_score(&old_mission, &prefs.player_name, old_score);
                true
            } else {
                false
            };

            if mission_changed && !world.briefing.is_empty() {
                let scores = top_scores(&world.mission_file);
                screen = Screen::Briefing { scores };
            }

            input.clear();
            last_tick += FRAME;
        }
    }
}

// ─── Event handlers ───────────────────────────────────────────────────────────

fn handle_title_event(
    ev: Event,
    sel: &mut usize,
    saves: &[SaveSlot],
    world: &mut World,
    prefs: &Prefs,
) -> io::Result<Option<Screen>> {
    let Event::Key(KeyEvent { code, modifiers, .. }) = ev else {
        return Ok(None);
    };
    if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
        return Err(io::Error::new(io::ErrorKind::Interrupted, "ctrl-c"));
    }

    match code {
        KeyCode::Char('q') | KeyCode::Esc => {
            return Err(io::Error::new(io::ErrorKind::Other, "quit"));
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if *sel > 0 { *sel -= 1; }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if *sel + 1 < TITLE_OPTIONS.len() { *sel += 1; }
        }
        KeyCode::Enter | KeyCode::Char(' ') => match *sel {
            0 => return Ok(Some(start_new_game(world))),
            1 if !saves.is_empty() => {
                return Ok(Some(Screen::LoadMenu {
                    slots: saves.to_vec(),
                    sel: 0,
                }));
            }
            2 => return Err(io::Error::new(io::ErrorKind::Other, "quit")),
            _ => {}
        },
        KeyCode::Char('n') => return Ok(Some(start_new_game(world))),
        KeyCode::Char('l') if !saves.is_empty() => {
            return Ok(Some(Screen::LoadMenu {
                slots: saves.to_vec(),
                sel: 0,
            }));
        }
        _ => {}
    }
    let _ = prefs;
    Ok(None)
}

fn handle_load_event(
    ev: Event,
    slots: &[SaveSlot],
    sel: &mut usize,
    world: &mut World,
    _prefs: &Prefs,
) -> io::Result<Option<Screen>> {
    let Event::Key(KeyEvent { code, .. }) = ev else {
        return Ok(None);
    };

    match code {
        KeyCode::Char('q') | KeyCode::Esc => return Ok(Some(Screen::Title(1))),
        KeyCode::Up | KeyCode::Char('k') => {
            if *sel > 0 { *sel -= 1; }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if *sel + 1 < slots.len() { *sel += 1; }
        }
        KeyCode::Enter | KeyCode::Char(' ') => {
            if let Some(slot) = slots.get(*sel) {
                return Ok(Some(load_slot(world, &slot.mission.clone())));
            }
        }
        KeyCode::Char(c) if c.is_ascii_digit() => {
            let idx = (c as u8 - b'0') as usize;
            if let Some(slot) = slots.get(idx) {
                return Ok(Some(load_slot(world, &slot.mission.clone())));
            }
        }
        _ => {}
    }
    Ok(None)
}

/// Returns true if the caller should return to the title screen.
#[allow(clippy::too_many_arguments)]
fn handle_playing_event(
    ev: Event,
    input: &mut InputState,
    dense_stars: &mut bool,
    show_orrery: &mut bool,
    show_names: &mut bool,
    show_texture: &mut bool,
    show_lines: &mut bool,
    orbit_cam: &mut bool,
    autopilot: &mut AutopilotMode,
    paused: &mut bool,
    world: &mut World,
) -> io::Result<bool> {
    if let Event::Key(KeyEvent { code, modifiers, .. }) = ev {
        if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
            return Err(io::Error::new(io::ErrorKind::Interrupted, "ctrl-c"));
        }
        // P always toggles pause regardless of current state.
        if code == KeyCode::Char('p') {
            *paused = !*paused;
            return Ok(false);
        }
        // Q/Esc always quits, even while paused.
        if *paused {
            if code == KeyCode::Char('q') || code == KeyCode::Esc {
                return Ok(true);
            }
            return Ok(false);
        }
        match code {
            KeyCode::Char('q') | KeyCode::Esc => return Ok(true),
            KeyCode::Char('w') | KeyCode::Up    => input.forward    = true,
            KeyCode::Char('s') | KeyCode::Down  => input.backward   = true,
            KeyCode::Char('a') | KeyCode::Left  => input.yaw_left   = true,
            KeyCode::Char('d') | KeyCode::Right => input.yaw_right  = true,
            KeyCode::Char('i')                  => input.pitch_up   = true,
            KeyCode::Char('k')                  => input.pitch_down = true,
            KeyCode::Char('j')                  => input.roll_left  = true,
            KeyCode::Char('l')                  => input.roll_right = true,
            KeyCode::Char(' ')                  => input.fire       = true,
            KeyCode::Char('m') => {
                // Toggle: dismiss message if showing, recall last if not.
                if !world.message.text.is_empty() {
                    world.message.text.clear();
                } else if !world.message.last_text.is_empty() {
                    world.message.text = world.message.last_text.clone();
                    world.message.age = 0.0;
                }
            }
            KeyCode::Char('u') => lock_nearest_enemy(world),
            KeyCode::Char('y') => cycle_target(world),
            KeyCode::Char('f') => cycle_weapon(world),
            KeyCode::Char('x') => {
                world.warpspeed = !world.warpspeed;
                let status = if world.warpspeed { "WARP ON" } else { "WARP OFF" };
                crate::ui::console::console_add(world, status);
            }
            KeyCode::Char('z')                  => *dense_stars  = !*dense_stars,
            KeyCode::Char('b')                  => *show_lines   = !*show_lines,
            KeyCode::Char('o')                  => *show_orrery  = !*show_orrery,
            KeyCode::Char('n')                  => *show_names   = !*show_names,
            KeyCode::Char('t')                  => *show_texture = !*show_texture,
            KeyCode::Char('c')                  => *orbit_cam    = !*orbit_cam,
            KeyCode::Char('g') => {
                // Toggle orbit autopilot on nearest planet.
                if *autopilot != AutopilotMode::Off {
                    *autopilot = AutopilotMode::Off;
                } else if let Some(idx) = find_nearest_planet(world) {
                    *autopilot = AutopilotMode::Orbit { planet_idx: idx };
                    *orbit_cam = true;  // auto-switch to third-person for the cinematic view
                }
            }
            _ => {}
        }
    }
    Ok(false)
}

fn is_any_keypress(ev: &Event) -> bool {
    matches!(
        ev,
        Event::Key(KeyEvent {
            kind: crossterm::event::KeyEventKind::Press,
            ..
        })
    )
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn start_new_game(world: &mut World) -> Screen {
    try_load_mission(world, START_MISSION);
    briefing_or_play(world)
}

fn load_slot(world: &mut World, mission: &str) -> Screen {
    try_load_mission(world, mission);
    briefing_or_play(world)
}

fn briefing_or_play(world: &World) -> Screen {
    if !world.briefing.is_empty() {
        let scores = top_scores(&world.mission_file);
        Screen::Briefing { scores }
    } else {
        Screen::Playing
    }
}

/// Lock the nearest active enemy target.
fn lock_nearest_enemy(world: &mut World) {
    let pos = world.player.pos;
    let nearest = world.targets.iter().enumerate()
        .filter(|(_, t)| t.age > 0.0 && !t.hidden && !t.friendly)
        .min_by(|(_, a), (_, b)| {
            let da = (a.pos - pos).mag2();
            let db = (b.pos - pos).mag2();
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(i, _)| i);
    world.lock.target = nearest;
    if let Some(i) = nearest {
        let name = world.targets[i].name.clone();
        crate::ui::console::console_add(world, format!("Locked: {}", name));
    }
}

/// Cycle to the next active target (enemy or friendly).
fn cycle_target(world: &mut World) {
    let active: Vec<usize> = world.targets.iter().enumerate()
        .filter(|(_, t)| t.age > 0.0 && !t.hidden)
        .map(|(i, _)| i)
        .collect();
    if active.is_empty() {
        world.lock.target = None;
        return;
    }
    let current = world.lock.target;
    let next = match current {
        Some(cur) => {
            let pos = active.iter().position(|&i| i == cur).unwrap_or(0);
            active[(pos + 1) % active.len()]
        }
        None => active[0],
    };
    world.lock.target = Some(next);
    let name = world.targets[next].name.clone();
    crate::ui::console::console_add(world, format!("Target: {}", name));
}

/// Cycle to the next available weapon.
fn cycle_weapon(world: &mut World) {
    let n = world.weapons.len();
    if n == 0 { return; }
    world.player.weapon = (world.player.weapon + 1) % n;
    let name = world.weapons[world.player.weapon].name.clone();
    crate::ui::console::console_add(world, format!("Weapon: {}", name));
}

fn player_fire(world: &mut World) {
    let wep = world.player.weapon;
    if wep >= world.weapons.len() { return; }
    world.player.msl_idle += world.delta_t;
    if world.player.msl_idle >= world.weapons[wep].idle {
        world.player.msl_idle = 0.0;
        let pos = world.player.pos;
        let vel = world.player.vel;
        let dir = world.player.view;
        fire_missile(world, pos, vel, dir, true, wep, -1);
    }
}

fn try_load_mission(world: &mut World, filename: &str) {
    let missions_dir = Path::new(MISSIONS_DIR);
    let models_dir = Path::new(MODELS_DIR);
    if let Err(e) = load_mission(world, filename, missions_dir, models_dir) {
        world.briefing = format!("Mission load error: {}", e);
    }
}

fn apply_prefs(prefs: &Prefs, world: &mut World) {
    world.player.name = prefs.player_name.clone();
    world.gravity = prefs.gravity;
    world.vulnerable = !prefs.vulnerable; // vulnerable=false means god mode OFF (takes damage)
}

/// Called from scoring to add a kill message.
#[allow(dead_code)]
pub fn kill_message(world: &mut World, name: &str) {
    console_add(world, format!("Destroyed: {}", name));
}
