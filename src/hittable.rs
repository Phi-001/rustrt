use super::material::*;
use super::ray::*;
use super::vector::*;
use super::Float;

pub enum Facing {
    Front,
    Back,
}

use Facing::*;

impl Default for Facing {
    fn default() -> Facing {
        Front
    }
}

#[derive(Default)]
pub struct Interaction {
    pub p: Point3,
    pub normal: Vector3,
    pub t: Float,
    pub facing: Facing,
    pub material: Option<&'static Material>,
}

impl Interaction {
    #[inline]
    pub fn set_face_normal(&mut self, ray: &Ray, outward_normal: &Vector3) {
        if Vector3::dot(&ray.direction, outward_normal) < 0.0 {
            self.facing = Front;
        } else {
            self.facing = Back;
        }
        self.normal = match self.facing {
            Front => *outward_normal,
            Back => -*outward_normal,
        }
    }
}

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
}

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
        let outward_normal = (interaction.p - self.position) / self.radius;
        interaction.set_face_normal(ray, &outward_normal);
        interaction.material = Some(self.material);

        true
    }
}

#[derive(Default)]
pub struct HittableList {
    pub objects: Vec<Hittable>,
}

impl HittableList {
    pub fn add(&mut self, object: Hittable) {
        self.objects.push(object);
    }

    pub fn hit(
        &self,
        ray: &Ray,
        t_min: Float,
        t_max: Float,
        interaction: &mut Interaction,
    ) -> bool {
        let mut temp_interaction = Interaction::default();
        let mut hit_anything = false;
        let mut closest_so_far = t_max;

        for object in self.objects.iter() {
            if object.hit(ray, t_min, closest_so_far, &mut temp_interaction) {
                hit_anything = true;
                closest_so_far = temp_interaction.t;
            }
        }

        *interaction = temp_interaction;

        hit_anything
    }
}
