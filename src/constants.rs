pub const KM_TO_UNITS1: f64 = 6_000.0;
pub const KM_TO_UNITS2: f64 = 6_000.0 / 1_000_000.0;

pub const THETA: f64 = 1.6;
pub const DELTAV: f64 = 0.2;
pub const WARP_COEFF: f64 = 0.2;
pub const MAX_THROTTLE: f64 = 10_000.0 / KM_TO_UNITS1;
pub const MAX_WARP_THROTTLE: f64 = 1_000_000.0 / KM_TO_UNITS1;

pub const G: f64 = 0.025;
pub const RMIN: f64 = 2.0;
pub const MAXDELTAT: f64 = 0.1;

pub const NPLANETS: usize = 32;
pub const NTARGETS: usize = 32;
pub const NMSLS: usize = 32;
pub const NBOOMS: usize = 32;
pub const NWAYPOINTS: usize = 32;
pub const NSTARS: usize = 2000;
pub const NWEAPONS: usize = 10;
pub const NPLAYER_WEAPONS: usize = 4;
pub const NEVENTS: usize = 32;
pub const ACTIONS_PER_EVENT: usize = 64;
pub const CONSLINES: usize = 10;
pub const NSAVES: usize = 10;

pub const MSL_EXPIRE: f64 = 5.0;
pub const MSL_IDLE: f64 = 0.2;
pub const MSL_MIN_AGE: f64 = 0.1;
pub const MSL_VEL: f64 = 0.5;
pub const BOOM_TIME: f64 = 1.0;
pub const CONSAGE: f64 = 3.0;
pub const SHIELD_REGEN: f64 = 5.0;
pub const DEAD_TIME: f64 = 5.0;
pub const MSG_MAXAGE: f64 = 30.0;

pub const TARGDIST: f64 = 0.02;
pub const TARGDIST2: f64 = TARGDIST * TARGDIST;
pub const MINFIREDIST: f64 = 1.0 / KM_TO_UNITS1;
pub const MINFIREDIST2: f64 = MINFIREDIST * MINFIREDIST;
pub const MAXFIREDIST: f64 = 3_000.0 / KM_TO_UNITS1;
pub const MAXFIREDIST2: f64 = MAXFIREDIST * MAXFIREDIST;
pub const TARG_MAXRANGE: f64 = 50_000.0 / KM_TO_UNITS1;
pub const TARG_MAXRANGE2: f64 = TARG_MAXRANGE * TARG_MAXRANGE;

pub const THINK_CUTOFFA: f64 = 20_000.0 / KM_TO_UNITS1;
pub const THINK_CUTOFFA2: f64 = THINK_CUTOFFA * THINK_CUTOFFA;
pub const THINK_CUTOFFB: f64 = 2_000.0 / KM_TO_UNITS1;
pub const THINK_CUTOFFB2: f64 = THINK_CUTOFFB * THINK_CUTOFFB;
pub const THINK_CUTOFFC: f64 = 500.0 / KM_TO_UNITS1;
pub const THINK_CUTOFFC2: f64 = THINK_CUTOFFC * THINK_CUTOFFC;
