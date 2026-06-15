use std::path::Path;

use crate::constants::KM_TO_UNITS1;
use crate::math::Vec3;
use crate::model::{ac3d_loader, tri_loader, Model};
use crate::planet_data::{init_weapons, reset_planets};
use crate::types::{
    EventAction, EventActionKind, EventTrigger, GameEvent, Strategy, Target, Waypoint, World,
};

use super::events::convert_approach_units;
use super::parser::Parser;

/// Load a mission file into `world`, resetting all mutable game state first.
/// `missions_dir` is the directory where .msn and .inc files live.
/// `models_dir` is the directory where model files live.
pub fn load_mission(
    world: &mut World,
    filename: &str,
    missions_dir: &Path,
    models_dir: &Path,
) -> Result<(), String> {
    reset_world(world);

    world.mission_file = filename.to_string();
    world.mission_verbose = false;

    let mut parser = Parser::new(missions_dir);
    let path = missions_dir.join(filename);
    parser.push_file(&path)?;

    // Cursor starts at origin.
    let mut cursor = Vec3::zero();

    while let Some(token) = parser.next() {
        let tok = token.to_lowercase();
        match tok.as_str() {
            "cursor"  => handle_cursor(&mut cursor, &mut parser, world)?,
            "player"  => handle_player(&cursor, &mut parser, world)?,
            "waypoint"=> handle_waypoint(&cursor, &mut parser, world)?,
            "object"  => handle_object(&cursor, &mut parser, world, models_dir)?,
            "briefing"=> handle_briefing(&mut parser, world)?,
            "event"   => handle_event(&cursor, &mut parser, world)?,
            "planet"  => handle_planet(&cursor, &mut parser, world)?,
            "weapon"  => handle_weapon(&mut parser, world)?,
            "include" => handle_include(&mut parser)?,
            "verbose" => world.mission_verbose = true,
            "terse"   => world.mission_verbose = false,
            _         => { /* skip unrecognized */ }
        }
    }

    Ok(())
}

// ─── Reset helpers ────────────────────────────────────────────────────────────

fn reset_world(world: &mut World) {
    for t in &mut world.targets {
        *t = Target::default();
        t.weapon = crate::constants::NPLAYER_WEAPONS;
        t.shields = 100.0;
        t.maxshields = 100.0;
        t.view = Vec3::new(1.0, 0.0, 0.0);
        t.up = Vec3::new(0.0, 0.0, 1.0);
        t.right = t.up.cross(t.view).normalize();
    }
    for e in &mut world.events {
        *e = GameEvent::default();
        e.enabled = true;
    }
    for w in &mut world.waypoints {
        *w = Waypoint::default();
    }
    world.nwaypoints = 0;
    world.player.score = 0;
    world.player.vel = Vec3::zero();
    world.briefing.clear();
    world.lock.target = None;

    init_weapons(world);
    reset_planets(world);
}

// ─── Cursor ───────────────────────────────────────────────────────────────────

fn handle_cursor(
    cursor: &mut Vec3,
    parser: &mut Parser,
    world: &World,
) -> Result<(), String> {
    parser.require_brace()?;
    let mut axis = 0usize;

    loop {
        let tok = parser.require()?;
        if tok == "}" {
            return Ok(());
        }

        // Is it a number (possibly with leading sign)?
        let first = tok.chars().next().unwrap_or(' ');
        if first == '+' || first == '-' || first.is_ascii_digit() {
            let relative = first == '+' || first == '-';
            let v: f64 = tok.parse().map_err(|_| format!("Bad number: {}", tok))?;
            let v = v / KM_TO_UNITS1;
            if relative {
                match axis {
                    0 => cursor.x += v,
                    1 => cursor.y += v,
                    _ => cursor.z += v,
                }
            } else {
                match axis {
                    0 => cursor.x = v,
                    1 => cursor.y = v,
                    _ => cursor.z = v,
                }
            }
            axis = (axis + 1) % 3;
        } else {
            // Planet or object name — set cursor to its position.
            if let Some(p) = world.planets.iter().position(|p| p.name.eq_ignore_ascii_case(&tok)) {
                *cursor = world.planets[p].pos;
                axis = 0;
            } else if let Some(t) = world
                .targets
                .iter()
                .position(|t| t.age > 0.0 && t.name.eq_ignore_ascii_case(&tok))
            {
                *cursor = world.targets[t].pos;
                axis = 0;
            }
            // Unknown name — cursor unchanged (matches C behaviour).
        }
    }
}

// ─── Player ───────────────────────────────────────────────────────────────────

fn handle_player(
    cursor: &Vec3,
    parser: &mut Parser,
    world: &mut World,
) -> Result<(), String> {
    parser.require_brace()?;
    world.player.pos = *cursor;
    world.player.vel = Vec3::zero();
    skip_to_close(parser)
}

// ─── Waypoint ─────────────────────────────────────────────────────────────────

fn handle_waypoint(
    cursor: &Vec3,
    parser: &mut Parser,
    world: &mut World,
) -> Result<(), String> {
    parser.require_brace()?;
    if world.nwaypoints < world.waypoints.len() {
        world.waypoints[world.nwaypoints].pos = *cursor;
        world.nwaypoints += 1;
    }
    skip_to_close(parser)
}

// ─── Object ───────────────────────────────────────────────────────────────────

fn handle_object(
    cursor: &Vec3,
    parser: &mut Parser,
    world: &mut World,
    models_dir: &Path,
) -> Result<(), String> {
    parser.require_brace()?;

    // Find a free target slot.
    let t = match world.targets.iter().position(|t| t.age <= 0.0) {
        Some(i) => i,
        None => return Err("Out of target slots".to_string()),
    };

    world.targets[t].pos = *cursor;
    world.targets[t].age = 0.1;
    world.targets[t].view = Vec3::new(1.0, 0.0, 0.0);
    world.targets[t].up = Vec3::new(0.0, 0.0, 1.0);
    world.targets[t].right = world.targets[t].up.cross(world.targets[t].view).normalize();
    world.targets[t].strategy = Strategy::DoNothing;
    world.targets[t].friendly = false;
    world.targets[t].weapon = crate::constants::NPLAYER_WEAPONS;
    world.targets[t].maxshields = 100.0;
    world.targets[t].shields = 100.0;
    world.targets[t].shieldregen = crate::constants::SHIELD_REGEN;
    world.targets[t].turnrate = 0.3;
    world.targets[t].maxvel = 0.01;
    world.targets[t].hidden = false;
    world.targets[t].invisible = false;
    world.targets[t].name = "Target".to_string();
    // default model is first in world.models (index 0)
    world.targets[t].model = if world.models.is_empty() { None } else { Some(0) };

    loop {
        let tok = parser.require()?;
        match tok.to_lowercase().as_str() {
            "}" => return Ok(()),
            "name" => {
                world.targets[t].name = parser.require()?;
            }
            "model" => {
                let mname = parser.require()?;
                let idx = find_or_load_model(world, &mname, models_dir);
                world.targets[t].model = idx;
            }
            "score" => {
                let v = parser.require()?;
                world.targets[t].score = v.parse().unwrap_or(0);
            }
            "strategy" => {
                let s = parser.require()?;
                world.targets[t].strategy = parse_strategy(&s);
            }
            "hidden" => world.targets[t].hidden = true,
            "invisible" => world.targets[t].invisible = true,
            "friendly" => world.targets[t].friendly = true,
            "weapon" => {
                let v = parser.require()?;
                let w: usize = v.parse().unwrap_or(crate::constants::NPLAYER_WEAPONS);
                world.targets[t].weapon = w.min(crate::constants::NWEAPONS - 1);
            }
            "maxshields" => {
                let v: f64 = parser.require()?.parse().unwrap_or(100.0);
                world.targets[t].maxshields = v;
                world.targets[t].shields = v;
            }
            "shieldregen" => {
                let v: f64 = parser.require()?.parse().unwrap_or(crate::constants::SHIELD_REGEN);
                world.targets[t].shieldregen = v;
            }
            "turnrate" => {
                let v: f64 = parser.require()?.parse().unwrap_or(0.3);
                world.targets[t].turnrate = v;
            }
            "speed" => {
                let v: f64 = parser.require()?.parse().unwrap_or(0.01);
                world.targets[t].maxvel = v;
            }
            _ => { /* skip */ }
        }
    }
}

// ─── Briefing ─────────────────────────────────────────────────────────────────

fn handle_briefing(parser: &mut Parser, world: &mut World) -> Result<(), String> {
    parser.require_brace()?;
    let mut words: Vec<String> = Vec::new();
    loop {
        let tok = parser.require()?;
        if tok == "}" {
            break;
        }
        words.push(tok);
    }
    world.briefing = words.join(" ");
    Ok(())
}

// ─── Event ────────────────────────────────────────────────────────────────────

fn handle_event(
    cursor: &Vec3,
    parser: &mut Parser,
    world: &mut World,
) -> Result<(), String> {
    parser.require_brace()?;

    let e = match world.events.iter().position(|e| !e.pending) {
        Some(i) => i,
        None => return Err("Out of event slots".to_string()),
    };

    world.events[e] = GameEvent::default();
    world.events[e].pending = true;
    world.events[e].enabled = true;
    world.events[e].pos = *cursor;

    let mut iact: i32 = -1;
    let mut ta_is_action = false; // false=trigger, true=action

    loop {
        let tok = parser.require()?;
        match tok.to_lowercase().as_str() {
            "}" => {
                convert_approach_units(world, e);
                return Ok(());
            }
            "name" => {
                world.events[e].name = parser.require()?;
            }
            "trigger" => {
                ta_is_action = false;
                let s = parser.require()?;
                world.events[e].trigger = parse_trigger(&s);
            }
            "action" => {
                iact += 1;
                ta_is_action = true;
                let s = parser.require()?;
                let kind = parse_action_kind(&s);
                world.events[e].actions.push(EventAction {
                    active: true,
                    kind,
                    ivalue: 0,
                    fvalue: 0.0,
                    cvalue: String::new(),
                });
            }
            "enabled" => world.events[e].enabled = true,
            "disabled" => world.events[e].enabled = false,
            "value" => {
                let val = read_value(parser)?;
                if !ta_is_action {
                    // Trigger value
                    world.events[e].cvalue = val.clone();
                    world.events[e].ivalue = val.parse().unwrap_or(0);
                    world.events[e].fvalue = val.parse().unwrap_or(0.0);
                } else if iact >= 0 {
                    let ia = iact as usize;
                    if ia < world.events[e].actions.len() {
                        world.events[e].actions[ia].cvalue = val.clone();
                        world.events[e].actions[ia].ivalue = val.parse().unwrap_or(0);
                        world.events[e].actions[ia].fvalue = val.parse().unwrap_or(0.0);
                    }
                }
            }
            _ => { /* skip */ }
        }
    }
}

/// Read a value token: either a bare token or `{ ... }` block joined with spaces.
fn read_value(parser: &mut Parser) -> Result<String, String> {
    let tok = parser.require()?;
    if tok != "{" {
        return Ok(tok);
    }
    let mut words: Vec<String> = Vec::new();
    loop {
        let t = parser.require()?;
        if t == "}" {
            break;
        }
        words.push(t);
    }
    Ok(words.join(" "))
}

// ─── Planet ───────────────────────────────────────────────────────────────────

fn handle_planet(
    cursor: &Vec3,
    parser: &mut Parser,
    world: &mut World,
) -> Result<(), String> {
    parser.require_brace()?;
    let mut planet_idx: Option<usize> = None;

    loop {
        let tok = parser.require()?;
        match tok.to_lowercase().as_str() {
            "}" => return Ok(()),
            "name" => {
                let name = parser.require()?;
                planet_idx = world.planets.iter().position(|p| p.name.eq_ignore_ascii_case(&name));
            }
            "newname" => {
                let new = parser.require()?;
                if let Some(p) = planet_idx {
                    world.planets[p].name = new;
                }
            }
            "reposition" => {
                if let Some(p) = planet_idx {
                    world.planets[p].pos = *cursor;
                }
            }
            "hidden" => {
                if let Some(p) = planet_idx {
                    world.planets[p].hidden = true;
                }
            }
            "oblicity" => {
                let v: f64 = parser.require()?.parse().unwrap_or(0.0);
                if let Some(p) = planet_idx {
                    world.planets[p].oblicity = v;
                }
            }
            "radius" => {
                let v: f64 = parser.require()?.parse().unwrap_or(0.0);
                if let Some(p) = planet_idx {
                    let r = v / KM_TO_UNITS1;
                    world.planets[p].radius = r;
                    world.planets[p].radius2 = r * r;
                    world.planets[p].mass = r * r * r;
                }
            }
            "map" => {
                // Texture maps not used in terminal version.
                let _ = parser.require()?;
            }
            _ => { /* skip */ }
        }
    }
}

// ─── Weapon ───────────────────────────────────────────────────────────────────

fn handle_weapon(parser: &mut Parser, world: &mut World) -> Result<(), String> {
    parser.require_brace()?;
    let mut weapon_idx: Option<usize> = None;

    loop {
        let tok = parser.require()?;
        match tok.to_lowercase().as_str() {
            "}" => {
                // Recompute range² for this weapon.
                if let Some(w) = weapon_idx {
                    let spd = world.weapons[w].speed;
                    let exp = world.weapons[w].expire;
                    let range = spd * exp;
                    world.weapons[w].range2 = range * range;
                }
                return Ok(());
            }
            "index" => {
                let v: usize = parser.require()?.parse().unwrap_or(0);
                if v < world.weapons.len() {
                    weapon_idx = Some(v);
                }
            }
            "name" => {
                let name = parser.require()?;
                if let Some(w) = weapon_idx {
                    world.weapons[w].name = name;
                }
            }
            "speed" => {
                let v: f64 = parser.require()?.parse().unwrap_or(0.0);
                if let Some(w) = weapon_idx {
                    world.weapons[w].speed = v / KM_TO_UNITS1;
                }
            }
            "yield" => {
                let v: f64 = parser.require()?.parse().unwrap_or(0.0);
                if let Some(w) = weapon_idx {
                    world.weapons[w].damage = v;
                }
            }
            "idle" => {
                let v: f64 = parser.require()?.parse().unwrap_or(0.0);
                if let Some(w) = weapon_idx {
                    world.weapons[w].idle = v;
                }
            }
            "expire" => {
                let v: f64 = parser.require()?.parse().unwrap_or(0.0);
                if let Some(w) = weapon_idx {
                    world.weapons[w].expire = v;
                }
            }
            "renderer" => {
                let v: u8 = parser.require()?.parse().unwrap_or(0);
                if let Some(w) = weapon_idx {
                    world.weapons[w].renderer = v;
                }
            }
            "color" => {
                let s = parser.require()?;
                if let Some(w) = weapon_idx {
                    let c: u32 = u32::from_str_radix(s.trim_start_matches("0x"), 16).unwrap_or(0);
                    world.weapons[w].color = [
                        ((c >> 16) & 0xff) as f32 / 256.0,
                        ((c >> 8) & 0xff) as f32 / 256.0,
                        (c & 0xff) as f32 / 256.0,
                    ];
                }
            }
            _ => { /* skip */ }
        }
    }
}

// ─── Include ──────────────────────────────────────────────────────────────────

fn handle_include(parser: &mut Parser) -> Result<(), String> {
    let filename = parser.require()?;
    parser.push_include(&filename)
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn skip_to_close(parser: &mut Parser) -> Result<(), String> {
    loop {
        let tok = parser.require()?;
        if tok == "}" {
            return Ok(());
        }
    }
}

fn parse_strategy(s: &str) -> Strategy {
    match s.to_lowercase().as_str() {
        "sit1"  => Strategy::Sit1,
        "sit2"  => Strategy::Sit2,
        "sit3"  => Strategy::Sit3,
        "sit4"  => Strategy::Sit4,
        "hunt1" => Strategy::Hunt1,
        "hunt2" => Strategy::Hunt2,
        "hunt3" => Strategy::Hunt3,
        "hunt4" => Strategy::Hunt4,
        _       => Strategy::DoNothing,
    }
}

fn parse_trigger(s: &str) -> EventTrigger {
    match s.to_lowercase().as_str() {
        "approach" => EventTrigger::Approach,
        "depart"   => EventTrigger::Depart,
        "true"     => EventTrigger::True,
        "score"    => EventTrigger::Score,
        "destroy"  => EventTrigger::Destroy,
        "alarm"    => EventTrigger::Alarm,
        "stopnear" => EventTrigger::StopNear,
        "shields"  => EventTrigger::Shields,
        _          => EventTrigger::Null,
    }
}

fn parse_action_kind(s: &str) -> EventActionKind {
    match s.to_lowercase().as_str() {
        "message"      => EventActionKind::Message,
        "hide"         => EventActionKind::Hide,
        "unhide"       => EventActionKind::Unhide,
        "destroy"      => EventActionKind::Boom, // use Boom as stand-in; Phase 5 adds real destroy
        "score"        => EventActionKind::None, // score delta handled separately; placeholder
        "enable"       => EventActionKind::Enable,
        "disable"      => EventActionKind::Disable,
        "loadmission"  => EventActionKind::LoadMission,
        "stop"         => EventActionKind::Stop,
        "boom"         => EventActionKind::Boom,
        "flash"        => EventActionKind::Flash,
        "moveobject"   => EventActionKind::MoveObject,
        "moveplayer"   => EventActionKind::MovePlayer,
        "moveplanet"   => EventActionKind::MovePlanet,
        "hideplanet"   => EventActionKind::HidePlanet,
        "unhideplanet" => EventActionKind::UnhidePlanet,
        "betray"       => EventActionKind::Betray,
        _              => EventActionKind::None,
    }
}

/// Find a model by stem name in world.models, or load it from disk.
/// Returns None if not found/loadable.
fn find_or_load_model(world: &mut World, name: &str, models_dir: &Path) -> Option<usize> {
    // Strip file extension to get stem.
    let stem = Path::new(name).file_stem()?.to_str()?;

    // Already loaded?
    if let Some(idx) = world.models.iter().position(|m| m.name.eq_ignore_ascii_case(stem)) {
        return Some(idx);
    }

    // Try .tri first, then .ac.
    let tri_path = models_dir.join(format!("{}.tri", stem));
    let ac_path  = models_dir.join(format!("{}.ac",  stem));

    let model: Option<Model> = if tri_path.exists() {
        tri_loader::load_tri(&tri_path).ok()
    } else if ac_path.exists() {
        ac3d_loader::load_ac3d(&ac_path).ok()
    } else {
        None
    };

    if let Some(m) = model {
        let idx = world.models.len();
        world.models.push(m);
        Some(idx)
    } else {
        None
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::World;

    fn make_world() -> World {
        let mut w = World::new();
        crate::planet_data::init_weapons(&mut w);
        crate::planet_data::reset_planets(&mut w);
        w
    }

    #[test]
    fn train01_player_near_earth() {
        let mut world = make_world();
        load_mission(
            &mut world,
            "train01.msn",
            Path::new("missions"),
            Path::new("models"),
        )
        .expect("load_mission failed");

        // Earth is the 3rd body (index 2 in our planet array — check by name).
        let earth = world
            .planets
            .iter()
            .find(|p| p.name.eq_ignore_ascii_case("Earth"))
            .expect("Earth not found");

        let dist = (world.player.pos - earth.pos).mag2().sqrt();
        // Cursor was set to Earth -15000 km → ~2.5 game units from Earth centre.
        // Player spawns at cursor, so range to Earth pos ≈ 2.5 units.
        assert!(
            dist < 10.0,
            "Player should be near Earth, got dist={:.3}",
            dist
        );
    }

    #[test]
    fn train01_events_loaded() {
        let mut world = make_world();
        load_mission(
            &mut world,
            "train01.msn",
            Path::new("missions"),
            Path::new("models"),
        )
        .unwrap();

        let active: Vec<_> = world.events.iter().filter(|e| e.pending).collect();
        assert!(!active.is_empty(), "train01 should load events");
    }

    #[test]
    fn train03_object_spawned() {
        let mut world = make_world();
        load_mission(
            &mut world,
            "train03.msn",
            Path::new("missions"),
            Path::new("models"),
        )
        .unwrap();

        let alive = world.targets.iter().filter(|t| t.age > 0.0).count();
        assert_eq!(alive, 1, "train03 should spawn exactly one target");
    }

    #[test]
    fn train04_four_objects_spawned() {
        let mut world = make_world();
        load_mission(
            &mut world,
            "train04.msn",
            Path::new("missions"),
            Path::new("models"),
        )
        .unwrap();

        let alive = world.targets.iter().filter(|t| t.age > 0.0).count();
        assert_eq!(alive, 4, "train04 should spawn four targets");
    }
}
