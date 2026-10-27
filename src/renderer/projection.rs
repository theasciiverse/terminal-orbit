use crate::math::Vec3;
use crate::types::Player;

// Standard-orbit camera framing, in planet radii (see Camera::standard_orbit).
/// Camera distance outside the ship's orbit.
const STD_ORBIT_OUT:   f64 = 0.16;
/// Camera distance behind (Above shot) or ahead of (Below shot) the ship.
const STD_ORBIT_ALONG: f64 = 0.42;
/// Camera height above (Above shot) or below (Below shot) the equatorial plane.
const STD_ORBIT_HIGH:  f64 = 0.06;
/// How far the view turns from the planet toward the ship (0 = planet, 1 = ship).
const STD_ORBIT_AIM:   f64 = 0.85;
/// Size of the player ship drawn in the standard-orbit shot, in planet radii
/// per model unit (the model is 6 units nose to tail).
pub const STD_ORBIT_SHIP_SCALE: f64 = 0.028;

/// The two standard-orbit shots, both looking across the ship at the planet
/// with north up and the ship on the equator.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum OrbitShot {
    /// From above and slightly behind; the planet fills the left.
    Above,
    /// From below and slightly in front; the planet fills the right.
    Below,
}

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

    /// Standard-orbit shot: from outside the orbit, just above or below the
    /// equatorial plane, looking across the ship toward the planet, north
    /// up.  All distances scale with the planet radius so the shot looks the
    /// same at any world.
    pub fn standard_orbit(
        ship_pos: Vec3,
        ship_view: Vec3,
        planet_pos: Vec3,
        planet_radius: f64,
        shot: OrbitShot,
        w_dots: u32,
        h_dots: u32,
    ) -> Self {
        let north = Vec3::new(0.0, 0.0, 1.0);
        let offset = ship_pos - planet_pos;
        let mut outward = Vec3::new(offset.x, offset.y, 0.0);
        if outward.mag2() < 1e-12 {
            outward = Vec3::new(1.0, 0.0, 0.0);
        }
        let outward = outward.normalize();
        // Direction of travel along the equator.
        let mut fwd = ship_view - outward * ship_view.dot(outward) - north * ship_view.z;
        if fwd.mag2() < 1e-10 {
            fwd = outward.cross(north);
        }
        let fwd = fwd.normalize();

        let r = planet_radius;
        let (along, high) = match shot {
            OrbitShot::Above => (-STD_ORBIT_ALONG, STD_ORBIT_HIGH),
            OrbitShot::Below => (STD_ORBIT_ALONG, -STD_ORBIT_HIGH),
        };
        let pos = ship_pos + outward * (STD_ORBIT_OUT * r) + fwd * (along * r) + north * (high * r);
        let to_ship = (ship_pos - pos).normalize();
        let to_planet = (planet_pos - pos).normalize();
        let view = (to_ship * STD_ORBIT_AIM + to_planet * (1.0 - STD_ORBIT_AIM)).normalize();
        Camera::look_at(pos, pos + view, north, w_dots, h_dots)
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
