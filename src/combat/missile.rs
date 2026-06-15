use crate::constants::{NMSLS, TARGDIST2};
use crate::math::Vec3;
use crate::model::Model;
use crate::mission::events::fire_event;
use crate::types::{EventTrigger, Missile, Planet, Target, World};

use super::explosions::spawn_boom;
use super::scoring::destroy_target;

/// Full missile simulation: port of MoveMissiles() from missile.c.
/// Replaces the Phase-2 stub in physics.rs.
pub fn move_missiles(world: &mut World) {
    let dt = world.delta_t;

    for m in 0..NMSLS {
        if world.missiles[m].age <= 0.0 {
            continue;
        }

        world.missiles[m].age += dt;
        let wep = world.missiles[m].weapon;

        // Expired?
        if world.missiles[m].age > world.weapons[wep].expire {
            world.missiles[m].age = 0.0;
            continue;
        }

        // Snapshot position and velocity for checks below.
        let pos = world.missiles[m].pos;
        let vel = world.missiles[m].vel;
        let friendly = world.missiles[m].friendly;

        // Hit a planet?
        if let Some(p) = inside_planet(pos, &world.planets) {
            let p_pos  = world.planets[p].pos;
            let p_r    = world.planets[p].radius;
            let damage = world.weapons[wep].damage / 100.0;
            let dp     = (pos - p_pos).normalize();
            let impact = p_pos + dp * p_r * 1.05;
            spawn_boom(world, impact, damage);
            world.missiles[m].age = 0.0;
            continue;
        }

        // Hit a target?
        let hit = hit_target(pos, friendly, &world.missiles[m], &world.targets, &world.models);
        if let Some(t) = hit {
            missile_hit_target(world, m, t);
            continue;
        }

        // Hit player?
        if !friendly {
            let d2 = (pos - world.player.pos).mag2();
            if d2 < TARGDIST2 {
                missile_hit_player(world, m);
                continue;
            }
        }

        // Apply gravity and move.
        if world.gravity {
            let g = crate::physics::gravity(&world.planets, pos);
            world.missiles[m].vel = vel + g;
        }
        world.missiles[m].pos = pos + vel * dt;
    }
}

/// Returns `Some(planet_idx)` if `pos` is inside any planet's radius.
fn inside_planet(pos: Vec3, planets: &[Planet]) -> Option<usize> {
    for (i, p) in planets.iter().enumerate() {
        if p.hidden { continue; }
        let dp = pos - p.pos;
        if dp.mag2() < p.radius * p.radius {
            return Some(i);
        }
    }
    None
}

/// Returns the index of the first target hit by this missile (bounding-sphere test).
fn hit_target(
    pos: Vec3,
    msl_friendly: bool,
    _msl: &Missile,
    targets: &[Target],
    models: &[Model],
) -> Option<usize> {
    for (t, target) in targets.iter().enumerate() {
        if target.age <= 0.0 || target.hidden {
            continue;
        }
        // Missiles from friendly ships only hit unfriendly targets and vice-versa.
        if msl_friendly == target.friendly {
            continue;
        }
        let radius = target
            .model
            .and_then(|mi| models.get(mi))
            .map(|m| m.radius)
            .unwrap_or(0.05); // sensible default if model is missing

        let dp = pos - target.pos;
        if dp.mag2() < radius * radius {
            return Some(t);
        }
    }
    None
}

/// Handle a missile hitting a target: apply damage, check Shields events, destroy if dead.
fn missile_hit_target(world: &mut World, m: usize, t: usize) {
    let damage = world.weapons[world.missiles[m].weapon].damage;
    let missile_pos = world.missiles[m].pos;

    // Small explosion at impact point.
    spawn_boom(world, missile_pos, damage / 100.0);

    world.targets[t].shields -= damage;
    if world.targets[t].shields < 0.0 {
        world.targets[t].shields = 0.0;
    }

    // Check Shields trigger events.
    let target_name = world.targets[t].name.clone();
    let shield_level = world.targets[t].shields;
    let n_events = world.events.len();
    for e in 0..n_events {
        if world.events[e].pending
            && world.events[e].enabled
            && world.events[e].trigger == EventTrigger::Shields
            && world.events[e].cvalue.eq_ignore_ascii_case(&target_name)
            && shield_level <= world.events[e].fvalue
        {
            fire_event(world, e);
        }
    }

    // Destroyed?
    if world.targets[t].shields <= 0.0 {
        let target_pos = world.targets[t].pos;
        spawn_boom(world, target_pos, 1.0);
        destroy_target(world, t);
    }

    world.missiles[m].age = 0.0;
}

/// Handle a missile hitting the player.
fn missile_hit_player(world: &mut World, m: usize) {
    let damage = world.weapons[world.missiles[m].weapon].damage;
    world.player.shields -= damage;
    world.missiles[m].age = 0.0;

    if world.player.shields <= 0.0 {
        world.player.shields = 100.0;
        // Signal a mission restart.
        let msn = world.mission_file.clone();
        world.pending_mission = Some(msn);
        world.message.text = "You were killed! Restarting mission.".to_string();
        world.message.age = 0.0;
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::weapons::fire_missile;
    use crate::math::Vec3;
    use crate::planet_data::init_weapons;
    use crate::types::World;

    fn make_world() -> World {
        let mut w = World::new();
        init_weapons(&mut w);
        // Deliberately skip reset_planets: Sol sits at the origin with a large
        // radius, which would swallow missiles fired from (0,0,0).
        w
    }

    #[test]
    fn player_missile_hits_and_damages_target() {
        let mut world = make_world();

        // Spawn a target right in front of the player.
        let t = 0;
        world.targets[t].age = 0.1;
        world.targets[t].pos = world.player.pos + Vec3::new(0.05, 0.0, 0.0);
        world.targets[t].view = Vec3::new(-1.0, 0.0, 0.0);
        world.targets[t].up = Vec3::new(0.0, 0.0, 1.0);
        world.targets[t].right = Vec3::new(0.0, -1.0, 0.0);
        world.targets[t].friendly = false;
        world.targets[t].shields = 100.0;
        world.targets[t].maxshields = 100.0;

        // Player fires a fast missile directly forward.
        let (p_pos, p_view) = (world.player.pos, world.player.view);
        fire_missile(&mut world, p_pos, Vec3::zero(), p_view, true, 0, -1);

        // Advance until the missile travels to the target.
        for _ in 0..30 {
            move_missiles(&mut world);
        }

        assert!(
            world.targets[t].shields < 100.0 || world.targets[t].age == 0.0,
            "Target should have taken damage or been destroyed"
        );
    }

    #[test]
    fn target_missile_hits_player() {
        let mut world = make_world();

        // Place a target right behind the player (so its forward is toward player).
        world.player.pos = Vec3::zero();
        world.player.shields = 100.0;
        world.player.maxshields = 100.0;

        // Fire an enemy missile directly at the player from 0.02 units away.
        fire_missile(
            &mut world,
            Vec3::new(0.02, 0.0, 0.0),
            Vec3::zero(),
            Vec3::new(-1.0, 0.0, 0.0), // toward player at origin
            false, // enemy missile
            crate::constants::NPLAYER_WEAPONS, // enemy weapon
            0,     // target 0 is the owner
        );

        for _ in 0..30 {
            move_missiles(&mut world);
        }

        assert!(
            world.player.shields < 100.0,
            "Player should have taken damage, shields={}", world.player.shields
        );
    }

    #[test]
    fn score_increments_on_kill() {
        let mut world = make_world();

        // Target that rewards 1 point.
        world.targets[0].age = 0.1;
        world.targets[0].pos = world.player.pos + Vec3::new(0.05, 0.0, 0.0);
        world.targets[0].friendly = false;
        world.targets[0].shields = 1.0;  // very low shields
        world.targets[0].maxshields = 1.0;
        world.targets[0].score = 1;
        world.targets[0].view = Vec3::new(1.0, 0.0, 0.0);
        world.targets[0].up = Vec3::new(0.0, 0.0, 1.0);
        world.targets[0].right = Vec3::new(0.0, -1.0, 0.0);

        assert_eq!(world.player.score, 0);

        let p_pos = world.player.pos;
        fire_missile(&mut world, p_pos, Vec3::zero(), Vec3::new(1.0, 0.0, 0.0), true, 0, -1);

        for _ in 0..30 {
            move_missiles(&mut world);
        }

        assert_eq!(world.player.score, 1, "Score should be 1 after kill");
        assert_eq!(world.targets[0].age, 0.0, "Target should be destroyed");
    }
}
