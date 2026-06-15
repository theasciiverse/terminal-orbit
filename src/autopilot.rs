use crate::constants::G;
use crate::math::Vec3;
use crate::types::World;

#[derive(Clone, Copy, PartialEq)]
pub enum AutopilotMode {
    Off,
    Orbit { planet_idx: usize },
}

/// Find the index of the nearest non-hidden planet with mass.
pub fn find_nearest_planet(world: &World) -> Option<usize> {
    let pos = world.player.pos;
    world.planets.iter().enumerate()
        .filter(|(_, p)| !p.hidden && p.mass > 0.0)
        .min_by(|(_, a), (_, b)| {
            let da = (a.pos - pos).mag2();
            let db = (b.pos - pos).mag2();
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(i, _)| i)
}

/// Prograde direction for a circular orbit around `planet` at position `pos`.
/// Planets orbit in the XY plane, so the solar system "north" is +Z.
/// Prograde = Z_hat × r_hat (counterclockwise when viewed from +Z).
fn prograde_dir(pos: Vec3, planet_pos: Vec3) -> Option<Vec3> {
    let radial = pos - planet_pos;
    if radial.mag2() < 1e-12 { return None; }
    let r_hat = radial.normalize();
    // Z × r_hat gives the prograde (eastward) tangent in the XY orbital plane.
    let z_hat = Vec3::new(0.0, 0.0, 1.0);
    let prograde = z_hat.cross(r_hat);
    if prograde.mag2() > 1e-8 {
        Some(prograde.normalize())
    } else {
        // Radial is purely along Z — use X as fallback.
        Some(Vec3::new(1.0, 0.0, 0.0))
    }
}

/// Target circular-orbit velocity at the player's current position.
pub fn orbit_velocity(world: &World, planet_idx: usize) -> Vec3 {
    let planet = &world.planets[planet_idx];
    let r = (world.player.pos - planet.pos).mag2().sqrt();
    if r < 1e-6 { return Vec3::zero(); }

    let v_orbit = (G * planet.mass / r).sqrt();
    prograde_dir(world.player.pos, planet.pos)
        .map(|d| d * v_orbit)
        .unwrap_or(Vec3::zero())
}

/// One autopilot tick — drives the ship into a stable close planar orbit.
///
/// Strategy (three simultaneous actions each frame):
///   1. Damp the radial velocity (cancel any drift toward / away from planet).
///   2. Accelerate the tangential velocity toward circular-orbit speed.
///   3. Zero the out-of-plane (Z) velocity to keep the orbit in the XY plane.
///
/// Time constants are short (≈1 s) so orbit insertion is snappy.
pub fn tick_autopilot(world: &mut World, mode: AutopilotMode, dt: f64) {
    let AutopilotMode::Orbit { planet_idx } = mode else { return };
    if planet_idx >= world.planets.len() { return; }

    let planet_pos = world.planets[planet_idx].pos;
    let planet_mass = world.planets[planet_idx].mass;

    let radial = world.player.pos - planet_pos;
    let r = radial.mag2().sqrt();
    if r < 1e-6 { return; }

    let r_hat = radial / r;
    let v_orbit = (G * planet_mass / r).sqrt();

    let Some(prograde) = prograde_dir(world.player.pos, planet_pos) else { return };

    let vel = world.player.vel;

    // Decompose current velocity into three components.
    let v_radial     = r_hat   * vel.dot(r_hat);    // toward/away from planet
    let v_zplane     = Vec3::new(0.0, 0.0, vel.z);  // out-of-orbital-plane
    let v_tangential = vel - v_radial - v_zplane;   // already in-plane tangential

    // Target: pure prograde tangential, no radial, no Z.
    let target_tangential = prograde * v_orbit;

    // Blend each component independently, fast time-constant.
    let alpha_rad = (dt / 0.8).min(1.0);  // damp radial quickly
    let alpha_tan = (dt / 1.2).min(1.0);  // smooth up to orbit speed
    let alpha_z   = (dt / 0.8).min(1.0);  // flatten to XY plane quickly

    let new_vel =
        v_radial      * (1.0 - alpha_rad)                          // radial → 0
        + v_tangential + (target_tangential - v_tangential) * alpha_tan  // tangential → orbit
        + v_zplane    * (1.0 - alpha_z);                           // z → 0

    world.player.vel = new_vel;

    // Orient the ship nose along prograde.
    let beta = (dt * 3.0).min(1.0);
    let blended = world.player.view + (prograde - world.player.view) * beta;
    if blended.mag2() > 1e-10 {
        world.player.view = blended.normalize();
    }
    // Z_hat is the natural "up" in the orbital plane.
    let z_hat = Vec3::new(0.0, 0.0, 1.0);
    let right = world.player.view.cross(z_hat);
    if right.mag2() > 1e-10 {
        world.player.right = right.normalize();
        world.player.up = world.player.right.cross(world.player.view).normalize();
    }
}

/// Name of the planet the autopilot is locked onto.
pub fn autopilot_target_name(world: &World, mode: AutopilotMode) -> Option<&str> {
    if let AutopilotMode::Orbit { planet_idx } = mode {
        world.planets.get(planet_idx).map(|p| p.name.as_str())
    } else {
        None
    }
}
