use crate::math::Vec3;
use crate::star_data::STARS;

use super::canvas::BrailleCanvas;
use super::projection::Camera;

/// Return all 2000 star unit-vectors from the compiled-in table.
pub fn all_stars() -> Vec<Vec3> {
    STARS.iter().map(|s| Vec3::new(s[0] as f64, s[1] as f64, s[2] as f64)).collect()
}

/// Draw the star field. `dense` = all 2000 stars; otherwise first 200.
pub fn draw_stars(canvas: &mut BrailleCanvas, camera: &Camera, stars: &[Vec3], dense: bool) {
    let count = if dense { stars.len() } else { stars.len().min(200) };
    for &star in &stars[..count] {
        if let Some((px, py)) = camera.project_direction(star) {
            canvas.set(px, py);
        }
    }
}
