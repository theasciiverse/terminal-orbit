use std::f64::consts::PI;

use crate::math::Vec3;
use crate::types::{Planet, World};

use super::canvas::BrailleCanvas;
use super::projection::Camera;
use super::viewport::draw_edge_world;

// Ring systems: (planet_index, outer_radius_km)
// Jupiter=8, Saturn=13, Uranus=21, Neptune=27
const RINGS: &[(usize, f64)] = &[
    (8,  129_130.0),
    (13, 140_154.0),
    (21,  50_271.0),
    (27,  63_000.0),
];

const KM_TO_UNITS: f64 = crate::constants::KM_TO_UNITS1;

pub fn draw_planets(canvas: &mut BrailleCanvas, camera: &Camera, world: &World) {
    if world.orbit_planets {
        draw_orbits(canvas, camera, world);
    }
    for p in 0..world.planets.len() {
        let planet = &world.planets[p];
        if planet.hidden || planet.radius <= 0.0 {
            continue;
        }
        let dp = planet.pos - camera.pos;
        let abs_range2 = dp.mag2();
        // Range in planet radii (r2 from C code).
        let r = if planet.radius > 0.0 { abs_range2.sqrt() / planet.radius } else { f64::MAX };

        if r < 5.0 {
            draw_sphere(canvas, camera, planet, 12, 12);
            draw_rings_for(canvas, camera, p, world);
        } else if r < 25.0 {
            draw_sphere(canvas, camera, planet, 8, 8);
            draw_rings_for(canvas, camera, p, world);
        } else if r < 200.0 {
            draw_sphere(canvas, camera, planet, 4, 4);
            draw_rings_for(canvas, camera, p, world);
        } else if r < 50_000.0 {
            // Single dot — large planet dot at moderate range.
            let _ = camera.project_point(planet.pos).map(|(x, y)| {
                canvas.set(x, y);
            });
        }
        // Beyond 50 000 radii: skip entirely.
    }
}

// ─── Wireframe sphere ────────────────────────────────────────────────────────

/// Draw a wireframe sphere at `planet.pos` with the planet's radius.
/// `n_lat` = number of latitude rings; `n_lon` = number of meridians.
fn draw_sphere(
    canvas: &mut BrailleCanvas,
    camera: &Camera,
    planet: &Planet,
    n_lat: usize,
    n_lon: usize,
) {
    let center = planet.pos;
    let r = planet.radius;

    // Oblicity: tilt the sphere pole from +Z around the X axis.
    let ob = planet.oblicity.to_radians();
    // Pole direction after oblicity tilt.
    let pole = Vec3::new(0.0, -ob.sin(), ob.cos()).normalize();
    // Two axes in the equatorial plane.
    let eq_x = Vec3::new(1.0, 0.0, 0.0);
    let eq_y = pole.cross(eq_x).normalize();

    // Latitude rings.
    let n_pts = n_lon.max(8) * 2;
    for i_lat in 1..n_lat {
        let lat = PI * (i_lat as f64 / n_lat as f64) - PI / 2.0;
        let ring_r = r * lat.cos();
        let ring_h = r * lat.sin();
        // Points around this ring.
        let pts: Vec<Vec3> = (0..=n_pts)
            .map(|i| {
                let theta = 2.0 * PI * i as f64 / n_pts as f64;
                center + pole * ring_h + eq_x * (ring_r * theta.cos()) + eq_y * (ring_r * theta.sin())
            })
            .collect();
        for i in 0..n_pts {
            draw_edge_world(canvas, camera, pts[i], pts[i + 1]);
        }
    }

    // Meridian arcs.
    let n_pts_m = n_lat * 2;
    for i_lon in 0..n_lon {
        let lon = PI * i_lon as f64 / n_lon as f64;
        let ax = eq_x * lon.cos() + eq_y * lon.sin();
        let pts: Vec<Vec3> = (0..=n_pts_m)
            .map(|i| {
                let lat = PI * i as f64 / n_pts_m as f64 - PI / 2.0;
                center + pole * (r * lat.sin()) + ax * (r * lat.cos())
            })
            .collect();
        for i in 0..n_pts_m {
            draw_edge_world(canvas, camera, pts[i], pts[i + 1]);
        }
    }
}

// ─── Ring systems ─────────────────────────────────────────────────────────────

fn draw_rings_for(canvas: &mut BrailleCanvas, camera: &Camera, planet_idx: usize, world: &World) {
    for &(primary, outer_km) in RINGS {
        if primary != planet_idx { continue; }
        let planet = &world.planets[planet_idx];
        let outer_r = outer_km / KM_TO_UNITS;
        draw_ring(canvas, camera, planet, outer_r);
    }
}

/// Draw a single ring (circle in the equatorial plane) at radius `ring_r`.
fn draw_ring(canvas: &mut BrailleCanvas, camera: &Camera, planet: &Planet, ring_r: f64) {
    const N_RING_PTS: usize = 32;
    let center = planet.pos;

    let ob = planet.oblicity.to_radians();
    let pole = Vec3::new(0.0, -ob.sin(), ob.cos()).normalize();
    let eq_x = Vec3::new(1.0, 0.0, 0.0);
    let eq_y = pole.cross(eq_x).normalize();

    let pts: Vec<Vec3> = (0..=N_RING_PTS)
        .map(|i| {
            let theta = 2.0 * PI * i as f64 / N_RING_PTS as f64;
            center + eq_x * (ring_r * theta.cos()) + eq_y * (ring_r * theta.sin())
        })
        .collect();
    for i in 0..N_RING_PTS {
        draw_edge_world(canvas, camera, pts[i], pts[i + 1]);
    }
}

// ─── Orbital paths ────────────────────────────────────────────────────────────

fn draw_orbits(canvas: &mut BrailleCanvas, camera: &Camera, world: &World) {
    const N_ORBIT_PTS: usize = 64;

    for p in 1..world.planets.len() {
        let planet = &world.planets[p];
        if planet.hidden || planet.dist <= 0.0 { continue; }

        // Orbit center is the primary's position.
        let center = if planet.is_moon {
            let pr = planet.primary;
            if pr < world.planets.len() { world.planets[pr].pos } else { Vec3::zero() }
        } else {
            // Non-moon: orbits Sol (index 0).
            world.planets[0].pos
        };

        // Don't draw moon orbits when primary is too far away (clutter).
        if planet.is_moon {
            let pr = planet.primary;
            if pr < world.planets.len() {
                let r = (world.planets[pr].pos - camera.pos).mag2().sqrt();
                if r > world.planets[pr].radius * 200.0 { continue; }
            }
        }

        let dist = planet.dist;
        let pts: Vec<Vec3> = (0..=N_ORBIT_PTS)
            .map(|i| {
                let theta = 2.0 * PI * i as f64 / N_ORBIT_PTS as f64;
                center + Vec3::new(dist * theta.sin(), dist * theta.cos(), 0.0)
            })
            .collect();
        for i in 0..N_ORBIT_PTS {
            draw_edge_world(canvas, camera, pts[i], pts[i + 1]);
        }
    }
}
