use crate::combat::explosions::{draw_booms, draw_missiles};
use crate::math::Vec3;
use crate::model::Model;
use crate::types::{Target, World};
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
