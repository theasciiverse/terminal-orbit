use std::collections::HashMap;
use std::f64::consts::PI;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::autopilot::AutopilotMode;
use crate::math::Vec3;
use crate::renderer::{
    canvas::BrailleCanvas,
    planets::{draw_single_planet, planet_color},
    projection::Camera,
    stars::{draw_constellation_lines, draw_stars},
    texture::PlanetTexture,
    viewport::{draw_enemy_ships, draw_friendly_ships, draw_junk, draw_player_ship_3p},
};
use crate::types::World;
use crate::ui::console::active_lines;

use super::radar::render_radar;

const HUD_HEIGHT: u16 = 12;
/// Distance behind the ship for the third-person camera.
const CAM_3P_DIST:   f64 = 2.0;
/// Height above the ship for the third-person camera.
const CAM_3P_HEIGHT: f64 = 0.5;

pub fn render(
    frame: &mut Frame,
    world: &World,
    stars: &[(Vec3, f32)],
    dense: bool,
    show_orrery: bool,
    show_names: bool,
    paused: bool,
    show_texture: bool,
    textures: &HashMap<String, PlanetTexture>,
    orbit_cam: bool,
    autopilot: AutopilotMode,
    show_lines: bool,
) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(HUD_HEIGHT)])
        .split(area);

    render_viewport(frame, chunks[0], world, stars, dense, show_orrery, show_names, show_texture, textures, orbit_cam, autopilot, show_lines);
    render_hud(frame, chunks[1], world, autopilot);

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
    show_texture: bool,
    textures: &HashMap<String, PlanetTexture>,
    orbit_cam: bool,
    autopilot: AutopilotMode,
    show_lines: bool,
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

    let camera = if orbit_cam {
        // Focus point: the orbited planet, or a point far ahead if no autopilot.
        let focus = match autopilot {
            AutopilotMode::Orbit { planet_idx } => {
                world.planets.get(planet_idx)
                    .map(|p| p.pos)
                    .unwrap_or_else(|| world.player.pos + world.player.view * 200.0)
            }
            AutopilotMode::Off => world.player.pos + world.player.view * 200.0,
        };
        Camera::third_person(
            world.player.pos,
            world.player.view,
            world.player.up,
            focus,
            w_dots,
            h_dots,
            CAM_3P_DIST,
            CAM_3P_HEIGHT,
        )
    } else {
        Camera::from_player(&world.player, w_dots, h_dots)
    };

    let mut c_stars_bright = BrailleCanvas::new(w_dots, h_dots);
    let mut c_stars_medium = BrailleCanvas::new(w_dots, h_dots);
    let mut c_stars_dim    = BrailleCanvas::new(w_dots, h_dots);
    let mut c_lines    = BrailleCanvas::new(w_dots, h_dots);
    let mut c_junk     = BrailleCanvas::new(w_dots, h_dots);
    let mut c_player   = BrailleCanvas::new(w_dots, h_dots);
    let mut c_friendly = BrailleCanvas::new(w_dots, h_dots);
    let mut c_enemy    = BrailleCanvas::new(w_dots, h_dots);
    let mut c_fx       = BrailleCanvas::new(w_dots, h_dots);

    draw_stars(&mut c_stars_bright, &mut c_stars_medium, &mut c_stars_dim, &camera, stars, dense);
    if show_lines {
        draw_constellation_lines(&mut c_lines, &camera);
    }
    // In first-person mode junk drifts with player velocity; in orbit cam keep it static.
    let junk_vel = if orbit_cam { Vec3::zero() } else { world.player.vel };
    draw_junk(&mut c_junk, &camera, junk_vel, world.abs_t);
    // Draw player ship wireframe in third-person.
    if orbit_cam {
        draw_player_ship_3p(&mut c_player, &camera, &world.player);
    }
    draw_friendly_ships(&mut c_friendly, &camera, world);
    draw_enemy_ships(&mut c_enemy, &camera, world);
    crate::combat::explosions::draw_missiles(&mut c_fx, &camera, world);
    crate::combat::explosions::draw_booms(&mut c_fx, &camera, world);

    // Build one canvas per planet with its own color (skip in texture mode — texture covers them).
    let mut planet_canvases: Vec<(BrailleCanvas, Color)> = Vec::new();
    if !show_texture {
        for p in 0..world.planets.len() {
            let planet = &world.planets[p];
            if planet.hidden || planet.radius <= 0.0 {
                continue;
            }
            let mut c = BrailleCanvas::new(w_dots, h_dots);
            draw_single_planet(&mut c, &camera, p, world);
            let color = planet_color(&planet.name);
            planet_canvases.push((c, color));
        }
    }

    // Base layers: dim stars, then brighter stars, then lines.
    let mut layers: Vec<(&BrailleCanvas, Color)> = vec![
        (&c_stars_dim,    Color::Rgb(80,  80,  80)),
        (&c_stars_medium, Color::Rgb(180, 180, 180)),
        (&c_lines,        Color::Rgb(45,  55,  95)),
        (&c_stars_bright, Color::White),
        (&c_junk,         Color::DarkGray),
    ];

    // Insert planet layers (each with its own color).
    for (c, color) in &planet_canvases {
        layers.push((c, *color));
    }

    // Player ship (third-person), then other ships and FX on top.
    if orbit_cam {
        layers.push((&c_player, Color::Cyan));
    }
    layers.push((&c_friendly, Color::Green));
    layers.push((&c_enemy,    Color::Red));
    layers.push((&c_fx,       Color::Yellow));

    render_colored_layers(frame, area, &layers);

    // Optional texture overlay on planets.
    if show_texture {
        render_planets_textured(frame, area, world, &camera, textures);
        // Re-stamp ship wireframe on top — the texture pass overwrites braille chars.
        if orbit_cam {
            render_braille_direct(frame, area, &c_player, Color::Cyan);
        }
    }

    render_console_overlay(frame, area, world);

    if show_names {
        render_3d_name_overlays(frame, area, world, &camera);
    }
}

/// Raycast a sphere at `center` with `radius` from `ray_origin` in direction `ray_dir` (unit).
/// Returns the distance t to the first intersection, or None if no hit.
fn raycast_planet(ray_origin: Vec3, ray_dir: Vec3, center: Vec3, radius: f64) -> Option<f64> {
    let oc = ray_origin - center;
    let b = 2.0 * oc.dot(ray_dir);
    let c = oc.dot(oc) - radius * radius;
    let discriminant = b * b - 4.0 * c;
    if discriminant < 0.0 {
        return None;
    }
    let sqrt_d = discriminant.sqrt();
    let t1 = (-b - sqrt_d) * 0.5;
    let t2 = (-b + sqrt_d) * 0.5;
    if t1 > 0.001 { Some(t1) }
    else if t2 > 0.001 { Some(t2) }
    else { None }
}

/// Render textured planets by raycasting every viewport cell against all planet spheres.
/// No bounding box — correct for any viewing angle, handles partial off-screen planets.
fn render_planets_textured(
    frame: &mut Frame,
    area: Rect,
    world: &World,
    camera: &Camera,
    textures: &HashMap<String, PlanetTexture>,
) {
    if area.width == 0 || area.height == 0 { return; }

    // Cull planets that are completely behind the camera.
    struct VisiblePlanet<'a> {
        planet: &'a crate::types::Planet,
        texture: Option<&'a PlanetTexture>,
        sun_dir: Vec3,
    }
    let sol_pos = world.planets.first().map(|p| p.pos).unwrap_or(Vec3::zero());
    let visible: Vec<VisiblePlanet> = world.planets.iter().filter_map(|planet| {
        if planet.hidden || planet.radius <= 0.0 { return None; }
        let dp = planet.pos - camera.pos;
        let fwd = dp.dot(camera.view);
        // Keep if any part of the sphere is in front: fwd + radius > 0
        if fwd + planet.radius <= 0.001 { return None; }
        let dist = dp.mag2().sqrt();
        let tex = textures.get(&planet.name.to_lowercase());
        let sun_dir = if dist > 0.001 { (sol_pos - planet.pos).normalize() } else { camera.view };
        Some(VisiblePlanet { planet, texture: tex, sun_dir })
    }).collect();

    if visible.is_empty() { return; }

    let w = area.width as f64;
    let h = area.height as f64;
    let buf = frame.buffer_mut();

    for cy in 0..area.height {
        for cx in 0..area.width {
            // NDC of cell centre.
            let ndc_x = ((cx as f64 + 0.5) / w) * 2.0 - 1.0;
            let ndc_y = 1.0 - ((cy as f64 + 0.5) / h) * 2.0;

            // World-space ray direction for this cell.
            let ray_dir = (camera.view
                + camera.right * (ndc_x * camera.aspect / camera.fov_scale)
                + camera.up    * (ndc_y           / camera.fov_scale))
                .normalize();

            // Find nearest planet hit.
            let mut best_t = f64::MAX;
            let mut best_color: Option<Color> = None;

            for vp in &visible {
                let Some(t) = raycast_planet(camera.pos, ray_dir, vp.planet.pos, vp.planet.radius) else { continue };
                if t >= best_t { continue; }
                best_t = t;

                let hit    = camera.pos + ray_dir * t;
                let normal = (hit - vp.planet.pos) / vp.planet.radius;

                // Rotate normal by planet oblicity (X-axis) before UV mapping.
                let ob  = vp.planet.oblicity.to_radians();
                let nx  = normal.x;
                let ny  = normal.y * ob.cos() + normal.z * ob.sin();
                let nz  = -normal.y * ob.sin() + normal.z * ob.cos();
                let u   = (nx.atan2(nz) + PI) / (2.0 * PI);
                let v   = (ny.asin() + PI / 2.0) / PI;

                let diffuse    = normal.dot(vp.sun_dir).max(0.0);
                // 0.40 ambient so the night side stays visible in cinematic view.
                let brightness = (0.40 + diffuse * 0.60).min(1.0);

                let (r, g, b) = if let Some(tex) = vp.texture {
                    tex.sample(u, v)
                } else {
                    match planet_color(&vp.planet.name) {
                        Color::Rgb(r, g, b) => (r, g, b),
                        _ => (120, 120, 120),
                    }
                };
                let r = (r as f64 * brightness) as u8;
                let g = (g as f64 * brightness) as u8;
                let b = (b as f64 * brightness) as u8;
                best_color = Some(Color::Rgb(r, g, b));
            }

            if let Some(color) = best_color {
                let screen_x = area.x + cx;
                let screen_y = area.y + cy;
                if let Some(cell) = buf.cell_mut(ratatui::layout::Position { x: screen_x, y: screen_y }) {
                    cell.set_char('█');
                    cell.set_fg(color);
                    cell.set_bg(Color::Black);
                }
            }
        }
    }
}

/// Write non-empty braille cells from `canvas` directly to the frame buffer.
/// Used to stamp a wireframe layer on top of texture pixels after the texture pass.
fn render_braille_direct(frame: &mut Frame, area: Rect, canvas: &BrailleCanvas, color: Color) {
    let rows = canvas.rows();
    let buf = frame.buffer_mut();
    for (row_idx, row_str) in rows.iter().enumerate() {
        let screen_y = area.y + row_idx as u16;
        if screen_y >= area.y + area.height { break; }
        for (col_idx, ch) in row_str.chars().enumerate() {
            if ch as u32 > 0x2800 {
                let screen_x = area.x + col_idx as u16;
                if screen_x >= area.x + area.width { break; }
                if let Some(cell) = buf.cell_mut(ratatui::layout::Position { x: screen_x, y: screen_y }) {
                    cell.set_char(ch);
                    cell.set_fg(color);
                }
            }
        }
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

fn render_hud(frame: &mut Frame, area: Rect, world: &World, autopilot: AutopilotMode) {
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
    render_stats_panel(frame, chunks[1], world, autopilot);
}

fn render_radar_panel(frame: &mut Frame, area: Rect, world: &World) {
    let rows = render_radar(world);
    let lines: Vec<Line> = rows.into_iter().map(|s| Line::from(s)).collect();
    let block = Block::default().borders(Borders::ALL).title(" Radar ");
    frame.render_widget(Paragraph::new(lines).block(block), area);
}

fn render_stats_panel(frame: &mut Frame, area: Rect, world: &World, autopilot: AutopilotMode) {
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

    // Autopilot status row.
    if let AutopilotMode::Orbit { planet_idx } = autopilot {
        let planet_name = world.planets.get(planet_idx)
            .map(|p| p.name.as_str()).unwrap_or("?");
        lines.push(Line::from(Span::styled(
            format!(" [AUTOPILOT]  Orbit: {}  — press G to disengage", planet_name),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )));
    }

    // Last row: mission message or key hint.
    let msg = if !world.message.text.is_empty() {
        world.message.text.lines().next().unwrap_or("").to_string()
    } else {
        format!(" {}  [WASD/IJKL fly  SPC fire  G orbit  C cam  T texture  B lines  O orrery  N names  Q quit]", world.mission_file)
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
