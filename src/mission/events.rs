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

/// Notify all Destroy-triggered events that target `t` was destroyed.
pub fn notify_destroy(world: &mut World, target_name: &str) {
    let n = world.events.len();
    for e in 0..n {
        if world.events[e].pending
            && world.events[e].enabled
            && world.events[e].trigger == EventTrigger::Destroy
            && world.events[e].cvalue.eq_ignore_ascii_case(target_name)
        {
            fire_event(world, e);
        }
    }
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

            EventActionKind::Flash => {
                // Visual flash — no-op for now (Phase 5 adds explosions).
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
        EventTrigger::Approach | EventTrigger::Depart | EventTrigger::StopNear => {
            world.events[e].fvalue /= KM_TO_UNITS1;
        }
        _ => {}
    }
}
