use crate::constants::*;
use crate::math::Vec3;
use crate::types::*;

/// Dispatch AI for target `t`. Port of ThinkTarget() from think.c.
pub fn think_target(t: usize, world: &mut World) {
    match world.targets[t].strategy {
        Strategy::DoNothing => {}
        Strategy::Sit1  => sit1(t, world),
        Strategy::Sit2  => sit2(t, world),
        Strategy::Sit3  => sit3(t, world),
        Strategy::Sit4  => sit4(t, world),
        Strategy::Hunt1 => hunt1(t, world),
        Strategy::Hunt2 => hunt2(t, world),
        Strategy::Hunt3 => hunt3(t, world),
        Strategy::Hunt4 => hunt4(t, world),
    }
}

// --- Sit strategies (don't move, just turn & shoot) ---

fn sit1(t: usize, world: &mut World) {
    let pos = world.player.pos;
    turn_toward(t, pos, world);
}

fn sit2(t: usize, world: &mut World) {
    let wep_speed = world.weapons[world.targets[t].weapon].speed;
    let tgt_pos = world.targets[t].pos;
    let tgt_vel = world.targets[t].vel;
    let player_pos = world.player.pos;
    let player_vel = world.player.vel;

    match aim(tgt_pos, tgt_vel, player_pos, player_vel, wep_speed) {
        Some(intercept) => turn_toward(t, intercept, world),
        None => turn_toward(t, player_pos, world),
    }
}

fn sit3(t: usize, world: &mut World) {
    if let Some((pos, _vel)) = find_enemy(t, world) {
        turn_toward(t, pos, world);
    }
}

fn sit4(t: usize, world: &mut World) {
    let wep_speed = world.weapons[world.targets[t].weapon].speed;
    let tgt_pos = world.targets[t].pos;
    let tgt_vel = world.targets[t].vel;

    if let Some((enemy_pos, enemy_vel)) = find_enemy(t, world) {
        match aim(tgt_pos, tgt_vel, enemy_pos, enemy_vel, wep_speed) {
            Some(intercept) => turn_toward(t, intercept, world),
            None => turn_toward(t, enemy_pos, world),
        }
    }
}

// --- Hunt strategies (move toward target and shoot) ---

fn hunt1(t: usize, world: &mut World) {
    let pos = world.player.pos;
    move_toward(t, pos, world);
}

fn hunt2(t: usize, world: &mut World) {
    let wep_speed = world.weapons[world.targets[t].weapon].speed;
    let tgt_pos = world.targets[t].pos;
    let tgt_vel = world.targets[t].vel;
    let player_pos = world.player.pos;
    let player_vel = world.player.vel;

    match aim(tgt_pos, tgt_vel, player_pos, player_vel, wep_speed) {
        Some(intercept) => move_toward(t, intercept, world),
        None => hunt1(t, world),
    }
}

fn hunt3(t: usize, world: &mut World) {
    if let Some((pos, _vel)) = find_enemy(t, world) {
        move_toward(t, pos, world);
    }
}

fn hunt4(t: usize, world: &mut World) {
    let wep_speed = world.weapons[world.targets[t].weapon].speed;
    let tgt_pos = world.targets[t].pos;
    let tgt_vel = world.targets[t].vel;

    if let Some((enemy_pos, enemy_vel)) = find_enemy(t, world) {
        match aim(tgt_pos, tgt_vel, enemy_pos, enemy_vel, wep_speed) {
            Some(intercept) => move_toward(t, intercept, world),
            None => move_toward(t, enemy_pos, world),
        }
    }
}

// --- Core helpers ---

/// Turn target `t` toward `pos` and fire if facing and in range.
/// Port of TurnToward() from think.c.
pub fn turn_toward(t: usize, pos: Vec3, world: &mut World) {
    let v = pos - world.targets[t].pos;
    let r = v.mag2();

    if r > THINK_CUTOFFA2 {
        world.targets[t].vel = Vec3::default();
        return;
    }

    let v = v.normalize();
    let alpha = v.dot(world.targets[t].right);
    let beta  = v.dot(world.targets[t].up);
    let theta = v.dot(world.targets[t].view);
    let turnrate = world.targets[t].turnrate;
    let wep = world.targets[t].weapon;
    let weapon_range2 = world.weapons[wep].range2;

    if alpha > 0.0 {
        world.targets[t].move_left = turnrate;
    } else {
        world.targets[t].move_right = turnrate;
    }

    if beta > 0.0 {
        world.targets[t].move_up = turnrate;
    } else {
        world.targets[t].move_down = turnrate;
    }

    if theta > 0.9 && r < weapon_range2 {
        target_fires_missile(t, world);
    }
}

/// Move target `t` toward `pos`, adjusting speed by distance bands.
/// Port of MoveToward() from think.c.
pub fn move_toward(t: usize, pos: Vec3, world: &mut World) {
    let v = pos - world.targets[t].pos;
    let r = v.mag2();

    if r > THINK_CUTOFFA2 {
        world.targets[t].vel = Vec3::default();
        return;
    }

    let v = v.normalize();
    let alpha = v.dot(world.targets[t].right);
    let beta  = v.dot(world.targets[t].up);
    let theta = v.dot(world.targets[t].view);
    let turnrate = world.targets[t].turnrate;
    let maxvel = world.targets[t].maxvel;
    let wep = world.targets[t].weapon;
    let weapon_range2 = world.weapons[wep].range2;

    if alpha > 0.0 {
        world.targets[t].move_left = turnrate;
    } else {
        world.targets[t].move_right = turnrate;
    }

    if beta > 0.0 {
        world.targets[t].move_up = turnrate;
    } else {
        world.targets[t].move_down = turnrate;
    }

    if r > THINK_CUTOFFB2 {
        if theta > 0.0 {
            world.targets[t].move_forward = maxvel;
        } else {
            world.targets[t].move_backward = maxvel;
        }
    }

    if r < THINK_CUTOFFC2 {
        world.targets[t].vel = Vec3::default();
    }

    if theta > 0.9 && r < weapon_range2 {
        target_fires_missile(t, world);
    }
}

/// Find the closest enemy to target `targ`.
/// Returns `Some((pos, vel))` of the closest threat (enemy target or player).
/// Port of FindEnemy() from think.c.
pub fn find_enemy(targ: usize, world: &World) -> Option<(Vec3, Vec3)> {
    let targ_friendly = world.targets[targ].friendly;
    let targ_pos = world.targets[targ].pos;
    let targ_range2 = world.targets[targ].range2;

    let mut best_dist: f64 = -1.0;
    let mut best_pos = Vec3::default();
    let mut best_vel = Vec3::default();

    for t in 0..NTARGETS {
        if world.targets[t].age > 0.0
            && !world.targets[t].hidden
            && t != targ
            && world.targets[t].friendly != targ_friendly
        {
            let r = (world.targets[t].pos - targ_pos).mag2();
            if best_dist < 0.0 || r < best_dist {
                best_dist = r;
                best_pos = world.targets[t].pos;
                best_vel = world.targets[t].vel;
            }
        }
    }

    // Non-friendly targets also consider the player
    if !targ_friendly && (best_dist < 0.0 || targ_range2 < best_dist) {
        best_dist = targ_range2;
        best_pos = world.player.pos;
        best_vel = world.player.vel;
    }

    if best_dist < 0.0 {
        None
    } else {
        Some((best_pos, best_vel))
    }
}

/// Compute intercept point for a shot with speed `vel` fired from `pos0/vel0`
/// at a target at `pos1/vel1`. Returns `Some(world_pos)` or `None` if no solution.
/// Port of Aim() from hud.c.
pub fn aim(
    pos0: Vec3, vel0: Vec3,
    pos1: Vec3, vel1: Vec3,
    vel: f64,
) -> Option<Vec3> {
    let va = pos1 - pos0;      // target position relative to shooter
    let vb = vel1 - vel0;      // relative velocity

    let a = vb.mag2() - vel * vel;
    let b = 2.0 * va.dot(vb);
    let c = va.mag2();

    let t = if a == 0.0 {
        if b == 0.0 {
            return None;
        }
        let t = -c / b;
        if t < 0.0 { return None; }
        t
    } else {
        let d = b * b - 4.0 * a * c;
        if d < 0.0 {
            return None;
        }
        if d == 0.0 {
            let t = -b / (2.0 * a);
            if t <= 0.0 { return None; }
            t
        } else {
            let root_d = d.sqrt();
            let t1 = (-b + root_d) / (2.0 * a);
            let t2 = (-b - root_d) / (2.0 * a);
            match (t1 >= 0.0, t2 >= 0.0) {
                (false, false) => return None,
                (true,  false) => t1,
                (false, true)  => t2,
                (true,  true)  => t1.min(t2),
            }
        }
    };

    // Intercept world position: pos1 + vel1*t
    Some(pos1 + vel1 * t)
}

/// Target `t` attempts to fire. Port of TargetFiresMissile() from target.c.
pub fn target_fires_missile(t: usize, world: &mut World) {
    let wep = world.targets[t].weapon;
    if world.targets[t].msl_idle > world.weapons[wep].idle {
        world.targets[t].msl_idle = 0.0;
        world.shots_fired += 1;

        let pos = world.targets[t].pos;
        let vel = world.targets[t].vel;
        let dir = world.targets[t].view;
        let friendly = world.targets[t].friendly;
        crate::combat::fire_missile(world, pos, vel, dir, friendly, wep, t as i32);
    }
}
