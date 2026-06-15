use crate::math::Vec3;
use crate::types::{Player, World};

/// Port of RadarCoords() from hud.c:211-258.
/// Returns (x_norm, y_norm) in [-1, 1] where (0,0) = directly ahead.
/// r = (1 - cos_angle) / 2 maps [ahead=0, behind=1].
fn radar_coords(player: &Player, obj_pos: Vec3) -> (f64, f64) {
    let v2 = obj_pos - player.pos;
    if v2.mag2() == 0.0 {
        return (0.0, 0.0);
    }
    let v2 = v2.normalize();

    let radar_r = player.view.dot(v2);
    let t = v2.dot(player.view);
    let v3 = player.view * (-t);
    let v4 = v3 + v2;

    let (cos, sin) = if v4.mag2() < 1e-12 {
        (1.0, 0.0)
    } else {
        let v4n = v4.normalize();
        (player.up.dot(v4n), player.right.dot(v4n))
    };

    let r = (1.0 - radar_r) / 2.0;
    (-r * sin, r * cos)
}

// ─── Character-grid radar ─────────────────────────────────────────────────────

// Radar grid: W×H characters. Center at (CX, CY). Ellipse to correct terminal
// 2:1 char aspect ratio (radius_x = RX cols, radius_y = RY rows).
const W: i32 = 21;
const H: i32 = 11;
const CX: i32 = 10;
const CY: i32 = 5;
const RX: f64 = 10.0;
const RY: f64 = 5.0;

fn in_circle(col: i32, row: i32) -> bool {
    let dx = (col - CX) as f64 / RX;
    let dy = (row - CY) as f64 / RY;
    dx * dx + dy * dy <= 1.0
}

fn on_ring(col: i32, row: i32) -> bool {
    let dx = (col - CX) as f64 / RX;
    let dy = (row - CY) as f64 / RY;
    let d2 = dx * dx + dy * dy;
    d2 >= 0.85 && d2 <= 1.0
}

fn on_half_ring(col: i32, row: i32) -> bool {
    let dx = (col - CX) as f64 / RX;
    let dy = (row - CY) as f64 / RY;
    let d2 = dx * dx + dy * dy;
    d2 >= 0.21 && d2 <= 0.28
}

/// Place a blip character on the grid, clamped to the radar circle.
fn plot(grid: &mut Vec<Vec<char>>, x_norm: f64, y_norm: f64, ch: char) {
    // Terminal y increases downward; negate y_norm.
    let col = (CX as f64 + x_norm * RX).round() as i32;
    let row = (CY as f64 - y_norm * RY).round() as i32;
    if col >= 0 && col < W && row >= 0 && row < H && in_circle(col, row) {
        grid[row as usize][col as usize] = ch;
    }
}

/// Render the radar as a Vec of strings (one per row), H rows × W cols.
/// Blip legend: `.` planet, `+` enemy, `o` friendly, `*` missile, `W` waypoint.
pub fn render_radar(world: &World) -> Vec<String> {
    let mut grid: Vec<Vec<char>> = (0..H)
        .map(|row| {
            (0..W)
                .map(|col| {
                    if on_ring(col, row) {
                        '·'
                    } else if on_half_ring(col, row) {
                        '·'
                    } else if in_circle(col, row) {
                        ' '
                    } else {
                        ' '
                    }
                })
                .collect()
        })
        .collect();

    // Centre marker.
    grid[CY as usize][CX as usize] = '⊕';

    let player = &world.player;

    // Planets — show non-moons always; moons only when primary is close.
    for (i, p) in world.planets.iter().enumerate() {
        if p.hidden || p.radius <= 0.0 { continue; }
        if p.is_moon {
            let pr = p.primary;
            if pr < world.planets.len() {
                let r2 = (world.planets[pr].pos - player.pos).mag2();
                if r2 > (world.planets[pr].radius * 1000.0).powi(2) { continue; }
            }
        }
        let _ = i;
        let (x, y) = radar_coords(player, p.pos);
        plot(&mut grid, x, y, '·');
    }

    // Targets — `+` enemy, `f` friendly. Locked target gets `X`.
    let locked = world.lock.target;
    for (i, t) in world.targets.iter().enumerate() {
        if t.age <= 0.0 || t.hidden || t.invisible { continue; }
        let (x, y) = radar_coords(player, t.pos);
        let ch = if Some(i) == locked {
            'X'
        } else if t.friendly {
            'f'
        } else {
            '+'
        };
        plot(&mut grid, x, y, ch);
    }

    // Missiles — `o`.
    for m in &world.missiles {
        if m.age <= 0.0 { continue; }
        let (x, y) = radar_coords(player, m.pos);
        plot(&mut grid, x, y, 'o');
    }

    // Current waypoint — `W`.
    let wp = world.player.waypoint;
    if wp < world.nwaypoints {
        let (x, y) = radar_coords(player, world.waypoints[wp].pos);
        plot(&mut grid, x, y, 'W');
    }

    grid.into_iter()
        .map(|row| row.into_iter().collect())
        .collect()
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn default_player() -> Player {
        Player {
            pos: Vec3::zero(),
            view: Vec3::new(1.0, 0.0, 0.0),
            up:   Vec3::new(0.0, 0.0, 1.0),
            right: Vec3::new(0.0, -1.0, 0.0),
            ..Default::default()
        }
    }

    #[test]
    fn ahead_maps_to_center() {
        let p = default_player();
        let (x, y) = radar_coords(&p, Vec3::new(10.0, 0.0, 0.0)); // directly ahead
        assert!(x.abs() < 0.01, "x should be ~0, got {x}");
        assert!(y.abs() < 0.01, "y should be ~0, got {y}");
    }

    #[test]
    fn behind_maps_to_rim() {
        let p = default_player();
        let (x, y) = radar_coords(&p, Vec3::new(-10.0, 0.0, 0.0)); // directly behind
        let r = (x * x + y * y).sqrt();
        assert!((r - 1.0).abs() < 0.01, "radius should be ~1 (rim), got {r}");
    }

    #[test]
    fn right_maps_to_left_side() {
        // Object to the right (player.right = -Y) should appear on the left of the radar
        // (SIN = dot(right, v4) = +1 → x_norm = -r*sin < 0).
        let p = default_player();
        let (x, _y) = radar_coords(&p, Vec3::new(0.0, -10.0, 0.0)); // along -Y = player right
        assert!(x < 0.0, "object to the right should be left on radar (x<0), got {x}");
    }

    #[test]
    fn above_maps_to_top() {
        // Object above player (along up = +Z) should appear at top of radar (y>0).
        let p = default_player();
        let (_x, y) = radar_coords(&p, Vec3::new(0.0, 0.0, 10.0)); // up
        assert!(y > 0.0, "object above should be at top of radar (y>0), got {y}");
    }
}
