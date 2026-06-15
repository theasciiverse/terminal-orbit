use std::f64::consts::PI;

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::math::Vec3;
use crate::renderer::canvas::BrailleCanvas;
use crate::types::World;

/// Top-down 2D solar system view rendered into the viewport area.
/// Orbital paths, planet dots, and (when show_names=true) name labels.
pub fn render_orrery(frame: &mut Frame, area: Rect, world: &World, show_names: bool) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let w_dots = area.width as u32 * 2;
    let h_dots = area.height as u32 * 4;
    let mut canvas = BrailleCanvas::new(w_dots, h_dots);

    // Scale: fit all non-moon planetary orbits in view.
    let max_dist = world
        .planets
        .iter()
        .filter(|p| !p.hidden && p.dist > 0.0 && !p.is_moon)
        .map(|p| p.dist)
        .fold(1.0_f64, f64::max);

    let half = (w_dots.min(h_dots) as f64 * 0.48).max(1.0);
    let scale = half / max_dist;

    let cx = w_dots as f64 / 2.0;
    let cy = h_dots as f64 / 2.0;

    // World XY → braille dot coords (Y flipped: screen Y grows downward).
    let to_dot = |pos: Vec3| -> (i64, i64) {
        let x = (cx + pos.x * scale).round() as i64;
        let y = (cy - pos.y * scale).round() as i64;
        (x, y)
    };
    let in_canvas = |x: i64, y: i64| -> bool {
        x >= 0 && y >= 0 && x < w_dots as i64 && y < h_dots as i64
    };
    let safe_set = |canvas: &mut BrailleCanvas, x: i64, y: i64| {
        if in_canvas(x, y) {
            canvas.set(x as u32, y as u32);
        }
    };

    // Draw orbital circles for non-moons.
    const N_ORBIT: usize = 64;
    let sol_pos = world.planets.first().map(|p| p.pos).unwrap_or(Vec3::zero());

    for planet in world.planets.iter().skip(1) {
        if planet.hidden || planet.dist <= 0.0 || planet.is_moon {
            continue;
        }
        for i in 0..N_ORBIT {
            let a0 = 2.0 * PI * i as f64 / N_ORBIT as f64;
            let a1 = 2.0 * PI * (i + 1) as f64 / N_ORBIT as f64;
            let p0 = sol_pos + Vec3::new(planet.dist * a0.sin(), planet.dist * a0.cos(), 0.0);
            let p1 = sol_pos + Vec3::new(planet.dist * a1.sin(), planet.dist * a1.cos(), 0.0);
            let (x0, y0) = to_dot(p0);
            let (x1, y1) = to_dot(p1);
            if in_canvas(x0, y0) && in_canvas(x1, y1) {
                canvas.line(x0 as u32, y0 as u32, x1 as u32, y1 as u32);
            }
        }
    }

    // Collect planet dot positions for name overlays.
    let mut labels: Vec<(u16, u16, String, Color)> = Vec::new();

    for planet in &world.planets {
        if planet.hidden {
            continue;
        }
        let (x, y) = to_dot(planet.pos);
        safe_set(&mut canvas, x, y);
        // Larger dot for Sol and nearby planets.
        safe_set(&mut canvas, x + 1, y);
        safe_set(&mut canvas, x, y + 1);

        if in_canvas(x, y) {
            let cell_x = area.x.saturating_add((x as u16) / 2);
            let cell_y = area.y.saturating_add((y as u16) / 4);
            labels.push((cell_x, cell_y, planet.name.clone(), Color::Yellow));
        }
    }

    // Draw player as a cross (+).
    let (px, py) = to_dot(world.player.pos);
    for (dx, dy) in [(-3i64, 0), (-1, 0), (0, 0), (1, 0), (3, 0), (0, -2), (0, -1), (0, 1), (0, 2)] {
        safe_set(&mut canvas, px + dx, py + dy);
    }
    // Label the player.
    if in_canvas(px, py) {
        let cell_x = area.x.saturating_add((px as u16) / 2 + 1);
        let cell_y = area.y.saturating_add((py as u16) / 4);
        labels.push((cell_x, cell_y, world.player.name.clone(), Color::Green));
    }

    // Render braille canvas into the area.
    let lines: Vec<Line> = canvas.rows().into_iter().map(Line::from).collect();
    frame.render_widget(Paragraph::new(lines), area);

    // Orrery legend at top-left.
    let legend = Line::from(vec![
        Span::styled(" [O] 3D view  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[N] names", Style::default().fg(if show_names { Color::Yellow } else { Color::DarkGray })),
    ]);
    let legend_rect = Rect { x: area.x, y: area.y, width: area.width, height: 1 };
    frame.render_widget(Paragraph::new(legend), legend_rect);

    if show_names {
        for (lx, ly, name, color) in &labels {
            let lx = (*lx + 1).min(area.x + area.width.saturating_sub(name.len() as u16 + 1));
            if *ly >= area.y && *ly < area.y + area.height && lx < area.x + area.width {
                let w = (name.len() as u16).min(area.x + area.width - lx);
                let r = Rect { x: lx, y: *ly, width: w, height: 1 };
                frame.render_widget(
                    Paragraph::new(Span::styled(name.clone(), Style::default().fg(*color))),
                    r,
                );
            }
        }
    }
}
