/// 3D vector. All game-world coordinates use f64 to match the original C.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    /// Squared magnitude (avoids sqrt — use for distance comparisons).
    pub fn mag2(self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// Magnitude (Mag in util.c).
    pub fn mag(self) -> f64 {
        self.mag2().sqrt()
    }

    /// Dot product (Dotp in util.c).
    pub fn dot(self, rhs: Self) -> f64 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    /// Cross product (Crossp in util.c). Order: self × rhs.
    pub fn cross(self, rhs: Self) -> Self {
        Self {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }

    /// Normalize in place (Normalize in util.c).
    pub fn normalize(self) -> Self {
        self / self.mag()
    }

    /// Squared distance between two points (Dist2 in util.c).
    pub fn dist2(self, other: Self) -> f64 {
        (self - other).mag2()
    }

    /// Find a vector perpendicular to self (Perp in util.c).
    /// Ported verbatim — preserves the exact case logic from util.c.
    pub fn perp(self) -> Self {
        let (x, y, z) = (self.x, self.y, self.z);
        if x != 0.0 && y != 0.0 {
            Self::new(-1.0 / x, 1.0 / y, 0.0)
        } else if y != 0.0 && z != 0.0 {
            Self::new(0.0, -1.0 / y, 1.0 / z)
        } else if x != 0.0 && z != 0.0 {
            Self::new(-1.0 / x, 0.0, 1.0 / z)
        } else if x == 0.0 && y == 0.0 && z != 0.0 {
            Self::new(1.0, 0.0, 0.0)
        } else if x != 0.0 && y == 0.0 && z == 0.0 {
            Self::new(0.0, 1.0, 0.0)
        } else if x == 0.0 && y != 0.0 && z == 0.0 {
            Self::new(1.0, 0.0, 0.0)
        } else {
            Self::new(1.0, 0.0, 0.0)
        }
    }
}

impl std::ops::Add for Vec3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl std::ops::Sub for Vec3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl std::ops::Mul<f64> for Vec3 {
    type Output = Self;
    fn mul(self, s: f64) -> Self {
        Self::new(self.x * s, self.y * s, self.z * s)
    }
}

impl std::ops::Div<f64> for Vec3 {
    type Output = Self;
    fn div(self, s: f64) -> Self {
        Self::new(self.x / s, self.y / s, self.z / s)
    }
}

impl std::ops::Neg for Vec3 {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y, -self.z)
    }
}

/// Rotate vector `v` about axis `n` by `theta` radians, returning the result.
/// Direct port of RotateAbout() from util.c (Rodrigues rotation formula, expanded).
pub fn rotate_about(v: Vec3, n: Vec3, theta: f64) -> Vec3 {
    let (big_x, big_y, big_z) = (v.x, v.y, v.z);
    let (x, y, z) = (n.x, n.y, n.z);

    let sintheta = theta.sin();
    let costheta = theta.cos();

    let t1 = x * x;
    let t2 = y * y;
    let t3 = z * z;

    let t4 = 1.0 - t1;
    let t5 = 1.0 - t2;
    let t6 = 1.0 - t3;

    let t7 = 1.0 - costheta;

    let t8 = x * y;
    let t9 = x * z;
    let t10 = y * z;

    let t11 = x * sintheta;
    let t12 = y * sintheta;
    let t13 = z * sintheta;

    let t14 = t4 * costheta;
    let t15 = t5 * costheta;
    let t16 = t6 * costheta;

    let t17 = t8 * t7;
    let t18 = t9 * t7;
    let t19 = t10 * t7;

    let a = t1 + t14;
    let b = t17 + t13;
    let c = t18 - t12;

    let d = t17 - t13;
    let e = t2 + t15;
    let f = t19 + t11;

    let g = t18 + t12;
    let h = t19 - t11;
    let i = t3 + t16;

    Vec3::new(
        big_x * a + big_y * b + big_z * c,
        big_x * d + big_y * e + big_z * f,
        big_x * g + big_y * h + big_z * i,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-10;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < EPS
    }

    fn vec_approx_eq(a: Vec3, b: Vec3) -> bool {
        approx_eq(a.x, b.x) && approx_eq(a.y, b.y) && approx_eq(a.z, b.z)
    }

    #[test]
    fn mag_unit_vectors() {
        assert!(approx_eq(Vec3::new(1.0, 0.0, 0.0).mag(), 1.0));
        assert!(approx_eq(Vec3::new(0.0, 1.0, 0.0).mag(), 1.0));
        assert!(approx_eq(Vec3::new(0.0, 0.0, 1.0).mag(), 1.0));
        assert!(approx_eq(Vec3::new(3.0, 4.0, 0.0).mag(), 5.0));
    }

    #[test]
    fn dot_product() {
        let a = Vec3::new(1.0, 0.0, 0.0);
        let b = Vec3::new(0.0, 1.0, 0.0);
        assert!(approx_eq(a.dot(b), 0.0));
        assert!(approx_eq(a.dot(a), 1.0));
        assert!(approx_eq(
            Vec3::new(1.0, 2.0, 3.0).dot(Vec3::new(4.0, 5.0, 6.0)),
            32.0
        ));
    }

    #[test]
    fn cross_product() {
        let x = Vec3::new(1.0, 0.0, 0.0);
        let y = Vec3::new(0.0, 1.0, 0.0);
        let z = Vec3::new(0.0, 0.0, 1.0);
        assert!(vec_approx_eq(x.cross(y), z));
        assert!(vec_approx_eq(y.cross(z), x));
        assert!(vec_approx_eq(z.cross(x), y));
        // Anti-commutative
        assert!(vec_approx_eq(y.cross(x), -z));
    }

    #[test]
    fn normalize_preserves_direction() {
        let v = Vec3::new(3.0, 4.0, 0.0);
        let n = v.normalize();
        assert!(approx_eq(n.mag(), 1.0));
        assert!(approx_eq(n.x, 0.6));
        assert!(approx_eq(n.y, 0.8));
    }

    #[test]
    fn dist2() {
        let a = Vec3::new(0.0, 0.0, 0.0);
        let b = Vec3::new(3.0, 4.0, 0.0);
        assert!(approx_eq(a.dist2(b), 25.0));
    }

    // rotate_about uses the CLOCKWISE convention from the original C (not right-hand rule).
    // Positive theta rotates CW when viewed from the tip of the axis toward the origin.
    // Rotating +X 90° CW around +Z (viewed from above) → -Y.
    #[test]
    fn rotate_about_x_around_z_90deg() {
        let v = Vec3::new(1.0, 0.0, 0.0);
        let axis = Vec3::new(0.0, 0.0, 1.0);
        let result = rotate_about(v, axis, std::f64::consts::FRAC_PI_2);
        assert!(vec_approx_eq(result, Vec3::new(0.0, -1.0, 0.0)));
    }

    // Rotating +Y 90° CW around +X (viewed from right) → -Z.
    #[test]
    fn rotate_about_y_around_x_90deg() {
        let v = Vec3::new(0.0, 1.0, 0.0);
        let axis = Vec3::new(1.0, 0.0, 0.0);
        let result = rotate_about(v, axis, std::f64::consts::FRAC_PI_2);
        assert!(vec_approx_eq(result, Vec3::new(0.0, 0.0, -1.0)));
    }

    // rotate_about: full 360° rotation returns original vector
    #[test]
    fn rotate_about_full_circle() {
        let v = Vec3::new(1.0, 2.0, 3.0).normalize();
        let axis = Vec3::new(0.0, 1.0, 0.0);
        let result = rotate_about(v, axis, 2.0 * std::f64::consts::PI);
        assert!(vec_approx_eq(result, v));
    }

    // rotate_about: rotation preserves vector magnitude
    #[test]
    fn rotate_about_preserves_magnitude() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        let axis = Vec3::new(1.0, 1.0, 1.0).normalize();
        let result = rotate_about(v, axis, 1.23456);
        assert!(approx_eq(result.mag(), v.mag()));
    }

    // rotate_about: rotating a vector parallel to axis leaves it unchanged
    #[test]
    fn rotate_about_parallel_unchanged() {
        let axis = Vec3::new(0.0, 0.0, 1.0);
        let v = Vec3::new(0.0, 0.0, 5.0);
        let result = rotate_about(v, axis, 1.0);
        assert!(vec_approx_eq(result, v));
    }

    #[test]
    fn perp_is_perpendicular() {
        let vectors = [
            Vec3::new(1.0, 2.0, 3.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, 1.0, 1.0),
        ];
        for v in vectors {
            let p = v.perp();
            assert!(
                approx_eq(v.dot(p), 0.0),
                "perp({:?}) = {:?} is not perpendicular",
                v,
                p
            );
        }
    }

    #[test]
    fn arithmetic_ops() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(4.0, 5.0, 6.0);
        assert!(vec_approx_eq(a + b, Vec3::new(5.0, 7.0, 9.0)));
        assert!(vec_approx_eq(b - a, Vec3::new(3.0, 3.0, 3.0)));
        assert!(vec_approx_eq(a * 2.0, Vec3::new(2.0, 4.0, 6.0)));
        assert!(vec_approx_eq(b / 2.0, Vec3::new(2.0, 2.5, 3.0)));
        assert!(vec_approx_eq(-a, Vec3::new(-1.0, -2.0, -3.0)));
    }
}
