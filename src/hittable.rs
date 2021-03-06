use super::bounds::Bounds3;
use super::material::*;
use super::ray::*;
use super::vector::*;
use super::Float;

mod bvh;

pub use bvh::HittableList;

#[derive(Default)]
pub struct Interaction {
    pub p: Point3,
    pub normal: Vector3,
    pub t: Float,
    pub material: Option<&'static Material>,
}

#[derive(Clone)]
pub enum Hittable {
    Sphere(Sphere),
}

pub use Hittable::*;

impl Hittable {
    pub fn hit(
        &self,
        ray: &Ray,
        t_min: Float,
        t_max: Float,
        interaction: &mut Interaction,
    ) -> bool {
        match self {
            Sphere(sphere) => sphere.hit(ray, t_min, t_max, interaction),
        }
    }

    pub fn bound(&self) -> Bounds3 {
        match self {
            Sphere(sphere) => sphere.bound(),
        }
    }
}

#[derive(Clone)]
pub struct Sphere {
    pub position: Vector3,
    pub radius: Float,
    pub material: &'static Material,
}

impl Sphere {
    pub fn hit(
        &self,
        ray: &Ray,
        t_min: Float,
        t_max: Float,
        interaction: &mut Interaction,
    ) -> bool {
        let oc = ray.origin - self.position;
        let a = ray.direction.length_squared();
        let half_b = Vector3::dot(&oc, &ray.direction);
        let c = oc.length_squared() - self.radius * self.radius;

        let discriminant = half_b * half_b - a * c;
        if discriminant < 0.0 {
            return false;
        }
        let sqrt_d = Float::sqrt(discriminant);

        // Find the nearest root that lies in the acceptable range.
        let mut root = (-half_b - sqrt_d) / a;
        if root < t_min || t_max < root {
            root = (-half_b + sqrt_d) / a;
            if root < t_min || t_max < root {
                return false;
            }
        }

        interaction.t = root;
        interaction.p = ray.at(interaction.t);
        interaction.normal = (interaction.p - self.position) / self.radius;
        interaction.material = Some(self.material);

        true
    }

    pub fn bound(&self) -> Bounds3 {
        let radius = Vector3::new(
            Float::abs(self.radius),
            Float::abs(self.radius),
            Float::abs(self.radius),
        );
        Bounds3 {
            p_min: self.position - radius,
            p_max: self.position + radius,
        }
    }
}
