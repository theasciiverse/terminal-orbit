use crate::types::{Missile, World};

/// Fire a missile from the given position in the given direction.
/// `friendly` mirrors the owner's allegiance. `owner` is -1 for player,
/// >= 0 for a target index.
pub fn fire_missile(
    world: &mut World,
    pos: crate::math::Vec3,
    vel: crate::math::Vec3,
    dir: crate::math::Vec3,
    friendly: bool,
    wep: usize,
    owner: i32,
) {
    let m = find_free_missile(&world.missiles);
    let speed = world.weapons[wep].speed;
    world.missiles[m].age = world.delta_t.max(0.001);
    world.missiles[m].pos = pos;
    world.missiles[m].vel = dir * speed + vel;
    world.missiles[m].friendly = friendly;
    world.missiles[m].weapon = wep;
    world.missiles[m].owner = owner;
}

/// Find a free missile slot.  Returns the oldest slot if all are in use.
fn find_free_missile(missiles: &[Missile]) -> usize {
    let mut oldest = 0;
    let mut max_age = 0.0_f64;
    for (i, m) in missiles.iter().enumerate() {
        if m.age == 0.0 {
            return i;
        }
        if m.age > max_age {
            max_age = m.age;
            oldest = i;
        }
    }
    oldest
}
