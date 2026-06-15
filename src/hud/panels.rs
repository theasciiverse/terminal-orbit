use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::math::Vec3;
use crate::renderer::{
    canvas::BrailleCanvas,
    projection::Camera,
    stars::draw_stars,
    viewport::draw_scene,
};
use crate::types::World;
use crate::ui::console::active_lines;

use super::radar::render_radar;

const HUD_HEIGHT: u16 = 12;

pub fn render(frame: &mut Frame, world: &World, stars: &[Vec3], dense: bool) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(HUD_HEIGHT)])
        .split(area);

    render_viewport(frame, chunks[0], world, stars, dense);
    render_hud(frame, chunks[1], world);
}

fn render_viewport(frame: &mut Frame, area: Rect, world: &World, stars: &[Vec3], dense: bool) {
    let w_dots = area.width as u32 * 2;
    let h_dots = area.height as u32 * 4;

    if w_dots == 0 || h_dots == 0 {
        return;
    }

    let camera = Camera::from_player(&world.player, w_dots, h_dots);
    let mut canvas = BrailleCanvas::new(w_dots, h_dots);
    draw_stars(&mut canvas, &camera, stars, dense);
    draw_scene(&mut canvas, &camera, world);

    let lines: Vec<Line> = canvas.rows().into_iter().map(Line::from).collect();
    frame.render_widget(Paragraph::new(lines), area);

    // Console overlay: green messages at the bottom-left of the viewport.
    render_console_overlay(frame, area, world);
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
        format!(" {}  [WASD/IJKL fly  SPC fire  Q quit]", world.mission_file)
    };
    lines.push(Line::from(msg));

    let block = Block::default().borders(Borders::ALL).title(" HUD ");
    frame.render_widget(Paragraph::new(lines).block(block), area);
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
