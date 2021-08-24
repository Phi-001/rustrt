use super::vector::*;

pub struct Ray {
    pub origin: Point3,
    pub direction: Vector3,
}

impl Ray {
    #[inline]
    pub fn at(&self, t: f32) -> Point3 {
        self.origin + self.direction * t
    }
}
