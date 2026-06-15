mod ai;
mod app;
mod autopilot;
mod combat;
mod constants;
mod hud;
mod input;
mod math;
mod mission;
mod model;
mod physics;
mod planet_data;
mod renderer;
mod config;
mod save;
mod scores;
mod star_data;
mod types;
mod ui;

fn main() {
    if let Err(e) = app::run() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use crate::math::Vec3;
    use crate::types::{Strategy, World};
    use crate::constants::NPLAYER_WEAPONS;
    use crate::planet_data::{init_weapons, reset_planets};
    use crate::combat::move_missiles;
    use crate::physics::move_targets;

    /// Phase 1 headless test: a Hunt1 target placed near the player must chase and
    /// fire at least once within 300 frames (10 seconds at 30fps).
    #[test]
    fn headless_hunt1_fires_within_300_frames() {
        let mut world = World::new();
        init_weapons(&mut world);
        reset_planets(&mut world);

        // Place player near Earth (planet[3])
        let earth_pos = world.planets[3].pos;
        world.player.pos = earth_pos + Vec3::new(-2.0 * world.planets[3].radius, 0.0, 0.0);

        // Spawn one Hunt1 enemy 0.1 units ahead of the player, already facing player
        let t = 0;
        world.targets[t].age = 0.1;
        world.targets[t].pos = world.player.pos + Vec3::new(0.1, 0.0, 0.0);
        world.targets[t].view = Vec3::new(-1.0, 0.0, 0.0); // facing player
        world.targets[t].up = Vec3::new(0.0, 0.0, 1.0);
        world.targets[t].right = world.targets[t].up.cross(world.targets[t].view).normalize();
        world.targets[t].strategy = Strategy::Hunt1;
        world.targets[t].friendly = false;
        world.targets[t].turnrate = 1.0;
        world.targets[t].maxvel = 1.0;
        world.targets[t].weapon = NPLAYER_WEAPONS; // "Spare" enemy weapon (idle=2.0s)
        world.targets[t].shields = 100.0;
        world.targets[t].maxshields = 100.0;

        world.delta_t = 1.0 / 30.0;

        for _ in 0..300 {
            move_targets(&mut world);
            move_missiles(&mut world);
        }

        assert!(
            world.shots_fired > 0,
            "Hunt1 target should have fired at least once in 300 frames"
        );
    }
}
