use std::io::{self, BufRead};
use std::path::Path;

use crate::math::Vec3;
use super::{Model, ModelBuilder};

/// Load an AC3D `.ac` model file.
/// Vertices and loc offsets are divided by 100 (same scale as .tri files).
/// Group-level ROT transforms are skipped (first-pass simplification per CLAUDE.md).
pub fn load_ac3d(path: &Path) -> io::Result<Model> {
    let file = std::fs::File::open(path)?;
    let reader = io::BufReader::new(file);
    let name = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let lines: Vec<String> = reader.lines().collect::<io::Result<_>>()?;
    let lines: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();

    let mut builder = ModelBuilder::new();
    let mut pos = 0;

    // Skip header (AC3Db) and MATERIAL lines
    while pos < lines.len() {
        let line = lines[pos].trim();
        if line.starts_with("OBJECT ") {
            pos += 1;  // consume OBJECT line
            parse_object(&lines, &mut pos, Vec3::default(), &mut builder);
            break;
        }
        pos += 1;
    }

    Ok(builder.build(name))
}

/// Recursively parse one AC3D OBJECT (already past the "OBJECT <type>" line).
fn parse_object(lines: &[&str], pos: &mut usize, parent_loc: Vec3, builder: &mut ModelBuilder) {
    let mut my_loc = Vec3::default();
    let mut local_verts: Vec<Vec3> = Vec::new();

    while *pos < lines.len() {
        let line = lines[*pos].trim();

        if line.starts_with("loc ") {
            *pos += 1;
            if let Some(v) = parse_vec3(&line[4..]) {
                my_loc = Vec3::new(v.x / 100.0, v.y / 100.0, v.z / 100.0);
            }
        } else if line.starts_with("numvert ") {
            *pos += 1;
            let n: usize = line[8..].trim().parse().unwrap_or(0);
            let offset = parent_loc + my_loc;
            local_verts.clear();
            for _ in 0..n {
                if *pos < lines.len() {
                    let vline = lines[*pos].trim();
                    *pos += 1;
                    if let Some(v) = parse_vec3(vline) {
                        local_verts.push(Vec3::new(v.x / 100.0, v.y / 100.0, v.z / 100.0) + offset);
                    } else {
                        local_verts.push(offset); // fallback for malformed line
                    }
                }
            }
        } else if line.starts_with("numsurf ") {
            *pos += 1;
            let n: usize = line[8..].trim().parse().unwrap_or(0);
            for _ in 0..n {
                parse_surf(lines, pos, &local_verts, builder);
            }
        } else if line.starts_with("kids ") {
            *pos += 1;
            let n: usize = line[5..].trim().parse().unwrap_or(0);
            let child_loc = parent_loc + my_loc;
            for _ in 0..n {
                // Advance to the next OBJECT line
                while *pos < lines.len() && !lines[*pos].trim().starts_with("OBJECT ") {
                    *pos += 1;
                }
                if *pos < lines.len() {
                    *pos += 1; // consume OBJECT line
                    parse_object(lines, pos, child_loc, builder);
                }
            }
            return; // done with this object
        } else {
            *pos += 1; // skip unrecognised lines (name, texture, rot, url, …)
        }
    }
}

/// Parse one SURF block, extract polygon edges into builder.
/// Expects to be called just before the "SURF <flags>" line.
fn parse_surf(lines: &[&str], pos: &mut usize, local_verts: &[Vec3], builder: &mut ModelBuilder) {
    // Find the SURF line
    while *pos < lines.len() && !lines[*pos].trim().starts_with("SURF ") {
        // stop if we hit the next numsurf-level keyword or an OBJECT
        let line = lines[*pos].trim();
        if line.starts_with("OBJECT ") || line.starts_with("kids ") {
            return;
        }
        *pos += 1;
    }
    if *pos >= lines.len() {
        return;
    }
    *pos += 1; // consume SURF line

    // Parse mat and refs sub-lines
    while *pos < lines.len() {
        let line = lines[*pos].trim();

        if line.starts_with("mat ") {
            *pos += 1;
            // material index — not needed for wireframe
        } else if line.starts_with("refs ") {
            let n: usize = line[5..].trim().parse().unwrap_or(0);
            *pos += 1;
            let mut poly: Vec<usize> = Vec::with_capacity(n);
            for _ in 0..n {
                if *pos < lines.len() {
                    let rline = lines[*pos].trim();
                    *pos += 1;
                    let vidx: usize = rline.split_whitespace()
                        .next()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);
                    poly.push(vidx);
                }
            }
            // Add polygon edges
            let nv = poly.len();
            for i in 0..nv {
                let a = poly[i];
                let b = poly[(i + 1) % nv];
                if a < local_verts.len() && b < local_verts.len() {
                    builder.add_edge(local_verts[a], local_verts[b]);
                }
            }
            return; // refs is the last token in a SURF block
        } else {
            break; // unexpected line — stop
        }
    }
}

fn parse_vec3(s: &str) -> Option<Vec3> {
    let mut it = s.split_whitespace();
    let x: f64 = it.next()?.parse().ok()?;
    let y: f64 = it.next()?.parse().ok()?;
    let z: f64 = it.next()?.parse().ok()?;
    Some(Vec3::new(x, y, z))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn load_light1_ac() {
        let path = PathBuf::from("models/light1.ac");
        if !path.exists() { return; }
        let model = load_ac3d(&path).expect("load_ac3d failed");
        assert!(!model.edges.is_empty(), "should have edges");
        assert!(model.radius > 0.0, "radius should be positive");
        assert!(model.radius < 0.1, "radius too large: {}", model.radius);
    }
}
