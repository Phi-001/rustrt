use super::*;
use std::ops::*;

#[derive(Clone, Copy, Debug, Default)]
pub struct Vector3 {
    pub x: Float,
    pub y: Float,
    pub z: Float,
}

impl Vector3 {
    #[inline]
    pub fn dot(u: &Vector3, v: &Vector3) -> Float {
        v.x * u.x + v.y * u.y + v.z * u.z
    }

    #[inline]
    pub fn cross(u: &Vector3, v: &Vector3) -> Vector3 {
        Vector3 {
            x: u.y * v.z - u.z * v.y,
            y: u.z * v.x - u.x * v.z,
            z: u.x * v.y - u.y * v.x,
        }
    }

    #[inline]
    pub fn unit_vector(v: Vector3) -> Vector3 {
        v / v.length()
    }

    #[inline]
    pub fn length_squared(&self) -> Float {
        Vector3::dot(self, self)
    }

    #[inline]
    pub fn length(&self) -> Float {
        Float::sqrt(self.length_squared())
    }

    #[inline]
    pub fn near_zero(&self) -> bool {
        static EPSILON: Float = 1e-8;

        Float::abs(self.x) < EPSILON && Float::abs(self.y) < EPSILON && Float::abs(self.z) < EPSILON
    }

    #[inline]
    pub fn new(x: Float, y: Float, z: Float) -> Vector3 {
        Vector3 { x, y, z }
    }

    #[inline]
    pub fn reflect(v: &Vector3, n: &Vector3) -> Vector3 {
        *v - 2.0 * Vector3::dot(v, n) * *n
    }

    #[inline]
    pub fn refract(uv: &Vector3, n: &Vector3, eta_i_over_eta_t: Float) -> Vector3 {
        let cos_theta = Float::min(Vector3::dot(&-*uv, n), 1.0);
        let r_out_perpendicular = eta_i_over_eta_t * (*uv + cos_theta * *n);
        let r_out_parallel =
            -Float::sqrt(Float::abs(1.0 - r_out_perpendicular.length_squared())) * *n;
        r_out_perpendicular + r_out_parallel
    }

    #[inline]
    pub fn random(rng: &mut SmallRng) -> Vector3 {
        Vector3::new(rng.gen(), rng.gen(), rng.gen())
    }
}

impl Add for Vector3 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Vector3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Sub for Vector3 {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Vector3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Mul<Vector3> for Vector3 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Vector3) -> Self::Output {
        Vector3 {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}

impl Mul<Float> for Vector3 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Float) -> Self::Output {
        Vector3 {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl Mul<Vector3> for Float {
    type Output = Vector3;

    #[inline]
    fn mul(self, rhs: Vector3) -> Self::Output {
        rhs * self
    }
}

impl Div<Float> for Vector3 {
    type Output = Self;

    #[inline]
    fn div(self, rhs: Float) -> Self::Output {
        self * (1.0 / rhs)
    }
}

impl AddAssign for Vector3 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl SubAssign for Vector3 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs
    }
}

impl MulAssign<Float> for Vector3 {
    #[inline]
    fn mul_assign(&mut self, rhs: Float) {
        *self = *self * rhs
    }
}

impl DivAssign<Float> for Vector3 {
    #[inline]
    fn div_assign(&mut self, rhs: Float) {
        *self = *self / rhs
    }
}

impl Neg for Vector3 {
    type Output = Vector3;

    #[inline]
    fn neg(self) -> Self::Output {
        Vector3 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl Index<usize> for Vector3 {
    type Output = Float;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("Vector3 can only be indexed by 0, 1, or 2"),
        }
    }
}

impl IndexMut<usize> for Vector3 {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            _ => panic!("Vector3 can only be indexed by 0, 1, or 2"),
        }
    }
}

pub type Color3 = Vector3;

impl Color3 {
    #[inline]
    pub fn to_rgb(self, samples_per_pixel: usize) -> [u8; 3] {
        let scale = 1.0 / samples_per_pixel as Float;
        let r = self[0] * scale;
        let g = self[1] * scale;
        let b = self[2] * scale;
        [
            (256.0 * Float::clamp(r, 0.0, 0.999)) as u8,
            (256.0 * Float::clamp(g, 0.0, 0.999)) as u8,
            (256.0 * Float::clamp(b, 0.0, 0.999)) as u8,
        ]
    }
}

pub type Point3 = Vector3;
