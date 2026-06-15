use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::math::Vec3;
use crate::renderer::{
    canvas::BrailleCanvas,
    projection::Camera,
    stars::draw_stars,
    viewport::{draw_enemy_ships, draw_friendly_ships, draw_junk},
};
use crate::types::World;
use crate::ui::console::active_lines;

use super::radar::render_radar;

const HUD_HEIGHT: u16 = 12;

pub fn render(
    frame: &mut Frame,
    world: &World,
    stars: &[(Vec3, f32)],
    dense: bool,
    show_orrery: bool,
    show_names: bool,
    paused: bool,
) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(HUD_HEIGHT)])
        .split(area);

    render_viewport(frame, chunks[0], world, stars, dense, show_orrery, show_names);
    render_hud(frame, chunks[1], world);

    if paused {
        render_pause_overlay(frame, area);
    }
}

fn render_viewport(
    frame: &mut Frame,
    area: Rect,
    world: &World,
    stars: &[(Vec3, f32)],
    dense: bool,
    show_orrery: bool,
    show_names: bool,
) {
    if show_orrery {
        crate::ui::orrery::render_orrery(frame, area, world, show_names);
        return;
    }

    let w_dots = area.width as u32 * 2;
    let h_dots = area.height as u32 * 4;
    if w_dots == 0 || h_dots == 0 {
        return;
    }

    let camera = Camera::from_player(&world.player, w_dots, h_dots);

    let mut c_stars    = BrailleCanvas::new(w_dots, h_dots);
    let mut c_junk     = BrailleCanvas::new(w_dots, h_dots);
    let mut c_planets  = BrailleCanvas::new(w_dots, h_dots);
    let mut c_friendly = BrailleCanvas::new(w_dots, h_dots);
    let mut c_enemy    = BrailleCanvas::new(w_dots, h_dots);
    let mut c_fx       = BrailleCanvas::new(w_dots, h_dots);

    draw_stars(&mut c_stars, &camera, stars, dense);
    draw_junk(&mut c_junk, &camera, world.player.vel, world.abs_t);
    crate::renderer::planets::draw_planets(&mut c_planets, &camera, world);
    draw_friendly_ships(&mut c_friendly, &camera, world);
    draw_enemy_ships(&mut c_enemy, &camera, world);
    crate::combat::explosions::draw_missiles(&mut c_fx, &camera, world);
    crate::combat::explosions::draw_booms(&mut c_fx, &camera, world);

    let layers: &[(&BrailleCanvas, Color)] = &[
        (&c_stars,    Color::White),
        (&c_junk,     Color::DarkGray),
        (&c_planets,  Color::Cyan),
        (&c_friendly, Color::Green),
        (&c_enemy,    Color::Red),
        (&c_fx,       Color::Yellow),
    ];
    render_colored_layers(frame, area, layers);

    render_console_overlay(frame, area, world);

    if show_names {
        render_3d_name_overlays(frame, area, world, &camera);
    }
}

/// Merge multiple braille canvas layers into a single colored paragraph.
/// Layers are ordered lowest to highest priority; when multiple layers have dots
/// in the same cell, their bits are OR'd together and the highest-priority layer's
/// color wins.
fn render_colored_layers(frame: &mut Frame, area: Rect, layers: &[(&BrailleCanvas, Color)]) {
    // Collect rows from each layer up front.
    let layer_rows: Vec<Vec<String>> = layers.iter().map(|(c, _)| c.rows()).collect();

    let row_count = layer_rows.first().map(|r| r.len()).unwrap_or(0);

    let mut lines: Vec<Line> = Vec::with_capacity(row_count);

    for row_idx in 0..row_count {
        // Build a flat list of (char_as_u32, color) for every column in this row.
        let col_count = layer_rows
            .first()
            .and_then(|r| r.get(row_idx))
            .map(|s| s.chars().count())
            .unwrap_or(0);

        // For each terminal cell, compute merged braille bits and winning color.
        let mut cells: Vec<(char, Color)> = Vec::with_capacity(col_count);

        for col_idx in 0..col_count {
            let mut merged_bits: u32 = 0;
            let mut cell_color = Color::Reset;

            for (li, (_, layer_color)) in layers.iter().enumerate() {
                if let Some(row_str) = layer_rows.get(li).and_then(|r| r.get(row_idx)) {
                    if let Some(ch) = row_str.chars().nth(col_idx) {
                        let code = ch as u32;
                        if code >= 0x2800 && code <= 0x28FF {
                            let bits = code - 0x2800;
                            if bits > 0 {
                                merged_bits |= bits;
                                cell_color = *layer_color;
                            }
                        }
                    }
                }
            }

            let out_char = if merged_bits > 0 {
                char::from_u32(0x2800 + merged_bits).unwrap_or('⠀')
            } else {
                ' '
            };
            cells.push((out_char, cell_color));
        }

        // Run-length encode into Spans grouped by color.
        let mut spans: Vec<Span<'static>> = Vec::new();
        let mut run_start = 0;
        while run_start < cells.len() {
            let run_color = cells[run_start].1;
            let mut run_end = run_start + 1;
            while run_end < cells.len() && cells[run_end].1 == run_color {
                run_end += 1;
            }
            let text: String = cells[run_start..run_end].iter().map(|(c, _)| *c).collect();
            if run_color == Color::Reset {
                spans.push(Span::raw(text));
            } else {
                spans.push(Span::styled(text, Style::default().fg(run_color)));
            }
            run_start = run_end;
        }

        lines.push(Line::from(spans));
    }

    frame.render_widget(Paragraph::new(lines), area);
}

fn render_pause_overlay(frame: &mut Frame, area: Rect) {
    const W: u16 = 24;
    const H: u16 = 5;
    let x = area.x + area.width.saturating_sub(W) / 2;
    let y = area.y + area.height.saturating_sub(H) / 2;
    let popup = Rect { x, y, width: W, height: H };

    frame.render_widget(Clear, popup);
    frame.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .title(" Paused "),
        popup,
    );

    let inner = Rect { x: x + 1, y: y + 1, width: W - 2, height: H - 2 };
    let lines = vec![
        Line::from(Span::styled(
            "  II  PAUSED  II  ",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "   [P] to resume  ",
            Style::default().fg(Color::DarkGray),
        )),
    ];
    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_hud(frame: &mut Frame, area: Rect, world: &World) {
    // Split HUD: radar on left (~23 cols), stats on right.
    let radar_width = 23u16;
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(radar_width),
            Constraint::Min(20),
        ])
        .split(area);

    render_radar_panel(frame, chunks[0], world);
    render_stats_panel(frame, chunks[1], world);
}

fn render_radar_panel(frame: &mut Frame, area: Rect, world: &World) {
    let rows = render_radar(world);
    let lines: Vec<Line> = rows.into_iter().map(|s| Line::from(s)).collect();
    let block = Block::default().borders(Borders::ALL).title(" Radar ");
    frame.render_widget(Paragraph::new(lines).block(block), area);
}

fn render_stats_panel(frame: &mut Frame, area: Rect, world: &World) {
    let p = &world.player;
    let speed = p.vel.mag2().sqrt();
    let shields_pct = if p.maxshields > 0.0 {
        (p.shields / p.maxshields).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let throttle_pct = (p.throttle / 10.0).clamp(0.0, 1.0);

    let bar_width = (area.width.saturating_sub(20)).max(8) as usize;

    let mut lines: Vec<Line> = vec![
        // Row 0: shields bar
        bar_line("Shields", shields_pct, bar_width, Color::Green),
        // Row 1: throttle bar
        bar_line("Throttle", throttle_pct, bar_width, Color::Cyan),
        // Row 2: speed + position
        Line::from(format!(
            " Spd:{:.3}  Pos:({:.1},{:.1},{:.1})",
            speed, p.pos.x, p.pos.y, p.pos.z
        )),
        // Row 3: score + weapon
        Line::from(format!(
            " Score:{}  Wpn:{}",
            p.score,
            weapon_name(world),
        )),
    ];

    // Row 4: locked target info.
    if let Some(t_idx) = world.lock.target {
        if let Some(t) = world.targets.get(t_idx) {
            if t.age > 0.0 {
                let dist = (t.pos - p.pos).mag2().sqrt();
                let friendly_tag = if t.friendly { "[ally]" } else { "[enemy]" };
                lines.push(Line::from(format!(
                    " Target: {} {:.2}u {}",
                    t.name, dist, friendly_tag
                )));
            }
        }
    }

    // Row 5: waypoint arrow + distance.
    let wp_idx = p.waypoint;
    if wp_idx < world.nwaypoints {
        let wp_pos = world.waypoints[wp_idx].pos;
        let dist = (wp_pos - p.pos).mag2().sqrt();
        let arrow = waypoint_arrow(p.view, p.up, p.right, wp_pos - p.pos);
        lines.push(Line::from(format!(" WP#{} {} {:.2}u", wp_idx, arrow, dist)));
    }

    // Last row: mission message or key hint.
    let msg = if !world.message.text.is_empty() {
        world.message.text.lines().next().unwrap_or("").to_string()
    } else {
        format!(" {}  [WASD/IJKL fly  SPC fire  P pause  O orrery  N names  Q quit]", world.mission_file)
    };
    lines.push(Line::from(msg));

    let block = Block::default().borders(Borders::ALL).title(" HUD ");
    frame.render_widget(Paragraph::new(lines).block(block), area);
}

/// Overlay planet/target names on the 3D viewport using projected positions.
fn render_3d_name_overlays(frame: &mut Frame, area: Rect, world: &World, camera: &Camera) {
    // Planets — cyan labels.
    for planet in &world.planets {
        if planet.hidden || planet.radius <= 0.0 {
            continue;
        }
        if let Some((dx, dy)) = camera.project_point(planet.pos) {
            let cell_x = area.x + (dx / 2) as u16 + 1;
            let cell_y = area.y + (dy / 4) as u16;
            let w = planet.name.len() as u16;
            if cell_y < area.y + area.height && cell_x + w <= area.x + area.width {
                let r = Rect { x: cell_x, y: cell_y, width: w, height: 1 };
                frame.render_widget(
                    Paragraph::new(Span::styled(
                        planet.name.clone(),
                        Style::default().fg(Color::Cyan),
                    )),
                    r,
                );
            }
        }
    }

    // Active targets — red (enemy) or green (friendly).
    for target in &world.targets {
        if target.age <= 0.0 || target.hidden {
            continue;
        }
        if let Some((dx, dy)) = camera.project_point(target.pos) {
            let cell_x = area.x + (dx / 2) as u16 + 1;
            let cell_y = area.y + (dy / 4) as u16;
            let w = target.name.len() as u16;
            if cell_y < area.y + area.height && cell_x + w <= area.x + area.width {
                let color = if target.friendly { Color::Green } else { Color::Red };
                let r = Rect { x: cell_x, y: cell_y, width: w, height: 1 };
                frame.render_widget(
                    Paragraph::new(Span::styled(
                        target.name.clone(),
                        Style::default().fg(color),
                    )),
                    r,
                );
            }
        }
    }
}

fn render_console_overlay(frame: &mut Frame, area: Rect, world: &World) {
    let lines = active_lines(world);
    if lines.is_empty() {
        return;
    }
    let h = lines.len() as u16;
    let overlay = Rect {
        x: area.x + 1,
        y: area.y + area.height.saturating_sub(h + 1),
        width: area.width.saturating_sub(2),
        height: h,
    };
    let rlines: Vec<Line> = lines
        .iter()
        .map(|l| Line::from(Span::styled(l.to_string(), Style::default().fg(Color::Green))))
        .collect();
    frame.render_widget(Paragraph::new(rlines), overlay);
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn bar_line(label: &str, frac: f64, width: usize, color: Color) -> Line<'static> {
    let filled = (frac * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);
    let bar: String = "█".repeat(filled) + &"░".repeat(empty);
    let pct = (frac * 100.0) as u32;
    Line::from(vec![
        Span::raw(format!(" {:<8} ", label)),
        Span::styled(bar, Style::default().fg(color)),
        Span::raw(format!(" {:3}%", pct)),
    ])
}

fn weapon_name(world: &World) -> &str {
    let wep = world.player.weapon;
    if wep < world.weapons.len() {
        &world.weapons[wep].name
    } else {
        "?"
    }
}

/// 8-direction compass arrow toward a world-space offset from the player.
fn waypoint_arrow(view: crate::math::Vec3, up: crate::math::Vec3, right: crate::math::Vec3, delta: crate::math::Vec3) -> &'static str {
    if delta.mag2() < 1e-10 { return "·"; }
    let d = delta.normalize();
    let fwd = view.dot(d);
    let r = right.dot(d);
    let u = up.dot(d);

    // Project onto up/right plane for compass bearing; use fwd for front/back bias.
    let angle = u.atan2(r).to_degrees();
    // Bias: if mostly forward, up arrow; if mostly back, down arrow.
    if fwd > 0.7 {
        "↑"
    } else if fwd < -0.7 {
        "↓"
    } else {
        match (((angle + 360.0 + 22.5) % 360.0) / 45.0) as u32 {
            0 => "→",
            1 => "↗",
            2 => "↑",
            3 => "↖",
            4 => "←",
            5 => "↙",
            6 => "↓",
            7 => "↘",
            _ => "→",
        }
    }
}
