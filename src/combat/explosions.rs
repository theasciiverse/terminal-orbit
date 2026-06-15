use std::f64::consts::PI;

use crate::constants::BOOM_TIME;
use crate::math::Vec3;
use crate::renderer::canvas::BrailleCanvas;
use crate::renderer::projection::Camera;
use crate::types::World;

/// Spawn an explosion at `pos` with the given `size` (scale factor).
pub fn spawn_boom(world: &mut World, pos: Vec3, size: f64) {
    for boom in &mut world.booms {
        if boom.age <= 0.0 {
            boom.pos = pos;
            boom.age = world.delta_t.max(0.001);
            boom.size = size;
            boom.angle = 0.0;
            return;
        }
    }
    // All slots full — overwrite the oldest.
    if let Some(oldest) = world.booms.iter_mut().max_by(|a, b| a.age.partial_cmp(&b.age).unwrap()) {
        oldest.pos = pos;
        oldest.age = world.delta_t.max(0.001);
        oldest.size = size;
        oldest.angle = 0.0;
    }
}

/// Advance all active explosions one timestep.
pub fn move_booms(world: &mut World) {
    let dt = world.delta_t;
    for boom in &mut world.booms {
        if boom.age > 0.0 {
            boom.age += dt;
            if boom.age > BOOM_TIME {
                boom.age = 0.0;
            }
        }
    }
}

/// Draw all active explosions on the braille canvas.
/// Each boom is rendered as an expanding ring of dots.
pub fn draw_booms(canvas: &mut BrailleCanvas, camera: &Camera, world: &World) {
    const N_POINTS: usize = 16;
    const BASE_RADIUS_DOTS: f64 = 20.0;

    for boom in &world.booms {
        if boom.age <= 0.0 {
            continue;
        }

        let center = match camera.project_point(boom.pos) {
            Some(p) => p,
            None => continue,
        };

        let t = boom.age / BOOM_TIME;
        // Grow to max at t=0.5, shrink back to 0 at t=1.0.
        let scale = if t < 0.5 { t * 2.0 } else { (1.0 - t) * 2.0 };
        let r = scale * BASE_RADIUS_DOTS * boom.size;

        let cx = center.0 as f64;
        let cy = center.1 as f64;

        for i in 0..N_POINTS {
            let angle = (i as f64 / N_POINTS as f64) * 2.0 * PI;
            let px = (cx + r * angle.cos()).round() as i64;
            let py = (cy + r * angle.sin()).round() as i64;
            if px >= 0 && py >= 0 {
                canvas.set(px as u32, py as u32);
            }
        }

        // Also draw the centre dot for small booms.
        if r < 3.0 {
            canvas.set(center.0, center.1);
        }
    }
}

/// Draw all active missiles as small crosses on the braille canvas.
pub fn draw_missiles(canvas: &mut BrailleCanvas, camera: &Camera, world: &World) {
    for m in &world.missiles {
        if m.age <= 0.0 {
            continue;
        }
        if let Some((px, py)) = camera.project_point(m.pos) {
            canvas.set(px, py);
            // Small cross for visibility at a single braille-dot scale.
            if px >= 1 { canvas.set(px - 1, py); }
            canvas.set(px + 1, py);
            if py >= 1 { canvas.set(px, py - 1); }
            canvas.set(px, py + 1);
        }
    }
}
