use crate::math::Vec3;
use crate::types::Player;

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
