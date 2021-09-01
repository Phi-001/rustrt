use super::ray::*;
use super::vector::*;
use super::Float;
use std::ops::Index;

#[derive(Copy, Clone, Debug)]
pub struct Bounds3 {
    pub p_min: Point3,
    pub p_max: Point3,
}

impl Default for Bounds3 {
    fn default() -> Bounds3 {
        Bounds3 {
            p_min: Point3::new(Float::MAX, Float::MAX, Float::MAX),
            p_max: Point3::new(-Float::MAX, -Float::MAX, -Float::MAX),
        }
    }
}

impl Index<bool> for Bounds3 {
    type Output = Point3;

    fn index(&self, index: bool) -> &Self::Output {
        match index {
            false => &self.p_min,
            true => &self.p_max,
        }
    }
}

impl Bounds3 {
    pub fn center(&self) -> Point3 {
        self.p_min * 0.5 + self.p_max * 0.5
    }

    pub fn offset(&self, p: &Point3) -> Vector3 {
        let mut o = *p - self.p_min;
        if self.p_max.x > self.p_min.x {
            o.x /= self.p_max.x - self.p_min.x;
        }
        if self.p_max.y > self.p_min.y {
            o.y /= self.p_max.y - self.p_min.y;
        }
        if self.p_max.z > self.p_min.z {
            o.z /= self.p_max.z - self.p_min.z;
        }
        o
    }

    pub fn intersect(
        &self,
        ray: &Ray,
        inv_dir: &Vector3,
        dir_is_neg: &[bool; 3],
        t_max_in: Float,
        t_min_in: Float,
    ) -> bool {
        let t_min = (self[dir_is_neg[0]].x - ray.origin.x) * inv_dir.x;
        let t_max = (self[!dir_is_neg[0]].x - ray.origin.x) * inv_dir.x;
        let ty_min = (self[dir_is_neg[1]].y - ray.origin.y) * inv_dir.y;
        let ty_max = (self[!dir_is_neg[1]].y - ray.origin.y) * inv_dir.y;
        let tz_min = (self[dir_is_neg[2]].z - ray.origin.z) * inv_dir.z;
        let tz_max = (self[!dir_is_neg[2]].z - ray.origin.z) * inv_dir.z;

        let t_min = Float::max(t_min, ty_min);
        let t_max = Float::min(t_max, ty_max);

        let t_min = Float::max(t_min, tz_min);
        let t_max = Float::min(t_max, tz_max);

        (t_min <= t_max) && (t_min < t_max_in) && (t_max > t_min_in)
    }

    pub fn surface_area(&self) -> Float {
        let diagonal = self.diagonal();
        2.0 * diagonal.x * diagonal.y
            + 2.0 * diagonal.x * diagonal.z
            + 2.0 * diagonal.y * diagonal.z
    }

    pub fn union(b1: &Bounds3, b2: &Bounds3) -> Bounds3 {
        Bounds3 {
            p_min: Point3 {
                x: Float::min(b1.p_min.x, b2.p_min.x),
                y: Float::min(b1.p_min.y, b2.p_min.y),
                z: Float::min(b1.p_min.z, b2.p_min.z),
            },
            p_max: Point3 {
                x: Float::max(b1.p_max.x, b2.p_max.x),
                y: Float::max(b1.p_max.y, b2.p_max.y),
                z: Float::max(b1.p_max.z, b2.p_max.z),
            },
        }
    }

    pub fn union_point(b: &Bounds3, p: &Point3) -> Bounds3 {
        Bounds3 {
            p_min: Point3 {
                x: Float::min(b.p_min.x, p.x),
                y: Float::min(b.p_min.y, p.y),
                z: Float::min(b.p_min.z, p.z),
            },
            p_max: Point3 {
                x: Float::max(b.p_max.x, p.x),
                y: Float::max(b.p_max.y, p.y),
                z: Float::max(b.p_max.z, p.z),
            },
        }
    }

    pub fn diagonal(&self) -> Vector3 {
        self.p_max - self.p_min
    }

    pub fn maximum_extent(&self) -> usize {
        let diagonal = self.diagonal();
        if diagonal.x > diagonal.y && diagonal.x > diagonal.z {
            0
        } else if diagonal.y > diagonal.z {
            1
        } else {
            2
        }
    }
}
