use crate::math::Vec3;
use crate::types::Player;

// Standard-orbit camera framing, in planet radii (see Camera::standard_orbit).
const STD_ORBIT_BACK:  f64 = 0.30;
const STD_ORBIT_UP:    f64 = 0.12;
const STD_ORBIT_SIDE:  f64 = 0.10;
const STD_ORBIT_AHEAD: f64 = 0.20;
/// How far below the frame centre the horizon sits (radians; half the
/// vertical field of view is 30 degrees).
const STD_ORBIT_HORIZON: f64 = 20.0 * std::f64::consts::PI / 180.0;
const STD_ORBIT_MIN_TILT: f64 = 5.0 * std::f64::consts::PI / 180.0;
/// Size of the player ship drawn in the standard-orbit shot, in planet radii
/// per model unit (the model is 6 units nose to tail).
pub const STD_ORBIT_SHIP_SCALE: f64 = 0.02;

pub struct Camera {
    pub pos:       Vec3,
    pub view:      Vec3,
    pub up:        Vec3,
    pub right:     Vec3,
    /// 1 / tan(30°) — maps the vertical half-FOV (60° total) to NDC ±1.
    pub fov_scale: f64,
    /// width_dots / height_dots — braille dots are square so this is pure pixel ratio.
    pub aspect:    f64,
    pub w_dots:    u32,
    pub h_dots:    u32,
}

impl Camera {
    pub fn from_player(player: &Player, w_dots: u32, h_dots: u32) -> Self {
        Camera {
            pos:       player.pos,
            view:      player.view,
            up:        player.up,
            right:     player.right,
            fov_scale: 1.0 / (std::f64::consts::PI / 6.0).tan(),
            aspect:    w_dots as f64 / h_dots as f64,
            w_dots,
            h_dots,
        }
    }

    /// Project a world-space point to braille-dot coordinates.
    /// Returns None if behind the camera, outside FOV, or out of canvas bounds.
    pub fn project_point(&self, world_pos: Vec3) -> Option<(u32, u32)> {
        let dp = world_pos - self.pos;
        self.project_dp(dp)
    }

    /// Project a unit direction (for stars — no parallax).
    pub fn project_direction(&self, dir: Vec3) -> Option<(u32, u32)> {
        self.project_dp(dir)
    }

    /// Camera at `pos` looking at `target`, rolled so `up_hint` points up on screen.
    pub fn look_at(pos: Vec3, target: Vec3, up_hint: Vec3, w_dots: u32, h_dots: u32) -> Self {
        let view = (target - pos).normalize();
        let mut right = up_hint.cross(view);
        if right.mag2() < 1e-10 {
            right = Vec3::new(0.0, 0.0, 1.0).cross(view);
        }
        let right = right.normalize();
        Camera {
            pos,
            view,
            up: view.cross(right).normalize(),
            right,
            fov_scale: 1.0 / (std::f64::consts::PI / 6.0).tan(),
            aspect: w_dots as f64 / h_dots as f64,
            w_dots,
            h_dots,
        }
    }

    /// The classic starship-in-orbit shot: behind, above and a little to the
    /// side of the ship, which sits in the upper middle of the frame with the
    /// planet's curved horizon across the lower part.  All distances scale
    /// with the planet radius so the shot looks the same at any world.
    pub fn standard_orbit(
        ship_pos: Vec3,
        ship_view: Vec3,
        planet_pos: Vec3,
        planet_radius: f64,
        w_dots: u32,
        h_dots: u32,
    ) -> Self {
        let outward = (ship_pos - planet_pos).normalize();
        // Forward along the ground track, flattened onto the local horizon.
        let mut fwd = ship_view - outward * ship_view.dot(outward);
        if fwd.mag2() < 1e-10 {
            fwd = outward.cross(Vec3::new(0.0, 0.0, 1.0));
        }
        let fwd = fwd.normalize();
        let side = fwd.cross(outward).normalize();
        let r = planet_radius;
        let pos = ship_pos - fwd * (STD_ORBIT_BACK * r)
            + outward * (STD_ORBIT_UP * r)
            + side * (STD_ORBIT_SIDE * r);

        // Aim along the ground track, turned in toward the ship, then tilted
        // down so the horizon (which dips below level by acos(R / d) seen
        // from distance d) sits in the lower third of the frame.
        let ahead = (fwd * (STD_ORBIT_BACK + STD_ORBIT_AHEAD) - side * STD_ORBIT_SIDE).normalize();
        let cam_up = (pos - planet_pos).normalize();
        let dip = (r / (pos - planet_pos).mag2().sqrt()).min(1.0).acos();
        let tilt = (dip - STD_ORBIT_HORIZON).max(STD_ORBIT_MIN_TILT);
        let view = ahead * tilt.cos() - cam_up * tilt.sin();
        let target = pos + view;
        Camera::look_at(pos, target, cam_up, w_dots, h_dots)
    }

    /// Camera placed behind and above the ship, looking diagonally past it
    /// toward `focus` (the orbited planet).  The ship silhouette sits in the
    /// foreground with the planet visible beyond it.
    /// `dist` = distance behind ship, `height` = distance above ship.
    pub fn third_person(
        ship_pos: Vec3,
        ship_view: Vec3,
        ship_up: Vec3,
        focus: Vec3,
        w_dots: u32,
        h_dots: u32,
        dist: f64,
        height: f64,
    ) -> Self {
        // Camera sits behind and slightly above the ship.
        let pos = ship_pos - ship_view * dist + ship_up * height;
        // Look diagonally toward the planet (not at the ship).
        let view = (focus - pos).normalize();
        // Derive right/up from view + ship_up hint.
        let right = view.cross(ship_up);
        let (right, up) = if right.mag2() > 1e-10 {
            let r = right.normalize();
            (r, r.cross(view).normalize())
        } else {
            let fallback = Vec3::new(1.0, 0.0, 0.0);
            let r = view.cross(fallback).normalize();
            (r, r.cross(view).normalize())
        };
        Camera {
            pos,
            view,
            up,
            right,
            fov_scale: 1.0 / (std::f64::consts::PI / 6.0).tan(),
            aspect: w_dots as f64 / h_dots as f64,
            w_dots,
            h_dots,
        }
    }

    pub fn project_dp(&self, dp: Vec3) -> Option<(u32, u32)> {
        let z = dp.dot(self.view);
        if z <= 0.001 {
            return None;
        }
        let x_ndc = dp.dot(self.right) / z * self.fov_scale / self.aspect;
        let y_ndc = dp.dot(self.up)    / z * self.fov_scale;

        if x_ndc < -1.0 || x_ndc > 1.0 || y_ndc < -1.0 || y_ndc > 1.0 {
            return None;
        }

        let px = ((x_ndc + 1.0) * 0.5 * self.w_dots as f64) as u32;
        let py = ((1.0 - (y_ndc + 1.0) * 0.5) * self.h_dots as f64) as u32;

        if px >= self.w_dots || py >= self.h_dots {
            return None;
        }
        Some((px, py))
    }
}
