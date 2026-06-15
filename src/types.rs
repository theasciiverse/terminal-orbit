use crate::constants::*;
use crate::math::Vec3;
use crate::model::Model;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum FlightModel {
    #[default]
    Newtonian,
    Arcade,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum GameState {
    #[default]
    Normal,
    Init,
    Dead1,
    Dead2,
    LoadGame,
    GetText,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Strategy {
    #[default]
    DoNothing,
    Sit1, Sit2, Sit3, Sit4,
    Hunt1, Hunt2, Hunt3, Hunt4,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum LockType {
    #[default]
    Enemy,
    Friendly,
    Planet,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum EventTrigger {
    #[default]
    Null,
    Approach, Destroy, Score, Alarm, Depart, True, StopNear, Shields,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum EventActionKind {
    #[default]
    None,
    Message, Hide, Unhide, Enable, Disable,
    LoadMission, Stop, Boom, Flash,
    MoveObject, MovePlayer, MovePlanet, HidePlanet, UnhidePlanet, Betray,
}

#[derive(Debug, Clone, Default)]
pub struct EventAction {
    pub active: bool,
    pub kind: EventActionKind,
    pub ivalue: i32,
    pub fvalue: f64,
    pub cvalue: String,
}

#[derive(Debug, Clone, Default)]
pub struct GameEvent {
    pub name: String,
    pub pending: bool,
    pub enabled: bool,
    pub trigger: EventTrigger,
    pub ivalue: i32,
    pub fvalue: f64,
    pub cvalue: String,
    pub pos: Vec3,
    pub actions: Vec<EventAction>,
}

#[derive(Debug, Clone, Default)]
pub struct Player {
    pub name: String,
    pub model_name: String,
    pub pos: Vec3,
    pub up: Vec3,
    pub view: Vec3,
    pub right: Vec3,

    pub move_forward: f64,
    pub move_backward: f64,
    pub move_up: f64,
    pub move_down: f64,
    pub move_pitchleft: f64,
    pub move_pitchright: f64,
    pub move_left: f64,
    pub move_right: f64,

    pub flight_model: FlightModel,
    pub vel: Vec3,
    pub throttle: f64,
    pub score: i32,
    pub shields: f64,
    pub maxshields: f64,
    pub weapon: usize,
    pub msl_idle: f64,
    pub waypoint: usize,
    pub dead_timer: f64,
    pub still: bool,
    pub viewlock: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Target {
    pub pos: Vec3,
    pub vel: Vec3,
    pub view: Vec3,
    pub up: Vec3,
    pub right: Vec3,

    /// age == 0 means this slot is unused.
    pub age: f64,
    pub range2: f64,

    pub move_forward: f64,
    pub move_backward: f64,
    pub move_up: f64,
    pub move_down: f64,
    pub move_pitchleft: f64,
    pub move_pitchright: f64,
    pub move_left: f64,
    pub move_right: f64,

    pub msl_idle: f64,
    pub name: String,
    pub score: i32,
    pub model: Option<usize>,
    pub strategy: Strategy,
    pub hidden: bool,
    pub invisible: bool,
    pub friendly: bool,
    pub shields: f64,
    pub maxshields: f64,
    pub shieldregen: f64,
    pub turnrate: f64,
    pub maxvel: f64,
    pub weapon: usize,
}

#[derive(Debug, Clone, Default)]
pub struct Missile {
    pub pos: Vec3,
    pub vel: Vec3,
    /// age == 0 means not in use.
    pub age: f64,
    pub friendly: bool,
    pub weapon: usize,
    /// -1 = player, >=0 = target index
    pub owner: i32,
}

#[derive(Debug, Clone, Default)]
pub struct Planet {
    pub hidden: bool,
    pub dist: f64,
    pub pos: Vec3,
    pub theta: f64,
    pub radius: f64,
    pub oblicity: f64,
    pub radius2: f64,
    pub range2: f64,
    pub absrange2: f64,
    pub mass: f64,
    pub name: String,
    pub is_moon: bool,
    pub primary: usize,
    pub angvel: f64,
    pub custom: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Weapon {
    pub name: String,
    /// Renamed from C's 'yield' (reserved keyword).
    pub damage: f64,
    pub speed: f64,
    pub idle: f64,
    pub expire: f64,
    pub renderer: u8,
    pub color: [f32; 3],
    pub range2: f64,
}

#[derive(Debug, Clone, Default)]
pub struct Boom {
    pub pos: Vec3,
    /// age == 0 means not in use.
    pub age: f64,
    pub angle: f64,
    pub size: f64,
}

#[derive(Debug, Clone, Default)]
pub struct Lock {
    pub target: Option<usize>,
    pub lock_type: LockType,
}

#[derive(Debug, Clone)]
pub struct Console {
    pub age: Vec<f64>,
    pub next: usize,
    pub buf: Vec<String>,
}

impl Default for Console {
    fn default() -> Self {
        Console {
            age: vec![0.0; CONSLINES],
            next: 0,
            buf: vec![String::new(); CONSLINES],
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Waypoint {
    pub pos: Vec3,
}

#[derive(Debug, Clone, Default)]
pub struct Message {
    pub text: String,
    pub age: f64,
    /// Last message shown — survives when text clears, recalled with M key.
    pub last_text: String,
}

/// Top-level game world — owns all mutable state.
pub struct World {
    pub player: Player,
    pub targets: Vec<Target>,
    pub missiles: Vec<Missile>,
    pub planets: Vec<Planet>,
    pub weapons: Vec<Weapon>,
    pub booms: Vec<Boom>,
    pub events: Vec<GameEvent>,
    pub models: Vec<Model>,
    pub waypoints: Vec<Waypoint>,
    pub nwaypoints: usize,
    pub lock: Lock,
    pub console: Console,
    pub message: Message,

    pub delta_t: f64,
    pub abs_t: f64,

    pub gravity: bool,
    pub orbit_planets: bool,
    pub full_stop: bool,
    pub warpspeed: bool,
    pub superwarp: bool,
    pub state: GameState,
    pub vulnerable: bool,
    pub compression: f64,
    pub real_distances: bool,
    pub paused: bool,

    /// Current mission filename (e.g. "train01.msn").
    pub mission_file: String,
    /// Briefing text set by the mission loader.
    pub briefing: String,
    pub mission_verbose: bool,

    /// Set to Some(filename) to trigger a mission reload on the next tick.
    pub pending_mission: Option<String>,

    /// Running total of shots fired by targets; used in tests.
    pub shots_fired: u32,
}

impl World {
    pub fn new() -> Self {
        let mut targets: Vec<Target> = (0..NTARGETS).map(|_| {
            let mut t = Target::default();
            t.weapon = NPLAYER_WEAPONS;
            t.shields = 100.0;
            t.maxshields = 100.0;
            t.name = "Target".to_string();
            t.view = Vec3::new(1.0, 0.0, 0.0);
            t.up = Vec3::new(0.0, 0.0, 1.0);
            t.right = t.up.cross(t.view).normalize();
            t
        }).collect();
        // Silence unused warning — targets is populated above.
        let _ = &targets;

        let mut w = World {
            player: Player::default(),
            targets,
            missiles: vec![Missile::default(); NMSLS],
            planets: vec![Planet::default(); NPLANETS],
            weapons: vec![Weapon::default(); NWEAPONS],
            booms: vec![Boom::default(); NBOOMS],
            events: vec![GameEvent::default(); NEVENTS],
            models: Vec::new(),
            waypoints: vec![Waypoint::default(); NWAYPOINTS],
            nwaypoints: 0,
            lock: Lock::default(),
            console: Console::default(),
            message: Message::default(),
            delta_t: 1.0 / 30.0,
            abs_t: 0.0,
            gravity: false,
            orbit_planets: false,
            full_stop: false,
            warpspeed: false,
            superwarp: false,
            state: GameState::Normal,
            vulnerable: true,
            compression: 1.0,
            real_distances: false,
            paused: false,
            mission_file: String::new(),
            briefing: String::new(),
            mission_verbose: false,
            pending_mission: None,
            shots_fired: 0,
        };

        w.player.name = "Player".to_string();
        w.player.up = Vec3::new(0.0, 0.0, 1.0);
        w.player.view = Vec3::new(1.0, 0.0, 0.0);
        w.player.right = w.player.up.cross(w.player.view).normalize();
        w.player.shields = 100.0;
        w.player.maxshields = 100.0;

        w
    }
}
