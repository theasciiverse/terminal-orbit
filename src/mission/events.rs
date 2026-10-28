use crate::constants::KM_TO_UNITS1;
use crate::types::{EventActionKind, EventTrigger, World};

/// Process all pending, enabled events each game tick.
/// Returns `Some(filename)` if a `LoadMission` action fired and the
/// caller should reload the world from that file.
pub fn do_events(world: &mut World) -> Option<String> {
    let dt = world.delta_t;
    let n = world.events.len();
    for e in 0..n {
        if !world.events[e].pending || !world.events[e].enabled {
            continue;
        }

        let triggered = match world.events[e].trigger {
            EventTrigger::Null => false,
            EventTrigger::True => true,
            EventTrigger::Alarm => {
                world.events[e].fvalue -= dt;
                world.events[e].fvalue <= 0.0
            }
            EventTrigger::Score => world.player.score >= world.events[e].ivalue,
            EventTrigger::Approach => {
                let r2 = (world.events[e].pos - world.player.pos).mag2();
                let threshold = world.events[e].fvalue;
                r2 <= threshold * threshold
            }
            EventTrigger::Depart => {
                let r2 = (world.events[e].pos - world.player.pos).mag2();
                let threshold = world.events[e].fvalue;
                r2 > threshold * threshold
            }
            EventTrigger::StopNear => {
                if world.player.vel.mag2() > 0.0 {
                    false
                } else {
                    let r2 = (world.events[e].pos - world.player.pos).mag2();
                    let threshold = world.events[e].fvalue;
                    r2 <= threshold * threshold
                }
            }
            EventTrigger::Reach => {
                let name = &world.events[e].cvalue;
                let pos = world.events[e].pos;
                let r = world.events[e].fvalue;
                world.targets.iter().any(|t| {
                    t.age > 0.0 && t.name.eq_ignore_ascii_case(name) && (t.pos - pos).mag2() <= r * r
                })
            }
            // Destroy and Shields are checked externally (in combat code).
            EventTrigger::Destroy | EventTrigger::Shields => false,
        };

        if triggered {
            if let Some(next_msn) = fire_event(world, e) {
                return Some(next_msn);
            }
        }
    }
    None
}

/// Execute all actions for event `e`.  Returns `Some(filename)` on LoadMission.
pub fn fire_event(world: &mut World, e: usize) -> Option<String> {
    world.events[e].pending = false;

    let actions = world.events[e].actions.clone();
    for action in &actions {
        if !action.active {
            continue;
        }
        match action.kind {
            EventActionKind::None => {}

            EventActionKind::Message => {
                let text = action.cvalue.replace("\\\\", "\n");
                world.message.last_text = text.clone();
                world.message.text = text;
                world.message.age = 0.0;
            }

            EventActionKind::Hide => {
                if let Some(t) = find_target_by_name(world, &action.cvalue) {
                    world.targets[t].hidden = true;
                }
            }

            EventActionKind::Unhide => {
                if let Some(t) = find_target_by_name(world, &action.cvalue) {
                    world.targets[t].hidden = false;
                }
            }

            EventActionKind::Enable => {
                if let Some(ev) = find_event_by_name(world, &action.cvalue) {
                    world.events[ev].enabled = true;
                }
            }

            EventActionKind::Disable => {
                if let Some(ev) = find_event_by_name(world, &action.cvalue) {
                    world.events[ev].enabled = false;
                }
            }

            EventActionKind::Stop => {
                world.player.vel = crate::math::Vec3::zero();
            }

            EventActionKind::LoadMission => {
                return Some(action.cvalue.clone());
            }

            EventActionKind::MoveObject => {
                let pos = world.events[e].pos;
                if let Some(t) = find_target_by_name(world, &action.cvalue) {
                    world.targets[t].pos = pos;
                }
            }

            EventActionKind::MovePlayer => {
                world.player.pos = world.events[e].pos;
            }

            EventActionKind::MovePlanet => {
                let pos = world.events[e].pos;
                if let Some(p) = find_planet_by_name(world, &action.cvalue) {
                    world.planets[p].pos = pos;
                }
            }

            EventActionKind::HidePlanet => {
                if let Some(p) = find_planet_by_name(world, &action.cvalue) {
                    world.planets[p].hidden = true;
                }
            }

            EventActionKind::UnhidePlanet => {
                if let Some(p) = find_planet_by_name(world, &action.cvalue) {
                    world.planets[p].hidden = false;
                }
            }

            EventActionKind::Betray => {
                if let Some(t) = find_target_by_name(world, &action.cvalue) {
                    world.targets[t].friendly = !world.targets[t].friendly;
                }
            }

            EventActionKind::Boom => {
                let pos = world.events[e].pos;
                let size = action.fvalue;
                crate::combat::spawn_boom(world, pos, size);
            }

            EventActionKind::Score => {
                world.player.score += action.ivalue;
            }

            EventActionKind::Destroy => {
                if let Some(t) = find_target_by_name(world, &action.cvalue) {
                    let pos = world.targets[t].pos;
                    crate::combat::spawn_boom(world, pos, 1.0);
                    crate::combat::destroy_target(world, t);
                }
            }

            EventActionKind::Cargo => {
                world.cargo = if action.cvalue.eq_ignore_ascii_case("none") {
                    String::new()
                } else {
                    action.cvalue.clone()
                };
                if !world.cargo.is_empty() {
                    let line = format!("Cargo aboard: {}", world.cargo);
                    crate::ui::console::console_add(world, line);
                }
            }

            EventActionKind::Waypoint => {
                world.player.waypoint = action.ivalue.max(0) as usize;
            }

            EventActionKind::Flash => {
                // Visual flash: no-op for now (Phase 5 adds explosions).
            }
        }
    }
    None
}

fn find_target_by_name(world: &World, name: &str) -> Option<usize> {
    world.targets.iter().position(|t| t.age > 0.0 && t.name.eq_ignore_ascii_case(name))
}

fn find_event_by_name(world: &World, name: &str) -> Option<usize> {
    world.events.iter().position(|e| e.name.eq_ignore_ascii_case(name))
}

fn find_planet_by_name(world: &World, name: &str) -> Option<usize> {
    world.planets.iter().position(|p| p.name.eq_ignore_ascii_case(name))
}


/// Convert Approach/Depart fvalue from km to game units.
/// Called by the loader after parsing each event block.
pub fn convert_approach_units(world: &mut World, e: usize) {
    use crate::types::EventTrigger;
    match world.events[e].trigger {
        EventTrigger::Approach | EventTrigger::Depart | EventTrigger::StopNear
        | EventTrigger::Reach => {
            world.events[e].fvalue /= KM_TO_UNITS1;
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::Vec3;
    use crate::mission::loader::load_mission;
    use crate::physics::{move_player, move_targets};
    use std::path::Path;

    fn load(name: &str) -> World {
        let mut w = World::new();
        crate::planet_data::init_weapons(&mut w);
        crate::planet_data::reset_planets(&mut w);
        load_mission(&mut w, name, Path::new("missions"), Path::new("models")).unwrap();
        w
    }

    fn idx(world: &World, name: &str) -> usize {
        world.targets.iter()
            .position(|t| t.age > 0.0 && t.name.eq_ignore_ascii_case(name))
            .unwrap()
    }

    fn event(world: &World, name: &str) -> usize {
        world.events.iter().position(|e| e.name.eq_ignore_ascii_case(name)).unwrap()
    }

    /// Run the game tick (player, targets, events) for `secs` seconds.
    fn run(world: &mut World, secs: f64) -> Option<String> {
        let steps = (secs / world.delta_t) as usize;
        for _ in 0..steps {
            move_player(world);
            move_targets(world);
            if let Some(next) = do_events(world) {
                return Some(next);
            }
        }
        None
    }

    #[test]
    fn score_action_adds_to_player_score() {
        let mut w = load("ep2_01.msn");
        let e = event(&w, "release");
        fire_event(&mut w, e);
        assert_eq!(w.player.score, 1);
    }

    #[test]
    fn destroy_action_really_destroys() {
        let mut w = load("ep2_01.msn");
        w.events[0].actions = vec![crate::types::EventAction {
            active: true,
            kind: EventActionKind::Destroy,
            cvalue: "Hauler1".into(),
            ..Default::default()
        }];
        fire_event(&mut w, 0);
        assert!(w.targets.iter().all(|t| !(t.age > 0.0 && t.name == "Hauler1")));
    }

    #[test]
    fn hold_makes_stopnear_docking_possible_even_with_gravity() {
        let mut w = load("ep2_01.msn");
        w.gravity = true;
        let base = w.targets[idx(&w, "EarthBase")].pos;
        w.player.pos = base + Vec3::new(0.1, 0.0, 0.0); // 600 km out
        w.player.vel = Vec3::new(0.05, 0.0, 0.0);
        run(&mut w, 1.0);
        assert!(w.cargo.is_empty(), "drifting past must not count as docked");

        w.player.pos = base + Vec3::new(0.1, 0.0, 0.0);
        w.full_stop = true; // H
        run(&mut w, 0.5);
        assert_eq!(w.cargo, "Lunar relay satellite");
        assert_eq!(w.player.waypoint, 1);

        // Thrust releases the hold.
        w.player.move_forward = 1.0;
        move_player(&mut w);
        assert!(!w.full_stop);
    }

    #[test]
    fn ferry_shuttles_between_stops() {
        let mut w = load("ep2_01.msn");
        let h = idx(&w, "Hauler1");
        let first = w.targets[h].route[0].clone();
        let dest = w.targets[idx(&w, &first)].pos;
        let start = (w.targets[h].pos - dest).mag2().sqrt();
        // Route traffic flies even with the player far away.
        w.player.pos = Vec3::new(1e6, 0.0, 0.0);
        let mut reached = false;
        for _ in 0..(600.0 / w.delta_t) as usize {
            move_targets(&mut w);
            if w.targets[h].route_idx == 1 {
                reached = true;
                break;
            }
        }
        assert!(reached, "Hauler1 never reached {} ({:.1}u away)", first, start);
    }

    #[test]
    fn convoy_arrival_fires_reach_trigger() {
        let mut w = load("ep2_03.msn");
        let l = idx(&w, "Lantern");
        // Skip the trip: place Lantern on its last leg near the station.
        let station = w.targets[idx(&w, "EuropaStation")].pos;
        w.targets[l].route_idx = 1;
        w.targets[l].pos = station + Vec3::new(0.5, 0.0, 0.0);
        w.player.pos = station + Vec3::new(0.0, 2.0, 0.0);
        // Remove the raiders so the test is about arriving.
        for name in ["Raider1", "Raider2", "Raider3", "Raider4", "RaiderBoss"] {
            let r = idx(&w, name);
            w.targets[r].age = 0.0;
        }
        run(&mut w, 30.0);
        assert!(!w.events[event(&w, "arrived")].pending, "arrival never fired");
        assert!(w.events[event(&w, "next")].enabled);
        assert!(w.targets[l].vel.mag2() < 1e-12, "convoy should hold at its last stop");
    }

    #[test]
    fn salvagers_grabbing_the_container_fails_the_mission() {
        let mut w = load("ep2_02.msn");
        let m = idx(&w, "Magpie");
        let os = w.targets[idx(&w, "SampleOS")].pos;
        w.targets[m].pos = os + Vec3::new(0.05, 0.0, 0.0);
        run(&mut w, 0.2);
        assert!(w.events[event(&w, "retry")].enabled);
        assert!(!w.events[event(&w, "catch")].enabled);
    }
}
