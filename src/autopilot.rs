use crate::constants::{G, RMIN};
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

/// Standard orbit radius, in planet radii: low enough that the planet fills
/// the lower part of the orbit camera's frame.
pub const STANDARD_ORBIT: f64 = 1.25;

/// How fast the autopilot closes the gap to standard orbit (1/s).
const K_RADIAL: f64 = 0.6;
/// How fast it settles into the planet's equatorial (XY) plane (1/s).
const K_PLANE: f64 = 0.8;
/// Time constant for velocity changes, seconds.
const TAU: f64 = 0.5;

/// Standard orbit radius around `planet_idx`, in game units.
pub fn standard_orbit_radius(world: &World, planet_idx: usize) -> f64 {
    world.planets[planet_idx].radius * STANDARD_ORBIT
}

/// Circular-orbit speed at radius `r`, matched to the game's own gravity:
/// `physics::gravity` adds its pull once per frame (as orbit.c does) and
/// clamps r² at RMIN, so v = sqrt(a·r) with a = G·m / max(r², RMIN) / dt.
fn circular_speed(mass: f64, r: f64, dt: f64) -> f64 {
    let accel = G * mass / (r * r).max(RMIN) / dt;
    (accel * r).sqrt()
}

/// Target circular-orbit velocity at the player's current position.
pub fn orbit_velocity(world: &World, planet_idx: usize) -> Vec3 {
    let planet = &world.planets[planet_idx];
    let r = (world.player.pos - planet.pos).mag2().sqrt();
    if r < 1e-6 { return Vec3::zero(); }

    let v_orbit = circular_speed(planet.mass, r, world.delta_t);
    prograde_dir(world.player.pos, planet.pos)
        .map(|d| d * v_orbit)
        .unwrap_or(Vec3::zero())
}

/// One autopilot tick: fly to standard orbit and hold it, starship style.
///
/// The target velocity each frame is circular-orbit speed along prograde,
/// plus a radial term closing the gap to standard-orbit radius and a Z term
/// settling into the planet's equatorial plane.  The ship eases toward that
/// velocity, nose prograde and dorsal side away from the planet.
pub fn tick_autopilot(world: &mut World, mode: AutopilotMode, dt: f64) {
    let AutopilotMode::Orbit { planet_idx } = mode else { return };
    if planet_idx >= world.planets.len() { return; }

    let planet_pos = world.planets[planet_idx].pos;
    let planet_mass = world.planets[planet_idx].mass;
    let r_target = standard_orbit_radius(world, planet_idx);

    // Radius measured in the orbital plane, so a ship above a pole still
    // has a well-defined way down to the equator.
    let offset = world.player.pos - planet_pos;
    let mut flat = Vec3::new(offset.x, offset.y, 0.0);
    if flat.mag2() < 1e-12 {
        flat = Vec3::new(r_target, 0.0, 0.0);
    }
    let r = flat.mag2().sqrt();
    let r_hat = flat / r;
    let z_hat = Vec3::new(0.0, 0.0, 1.0);
    let prograde = z_hat.cross(r_hat).normalize();

    let target = prograde * circular_speed(planet_mass, r, dt)
        + r_hat * (-(r - r_target) * K_RADIAL)
        + z_hat * (-offset.z * K_PLANE);

    let alpha = (dt / TAU).min(1.0);
    world.player.vel = world.player.vel + (target - world.player.vel) * alpha;

    // With game gravity off nothing bends the path, and the ship would drift
    // outward off every tangent; supply the centripetal pull ourselves.
    if !world.gravity {
        let v_tan = world.player.vel.dot(prograde);
        world.player.vel = world.player.vel - r_hat * (v_tan * v_tan / r * dt);
    }

    // Nose along prograde, dorsal side facing away from the planet.
    let beta = (dt * 3.0).min(1.0);
    let blended = world.player.view + (prograde - world.player.view) * beta;
    if blended.mag2() > 1e-10 {
        world.player.view = blended.normalize();
    }
    let outward = offset.normalize();
    // Same handedness as move_player: right = up × view, up = view × right.
    let right = outward.cross(world.player.view);
    if right.mag2() > 1e-10 {
        world.player.right = right.normalize();
        world.player.up = world.player.view.cross(world.player.right).normalize();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::move_player;

    fn world_near_earth(offset: Vec3) -> (World, usize) {
        let mut w = World::new();
        crate::planet_data::init_weapons(&mut w);
        crate::planet_data::reset_planets(&mut w);
        let earth = w.planets.iter().position(|p| p.name == "Earth").unwrap();
        w.player.pos = w.planets[earth].pos + offset;
        (w, earth)
    }

    fn radius(w: &World, p: usize) -> f64 {
        (w.player.pos - w.planets[p].pos).mag2().sqrt()
    }

    fn fly(w: &mut World, mode: AutopilotMode, secs: f64) {
        for _ in 0..(secs / w.delta_t) as usize {
            tick_autopilot(w, mode, w.delta_t);
            move_player(w);
        }
    }

    #[test]
    fn settles_into_standard_orbit_with_gravity_on_or_off() {
        for gravity in [true, false] {
            settles_from_far_out_and_off_plane(gravity);
        }
    }

    fn settles_from_far_out_and_off_plane(gravity: bool) {
        let (mut w, earth) = world_near_earth(Vec3::new(12.0, -5.0, 4.0));
        w.gravity = gravity;
        let mode = AutopilotMode::Orbit { planet_idx: earth };
        fly(&mut w, mode, 20.0);
        let target = standard_orbit_radius(&w, earth);
        let r = radius(&w, earth);
        assert!((r - target).abs() < 0.02 * target,
            "gravity {}: r={:.3} target={:.3}", gravity, r, target);
        let dz = w.player.pos.z - w.planets[earth].pos.z;
        assert!(dz.abs() < 0.01, "still {:.3} out of the equatorial plane", dz);
        // Dorsal side away from the planet.
        let out = (w.player.pos - w.planets[earth].pos).normalize();
        assert!(w.player.up.dot(out) > 0.95);
    }

    #[test]
    fn orbit_holds_under_game_gravity_after_release() {
        let (mut w, earth) = world_near_earth(Vec3::new(3.0, 0.0, 0.0));
        w.gravity = true;
        fly(&mut w, AutopilotMode::Orbit { planet_idx: earth }, 20.0);
        let target = standard_orbit_radius(&w, earth);
        // Autopilot off: gravity alone, for about two full orbits.
        let period = 2.0 * std::f64::consts::PI * target / w.player.vel.mag2().sqrt();
        let mut worst: f64 = 0.0;
        for _ in 0..(2.0 * period / w.delta_t) as usize {
            move_player(&mut w);
            worst = worst.max((radius(&w, earth) - target).abs() / target);
        }
        assert!(worst < 0.05, "orbit drifted {:.1}% from standard radius", worst * 100.0);
    }
}
