use std::path::Path;
use std::collections::HashMap;

pub struct PlanetTexture {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,  // RGB triples, row-major
}

impl PlanetTexture {
    pub fn load(path: &Path) -> Option<Self> {
        // Parse P6 PPM: "P6\n<w> <h>\n<maxval>\n<binary RGB>"
        // Comments (lines starting with #) may appear in header.
        let data = std::fs::read(path).ok()?;

        // Parse header manually - find the 3 header tokens (after "P6")
        if data.len() < 2 || &data[0..2] != b"P6" { return None; }
        let mut pos = 2usize;

        // Skip whitespace and comments
        fn skip_ws_comments(pos: &mut usize, data: &[u8]) {
            loop {
                // skip whitespace
                while *pos < data.len() && (data[*pos] == b' ' || data[*pos] == b'\t' || data[*pos] == b'\r' || data[*pos] == b'\n') {
                    *pos += 1;
                }
                // skip comment
                if *pos < data.len() && data[*pos] == b'#' {
                    while *pos < data.len() && data[*pos] != b'\n' {
                        *pos += 1;
                    }
                } else {
                    break;
                }
            }
        }

        fn read_token(pos: &mut usize, data: &[u8]) -> Option<String> {
            skip_ws_comments(pos, data);
            let start = *pos;
            while *pos < data.len() && data[*pos] != b' ' && data[*pos] != b'\t' && data[*pos] != b'\r' && data[*pos] != b'\n' {
                *pos += 1;
            }
            if start == *pos { None } else { Some(String::from_utf8_lossy(&data[start..*pos]).into_owned()) }
        }

        let w_str = read_token(&mut pos, &data)?;
        let h_str = read_token(&mut pos, &data)?;
        let maxval_str = read_token(&mut pos, &data)?;

        let width: u32 = w_str.parse().ok()?;
        let height: u32 = h_str.parse().ok()?;
        let maxval: u32 = maxval_str.parse().ok()?;

        // After maxval token, skip exactly one whitespace byte
        if pos < data.len() && (data[pos] == b'\n' || data[pos] == b'\r' || data[pos] == b' ' || data[pos] == b'\t') {
            pos += 1;
        }

        let pixel_count = (width * height * 3) as usize;
        if pos + pixel_count > data.len() { return None; }

        let raw = &data[pos..pos + pixel_count];

        let pixels = if maxval == 255 {
            raw.to_vec()
        } else {
            raw.iter().map(|&b| ((b as u32 * 255) / maxval) as u8).collect()
        };

        Some(PlanetTexture { width, height, pixels })
    }

    pub fn sample(&self, u: f64, v: f64) -> (u8, u8, u8) {
        let u = u.clamp(0.0, 1.0);
        let v = v.clamp(0.0, 1.0);

        let x = (u * (self.width as f64 - 1.0)) as usize;
        let y = (v * (self.height as f64 - 1.0)) as usize;

        let x = x.min(self.width as usize - 1);
        let y = y.min(self.height as usize - 1);

        let idx = (y * self.width as usize + x) * 3;
        if idx + 2 < self.pixels.len() {
            (self.pixels[idx], self.pixels[idx + 1], self.pixels[idx + 2])
        } else {
            (0, 0, 0)
        }
    }
}

pub fn load_all_textures(maps_dir: &Path) -> HashMap<String, PlanetTexture> {
    let mut map = HashMap::new();
    let entries = match std::fs::read_dir(maps_dir) {
        Ok(e) => e,
        Err(_) => return map,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("ppm") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                if let Some(tex) = PlanetTexture::load(&path) {
                    map.insert(stem.to_lowercase(), tex);
                }
            }
        }
    }
    map
}
