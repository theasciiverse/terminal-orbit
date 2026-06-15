use std::io::{self, BufRead};
use std::path::Path;

use crate::math::Vec3;
use super::{Model, ModelBuilder};

/// Load a `.tri` model file.
/// Each line: `x1 y1 z1  x2 y2 z2  x3 y3 z3  0xRRGGBB`
/// Coordinates are divided by 100 to produce game-unit model-space positions.
pub fn load_tri(path: &Path) -> io::Result<Model> {
    let file = std::fs::File::open(path)?;
    let reader = io::BufReader::new(file);
    let name = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let mut builder = ModelBuilder::new();

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let coords = parse_tri_line(line);
        if let Some([v1, v2, v3]) = coords {
            builder.add_edge(v1, v2);
            builder.add_edge(v2, v3);
            builder.add_edge(v3, v1);
        }
    }

    Ok(builder.build(name))
}

/// Parse one .tri line into three Vec3 vertices (÷100 applied).
/// Format: `x1 y1 z1  x2 y2 z2  x3 y3 z3  <hex_color>`
fn parse_tri_line(line: &str) -> Option<[Vec3; 3]> {
    let mut it = line.split_whitespace();
    let mut f = || -> Option<f64> { it.next()?.parse().ok() };

    let v1 = Vec3::new(f()? / 100.0, f()? / 100.0, f()? / 100.0);
    let v2 = Vec3::new(f()? / 100.0, f()? / 100.0, f()? / 100.0);
    let v3 = Vec3::new(f()? / 100.0, f()? / 100.0, f()? / 100.0);
    // hex color token — ignored for wireframe rendering
    it.next()?;

    Some([v1, v2, v3])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn load_light1_tri() {
        let path = PathBuf::from("models/light1.tri");
        if !path.exists() { return; } // skip if models not present
        let model = load_tri(&path).expect("load_tri failed");
        assert!(!model.edges.is_empty(), "should have edges");
        assert!(model.radius > 0.0, "radius should be positive");
        // After ÷100, light1 fits roughly in ±0.018 game units
        assert!(model.radius < 0.1, "radius too large: {}", model.radius);
    }
}
