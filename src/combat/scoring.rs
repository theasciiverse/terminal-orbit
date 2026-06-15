use crate::mission::events::fire_event;
use crate::types::{EventTrigger, World};
use crate::ui::console::console_add;

/// Destroy target `t`: zero its age, credit score, clear lock, fire Destroy events.
/// Port of DestroyTarget() from target.c.
pub fn destroy_target(world: &mut World, t: usize) {
    let score_delta = world.targets[t].score;
    let name = world.targets[t].name.clone();

    world.targets[t].age = 0.0;
    world.player.score += score_delta;
    console_add(world, format!("Destroyed: {} (+{})", name, score_delta));

    if world.lock.target == Some(t) {
        world.lock.target = None;
    }

    // Fire all pending Destroy events that match this target name.
    let n = world.events.len();
    let mut i = 0;
    while i < n {
        if world.events[i].pending
            && world.events[i].enabled
            && world.events[i].trigger == EventTrigger::Destroy
            && world.events[i].cvalue.eq_ignore_ascii_case(&name)
        {
            fire_event(world, i); // returns Option<String>; LoadMission here is ignored
                                  // (handled next tick via do_events loop if needed)
        }
        i += 1;
    }
}
