use super::vector::*;
use super::Float;

#[derive(Default)]
pub struct Ray {
    pub origin: Point3,
    pub direction: Vector3,
}

impl Ray {
    #[inline]
    pub fn at(&self, t: Float) -> Point3 {
        self.origin + self.direction * t
    }
}
