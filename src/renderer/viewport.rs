use std::f64::consts::PI;

use crate::combat::explosions::{draw_booms, draw_missiles};
use crate::math::Vec3;
use crate::model::Model;
use crate::types::{Player, Target, World};
use super::canvas::BrailleCanvas;
use super::planets::draw_planets;
use super::projection::Camera;

pub fn draw_scene(canvas: &mut BrailleCanvas, camera: &Camera, world: &World) {
    for target in &world.targets {
        if target.age > 0.0 && !target.hidden && !target.invisible {
            if let Some(midx) = target.model {
                if midx < world.models.len() {
                    draw_target_ship(canvas, camera, target, &world.models[midx]);
                }
            }
        }
    }
    draw_planets(canvas, camera, world);
    draw_missiles(canvas, camera, world);
    draw_booms(canvas, camera, world);
}

/// Draw only enemy (non-friendly) ships.
pub fn draw_enemy_ships(canvas: &mut BrailleCanvas, camera: &Camera, world: &World) {
    for target in &world.targets {
        if target.age > 0.0 && !target.hidden && !target.invisible && !target.friendly {
            if let Some(midx) = target.model {
                if midx < world.models.len() {
                    draw_target_ship(canvas, camera, target, &world.models[midx]);
                }
            }
        }
    }
}

/// Draw only friendly ships.
pub fn draw_friendly_ships(canvas: &mut BrailleCanvas, camera: &Camera, world: &World) {
    for target in &world.targets {
        if target.age > 0.0 && !target.hidden && !target.invisible && target.friendly {
            if let Some(midx) = target.model {
                if midx < world.models.len() {
                    draw_target_ship(canvas, camera, target, &world.models[midx]);
                }
            }
        }
    }
}

/// Draw a wireframe ship model at the target's position/orientation.
/// Model axes: +X → target.view, +Y → target.right, +Z → target.up  (mirrors LookAt in orbit.c)
pub fn draw_target_ship(canvas: &mut BrailleCanvas, camera: &Camera, target: &Target, model: &Model) {
    for edge in &model.edges {
        let w0 = model_to_world(edge[0], target);
        let w1 = model_to_world(edge[1], target);
        draw_edge_world(canvas, camera, w0, w1);
    }
}

/// Transform a model-space vertex to world space using the target's orientation matrix.
fn model_to_world(v: Vec3, t: &Target) -> Vec3 {
    t.pos + t.view * v.x + t.right * v.y + t.up * v.z
}

/// 8 debris dots orbiting the viewport center, drifting opposite to the
/// player's velocity.  Gives a parallax cue for Newtonian motion.
pub fn draw_junk(canvas: &mut BrailleCanvas, camera: &Camera, player_vel: Vec3, t: f64) {
    let v_r = player_vel.dot(camera.right);
    let v_u = player_vel.dot(camera.up);

    let w = camera.w_dots as f64;
    let h = camera.h_dots as f64;
    let spread = w.min(h) * 0.38;
    let cx = w / 2.0;
    let cy = h / 2.0;

    for i in 0..8 {
        let angle = i as f64 * PI / 4.0;
        // Base position on a ring around the viewport center.
        let base_x = cx + angle.cos() * spread;
        let base_y = cy - angle.sin() * spread;
        // Drift: move opposite to velocity so debris appears to flow past.
        // rem_euclid wraps the accumulated drift within the viewport.
        let dx = (-v_r * t * 2.5).rem_euclid(w);
        let dy = (v_u * t * 2.5).rem_euclid(h);
        let x = (base_x + dx).rem_euclid(w) as u32;
        let y = (base_y + dy).rem_euclid(h) as u32;
        canvas.set(x, y);
    }
}

/// Draw a simple wireframe ship at the player's position and orientation,
/// used for the third-person orbit camera view.
pub fn draw_player_ship_3p(canvas: &mut BrailleCanvas, camera: &Camera, player: &Player) {
    // A minimal "fighter" silhouette in model space:
    //   +X = forward (view), +Y = right, +Z = up
    let edges: &[(Vec3, Vec3)] = &[
        // Fuselage spine
        (Vec3::new(-3.0, 0.0, 0.0), Vec3::new( 3.0, 0.0, 0.0)),
        // Main wings (swept back)
        (Vec3::new( 1.0, 0.0, 0.0), Vec3::new(-1.5,  4.0, 0.0)),
        (Vec3::new( 1.0, 0.0, 0.0), Vec3::new(-1.5, -4.0, 0.0)),
        // Wing tips
        (Vec3::new(-1.5,  4.0, 0.0), Vec3::new(-3.0,  3.0, 0.0)),
        (Vec3::new(-1.5, -4.0, 0.0), Vec3::new(-3.0, -3.0, 0.0)),
        // Vertical tail fin
        (Vec3::new(-1.0, 0.0, 0.0), Vec3::new(-3.0, 0.0, 2.0)),
        (Vec3::new(-3.0, 0.0, 0.0), Vec3::new(-3.0, 0.0, 2.0)),
        // Cockpit outline
        (Vec3::new( 2.0, 0.0, 0.5), Vec3::new( 1.0, 0.8, 0.0)),
        (Vec3::new( 2.0, 0.0, 0.5), Vec3::new( 1.0,-0.8, 0.0)),
    ];

    // Model → world: nose along player.view, starboard along player.right, up along player.up.
    let scale = 0.12_f64; // ship spans ~1 unit so it frames against a ~1-unit-radius planet
    for (v0, v1) in edges {
        let w0 = player.pos + player.view * v0.x * scale
                            + player.right * v0.y * scale
                            + player.up * v0.z * scale;
        let w1 = player.pos + player.view * v1.x * scale
                            + player.right * v1.y * scale
                            + player.up * v1.z * scale;
        draw_edge_world(canvas, camera, w0, w1);
    }
}

/// Project and draw one world-space edge onto the braille canvas.
/// Clips to the near plane (z = 0.001 in camera space) before projecting.
/// Public so the planet renderer can reuse it.
pub fn draw_edge_world(canvas: &mut BrailleCanvas, cam: &Camera, w0: Vec3, w1: Vec3) {
    const NEAR: f64 = 0.001;
    let d0 = w0 - cam.pos;
    let d1 = w1 - cam.pos;
    let z0 = d0.dot(cam.view);
    let z1 = d1.dot(cam.view);

    let (d0, d1) = match clip_near(d0, d1, z0, z1, NEAR) {
        Some(pair) => pair,
        None => return,
    };

    if let (Some(p0), Some(p1)) = (cam.project_dp(d0), cam.project_dp(d1)) {
        canvas.line(p0.0, p0.1, p1.0, p1.1);
    }
}

/// Clip an edge to the near plane z = `near`.  Returns None if both points are behind.
fn clip_near(d0: Vec3, d1: Vec3, z0: f64, z1: f64, near: f64) -> Option<(Vec3, Vec3)> {
    if z0 >= near && z1 >= near {
        return Some((d0, d1));
    }
    if z0 < near && z1 < near {
        return None;
    }
    // One point behind — clip to near plane
    let t = (near - z0) / (z1 - z0);
    let mid = d0 + (d1 - d0) * t;
    if z0 < near {
        Some((mid, d1))
    } else {
        Some((d0, mid))
    }
}
