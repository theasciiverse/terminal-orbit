pub mod ac3d_loader;
pub mod tri_loader;

use crate::math::Vec3;
use std::collections::HashSet;

/// A loaded ship model: deduplicated edges in model space.
/// Coordinates are already ÷100 and loc offsets applied.
pub struct Model {
    pub name:   String,
    pub edges:  Vec<[Vec3; 2]>,
    pub radius: f64,
}

/// Accumulates edges during loading with deduplication.
pub(crate) struct ModelBuilder {
    seen:   HashSet<[u64; 6]>,
    edges:  Vec<[Vec3; 2]>,
    max_r2: f64,
}

impl ModelBuilder {
    pub fn new() -> Self {
        ModelBuilder { seen: HashSet::new(), edges: Vec::new(), max_r2: 0.0 }
    }

    pub fn add_edge(&mut self, a: Vec3, b: Vec3) {
        if self.seen.insert(edge_key(a, b)) {
            self.edges.push([a, b]);
        }
        let r2 = |v: Vec3| v.x * v.x + v.y * v.y + v.z * v.z;
        self.max_r2 = self.max_r2.max(r2(a)).max(r2(b));
    }

    pub fn build(self, name: impl Into<String>) -> Model {
        Model { name: name.into(), edges: self.edges, radius: self.max_r2.sqrt() }
    }
}

/// Canonical key for an undirected edge — sorts the two endpoints so (A,B) == (B,A).
fn edge_key(a: Vec3, b: Vec3) -> [u64; 6] {
    let key_a = [a.x.to_bits(), a.y.to_bits(), a.z.to_bits()];
    let key_b = [b.x.to_bits(), b.y.to_bits(), b.z.to_bits()];
    let (lo, hi) = if key_a <= key_b { (key_a, key_b) } else { (key_b, key_a) };
    [lo[0], lo[1], lo[2], hi[0], hi[1], hi[2]]
}
