use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let stars_h = "asciiverse/reference/space-orbit/src/stars.h";
    println!("cargo:rerun-if-changed={stars_h}");

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest = Path::new(&out_dir).join("star_data.rs");

    if let Ok(text) = fs::read_to_string(stars_h) {
        let entries: Vec<[f32; 4]> = parse_stars(&text);
        if entries.len() >= 1900 {
            let mut out = String::from("pub const STARS: &[[f32; 4]] = &[\n");
            for e in &entries {
                out.push_str(&format!("    [{:.6}f32, {:.6}f32, {:.6}f32, {:.6}f32],\n",
                    e[0], e[1], e[2], e[3]));
            }
            out.push_str("];\n");
            fs::write(&dest, out).unwrap();
            return;
        }
    }

    // Fallback: Fibonacci sphere lattice (no stars.h available).
    let phi = (1.0_f32 + 5.0_f32.sqrt()) / 2.0;
    let n = 2000usize;
    let mut out = String::from("pub const STARS: &[[f32; 4]] = &[\n");
    for i in 0..n {
        let y = 1.0 - (i as f32 / (n - 1) as f32) * 2.0;
        let r = (1.0 - y * y).max(0.0).sqrt();
        let theta = 2.0 * std::f32::consts::PI * i as f32 / phi;
        out.push_str(&format!("    [{:.6}f32, {:.6}f32, {:.6}f32, 0.000000f32],\n",
            r * theta.cos(), y, r * theta.sin()));
    }
    out.push_str("];\n");
    fs::write(&dest, out).unwrap();
}

fn parse_stars(text: &str) -> Vec<[f32; 4]> {
    let mut result = Vec::with_capacity(2000);
    for line in text.lines() {
        let line = line.trim();
        if !line.starts_with('{') { continue; }
        // Extract everything between the outermost { } braces.
        let Some(open)  = line.find('{') else { continue };
        // Use the FIRST closing brace — the last line ends with `} };` (array closer)
        // and rfind would grab the wrong one.
        let Some(close) = line[open..].find('}').map(|i| open + i) else { continue };
        if close <= open { continue; }
        let inner = &line[open + 1..close];
        let nums: Vec<f32> = inner.split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect();
        if nums.len() == 4 {
            result.push([nums[0], nums[1], nums[2], nums[3]]);
        }
    }
    result
}
