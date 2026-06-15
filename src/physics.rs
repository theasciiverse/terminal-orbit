use crate::constants::*;
use crate::math::{rotate_about, Vec3};
use crate::types::*;

/// Compute gravity delta-v for one timestep at `pos`.
/// Port of Gravity() from orbit.c — planets farther than 200 units are ignored.
pub fn gravity(planets: &[Planet], pos: Vec3) -> Vec3 {
    let mut deltav = Vec3::default();
    for p in planets {
        let dp = p.pos - pos;
        let mut rr = dp.mag2();
        if rr > 200.0 * 200.0 {
            continue;
        }
        if rr < RMIN {
            rr = RMIN;
        }
        let r = rr.sqrt();
        let dv = G * p.mass / rr;
        deltav = deltav + dp * (dv / r);
    }
    deltav
}

/// Move the player one timestep. Port of MovePlayer() from player.c.
/// Skips: ViewLock, planet-crater handling, network sync.
pub fn move_player(world: &mut World) {
    let dt = world.delta_t;
    let theta = THETA * dt;
    let mut deltav = Vec3::default();

    let was_still = world.player.still;
    world.player.still = true;

    if world.player.move_left > 0.0 {
        let v = rotate_about(world.player.view, world.player.up, -theta * world.player.move_left);
        world.player.view = v.normalize();
        world.player.right = world.player.up.cross(world.player.view).normalize();
        world.player.still = false;
    }

    if world.player.move_right > 0.0 {
        let v = rotate_about(world.player.view, world.player.up, theta * world.player.move_right);
        world.player.view = v.normalize();
        world.player.right = world.player.up.cross(world.player.view).normalize();
        world.player.still = false;
    }

    if world.player.move_up > 0.0 {
        let v = rotate_about(world.player.view, world.player.right, theta * world.player.move_up);
        world.player.view = v.normalize();
        world.player.up = world.player.view.cross(world.player.right).normalize();
        world.player.still = false;
    }

    if world.player.move_down > 0.0 {
        let v = rotate_about(world.player.view, world.player.right, -theta * world.player.move_down);
        world.player.view = v.normalize();
        world.player.up = world.player.view.cross(world.player.right).normalize();
        world.player.still = false;
    }

    if world.player.move_pitchright > 0.0 {
        let v = rotate_about(world.player.up, world.player.view, -theta * world.player.move_pitchright);
        world.player.up = v.normalize();
        world.player.right = world.player.up.cross(world.player.view).normalize();
        world.player.still = false;
    }

    if world.player.move_pitchleft > 0.0 {
        let v = rotate_about(world.player.up, world.player.view, theta * world.player.move_pitchleft);
        world.player.up = v.normalize();
        world.player.right = world.player.up.cross(world.player.view).normalize();
        world.player.still = false;
    }

    if world.player.move_forward > 0.0 {
        match world.player.flight_model {
            FlightModel::Newtonian => {
                let scale = if world.warpspeed {
                    100.0 * world.player.move_forward * DELTAV * dt
                } else {
                    world.player.move_forward * DELTAV * dt
                };
                deltav = deltav + world.player.view * scale;
            }
            FlightModel::Arcade => {
                let inc = if world.warpspeed {
                    10.0 * world.player.move_forward * DELTAV * dt
                } else {
                    world.player.move_forward * DELTAV * dt
                };
                world.player.throttle = (world.player.throttle + inc).min(MAX_WARP_THROTTLE);
            }
        }
        world.player.still = false;
    }

    if world.player.move_backward > 0.0 {
        match world.player.flight_model {
            FlightModel::Newtonian => {
                let scale = if world.warpspeed {
                    -100.0 * world.player.move_backward * DELTAV * dt
                } else {
                    -world.player.move_backward * DELTAV * dt
                };
                deltav = deltav + world.player.view * scale;
            }
            FlightModel::Arcade => {
                let dec = if world.warpspeed {
                    10.0 * world.player.move_backward * DELTAV * dt
                } else {
                    world.player.move_backward * DELTAV * dt
                };
                world.player.throttle = (world.player.throttle - dec).max(0.0);
            }
        }
        world.player.still = false;
    }

    match world.player.flight_model {
        FlightModel::Newtonian => {
            world.player.vel = world.player.vel + deltav;
            if world.gravity {
                let grav = gravity(&world.planets, world.player.pos);
                world.player.vel = world.player.vel + grav;
            }
        }
        FlightModel::Arcade => {
            world.player.vel = world.player.view * world.player.throttle;
        }
    }

    world.player.pos = world.player.pos + world.player.vel * dt;

    // Regenerate shields
    world.player.shields = (world.player.shields + dt * SHIELD_REGEN).min(world.player.maxshields);
    world.player.msl_idle += dt;

    let _ = was_still; // used by network code — skip for now
}

/// Move a single target one timestep. Port of MoveTarget() from target.c.
/// Skips: planet-crater check, network-only gravity path.
pub fn move_target(world: &mut World, t: usize) {
    let dt = world.delta_t;
    let theta = THETA * dt;
    let mut deltav = Vec3::default();

    if world.targets[t].move_left > 0.0 {
        let v = rotate_about(world.targets[t].view, world.targets[t].up,
                              -theta * world.targets[t].move_left);
        world.targets[t].view = v.normalize();
        world.targets[t].right = world.targets[t].up.cross(world.targets[t].view).normalize();
    }

    if world.targets[t].move_right > 0.0 {
        let v = rotate_about(world.targets[t].view, world.targets[t].up,
                              theta * world.targets[t].move_right);
        world.targets[t].view = v.normalize();
        world.targets[t].right = world.targets[t].up.cross(world.targets[t].view).normalize();
    }

    if world.targets[t].move_up > 0.0 {
        let v = rotate_about(world.targets[t].view, world.targets[t].right,
                              theta * world.targets[t].move_up);
        world.targets[t].view = v.normalize();
        world.targets[t].up = world.targets[t].view.cross(world.targets[t].right).normalize();
    }

    if world.targets[t].move_down > 0.0 {
        let v = rotate_about(world.targets[t].view, world.targets[t].right,
                              -theta * world.targets[t].move_down);
        world.targets[t].view = v.normalize();
        world.targets[t].up = world.targets[t].view.cross(world.targets[t].right).normalize();
    }

    if world.targets[t].move_pitchright > 0.0 {
        let v = rotate_about(world.targets[t].up, world.targets[t].view,
                              -theta * world.targets[t].move_pitchright);
        world.targets[t].up = v.normalize();
        world.targets[t].right = world.targets[t].up.cross(world.targets[t].view).normalize();
    }

    if world.targets[t].move_pitchleft > 0.0 {
        let v = rotate_about(world.targets[t].up, world.targets[t].view,
                              theta * world.targets[t].move_pitchleft);
        world.targets[t].up = v.normalize();
        world.targets[t].right = world.targets[t].up.cross(world.targets[t].view).normalize();
    }

    if world.targets[t].move_forward > 0.0 {
        let v = world.targets[t].view * (world.targets[t].move_forward * DELTAV * dt);
        deltav = deltav + v;
    }

    if world.targets[t].move_backward > 0.0 {
        let v = world.targets[t].view * (-world.targets[t].move_backward * DELTAV * dt);
        deltav = deltav + v;
    }

    world.targets[t].vel = world.targets[t].vel + deltav;
    world.targets[t].pos = world.targets[t].pos + world.targets[t].vel * dt;

    // Reset all move flags (single-player: all flags cleared each frame)
    world.targets[t].move_forward = 0.0;
    world.targets[t].move_backward = 0.0;
    world.targets[t].move_up = 0.0;
    world.targets[t].move_down = 0.0;
    world.targets[t].move_pitchleft = 0.0;
    world.targets[t].move_pitchright = 0.0;
    world.targets[t].move_left = 0.0;
    world.targets[t].move_right = 0.0;

    // Regenerate shields
    let regen = world.targets[t].shieldregen;
    let max = world.targets[t].maxshields;
    world.targets[t].shields = (world.targets[t].shields + regen * dt).min(max);
}

/// Move all active targets. Port of MoveTargets() from target.c.
pub fn move_targets(world: &mut World) {
    let player_pos = world.player.pos;
    for t in 0..NTARGETS {
        if world.targets[t].age > 0.0 {
            let diff = world.targets[t].pos - player_pos;
            world.targets[t].range2 = diff.mag2();

            if !world.targets[t].hidden {
                let dt = world.delta_t;
                world.targets[t].age += dt;
                world.targets[t].msl_idle += dt;

                if world.targets[t].range2 < TARG_MAXRANGE2 {
                    crate::ai::think_target(t, world);
                }

                move_target(world, t);
            }
        }
    }
}


/// Advance planetary orbits. Port of MovePlanets() from planet.c.
pub fn move_planets(world: &mut World) {
    let dt = world.delta_t;
    for p in 1..NPLANETS {
        world.planets[p].theta -= world.planets[p].angvel * dt * world.compression
            * std::f64::consts::PI / 180.0;
        if world.planets[p].theta < 0.0 {
            world.planets[p].theta += 2.0 * std::f64::consts::PI;
        }
    }
    position_planets(world);
}

/// Recompute planet Cartesian positions from orbital angles.
/// Port of PositionPlanets() from planet.c.
pub fn position_planets(world: &mut World) {
    // First pass: position all non-moons
    for p in 0..NPLANETS {
        let th = world.planets[p].theta;
        let dist = world.planets[p].dist;
        world.planets[p].pos = Vec3::new(dist * th.sin(), dist * th.cos(), 0.0);
    }

    // Second pass: offset moons relative to their primary (with oblicity rotation)
    for p in 0..NPLANETS {
        if world.planets[p].is_moon {
            let pr = world.planets[p].primary;
            let primary_pos = world.planets[pr].pos;
            let oblicity_rad = -world.planets[pr].oblicity * std::f64::consts::PI / 180.0;
            let axis = Vec3::new(1.0, 0.0, 0.0);
            let rotated = rotate_about(world.planets[p].pos, axis, oblicity_rad);
            world.planets[p].pos = rotated + primary_pos;
        }
    }
}
