use crate::math::Vec3;
use crate::star_data::STARS;

use super::canvas::BrailleCanvas;
use super::projection::Camera;

/// Return all 2000 star unit-vectors with their visual magnitudes.
pub fn all_stars() -> Vec<(Vec3, f32)> {
    STARS.iter().map(|s| (Vec3::new(s[0] as f64, s[1] as f64, s[2] as f64), s[3])).collect()
}

/// Draw the star field with magnitude-based dot sizing.
/// `dense` = all 2000 stars; otherwise bright stars (mag < 4.5) up to 400.
pub fn draw_stars(canvas: &mut BrailleCanvas, camera: &Camera, stars: &[(Vec3, f32)], dense: bool) {
    for &(star, mag) in stars.iter() {
        if !dense {
            // In non-dense mode, skip dim stars and cap at 400
            if mag >= 4.5 { continue; }
        }
        if let Some((px, py)) = camera.project_direction(star) {
            if mag < 1.5 {
                // Bright star: 2×2 dot cluster
                canvas.set(px, py);
                canvas.set(px + 1, py);
                canvas.set(px, py + 1);
                canvas.set(px + 1, py + 1);
            } else if mag < 3.5 {
                // Medium star: 2 dots
                canvas.set(px, py);
                canvas.set(px + 1, py);
            } else {
                // Dim star: single dot
                canvas.set(px, py);
            }
        }
    }
}
