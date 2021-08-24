use super::*;

#[allow(dead_code)]
pub enum Material {
    Diffuse(Diffuse),
    Metal(Metal),
    Dielectric(Dielectric),
}

use Material::*;

impl Material {
    pub fn scatter(
        &self,
        ray_in: &Ray,
        interaction: &Interaction,
        rng: &mut SmallRng,
        attenuation: &mut Color3,
        scattered: &mut Ray,
    ) -> bool {
        match self {
            Diffuse(material) => material.scatter(ray_in, interaction, rng, attenuation, scattered),
            Metal(material) => material.scatter(ray_in, interaction, rng, attenuation, scattered),
            Dielectric(material) => {
                material.scatter(ray_in, interaction, rng, attenuation, scattered)
            }
        }
    }
}

pub struct Diffuse {
    pub albedo: Color3,
}

impl Diffuse {
    pub fn scatter(
        &self,
        _: &Ray,
        interaction: &Interaction,
        rng: &mut SmallRng,
        attenuation: &mut Color3,
        scattered: &mut Ray,
    ) -> bool {
        let mut scatter_direction = interaction.normal + random_unit_vector(rng);

        if scatter_direction.near_zero() {
            scatter_direction = interaction.normal;
        }

        *scattered = Ray {
            origin: interaction.p,
            direction: scatter_direction,
        };

        *attenuation = self.albedo;

        true
    }
}

pub struct Metal {
    pub albedo: Color3,
    pub fuzziness: Float,
}

impl Metal {
    pub fn scatter(
        &self,
        ray_in: &Ray,
        interaction: &Interaction,
        rng: &mut SmallRng,
        attenuation: &mut Color3,
        scattered: &mut Ray,
    ) -> bool {
        let reflected =
            Vector3::reflect(&Vector3::unit_vector(ray_in.direction), &interaction.normal);
        *scattered = Ray {
            origin: interaction.p,
            direction: reflected + self.fuzziness * random_unit_vector(rng),
        };
        *attenuation = self.albedo;

        Vector3::dot(&scattered.direction, &interaction.normal) > 0.0
    }
}

pub struct Dielectric {
    pub index_of_refraction: Float,
}

impl Dielectric {
    pub fn scatter(
        &self,
        ray_in: &Ray,
        interaction: &Interaction,
        rng: &mut SmallRng,
        attenuation: &mut Color3,
        scattered: &mut Ray,
    ) -> bool {
        *attenuation = Color3::new(1.0, 1.0, 1.0);

        let refraction_ratio = match &interaction.facing {
            Facing::Front => 1.0 / self.index_of_refraction,
            Facing::Back => self.index_of_refraction,
        };

        let unit_direction = Vector3::unit_vector(ray_in.direction);
        let cos_theta = Float::min(Vector3::dot(&-unit_direction, &interaction.normal), 1.0);
        let sin_theta = Float::sqrt(1.0 - cos_theta * cos_theta);

        let cannot_refract = (refraction_ratio * sin_theta) > 1.0;

        let direction =
            if cannot_refract || Dielectric::reflectance(cos_theta, refraction_ratio) > rng.gen() {
                Vector3::reflect(&unit_direction, &interaction.normal)
            } else {
                Vector3::refract(&unit_direction, &interaction.normal, refraction_ratio)
            };

        *scattered = Ray {
            origin: interaction.p,
            direction,
        };

        true
    }

    fn reflectance(cosine: Float, refraction_index: Float) -> Float {
        let r0 = (1.0 - refraction_index) / (1.0 + refraction_index);
        let r0 = r0 * r0;
        r0 + (1.0 - r0) * Float::powi(1.0 - cosine, 5)
    }
}

fn random_unit_vector(rng: &mut SmallRng) -> Vector3 {
    let p = Vector3 {
        x: rng.gen::<Float>() * 2.0 - 1.0,
        y: rng.gen::<Float>() * 2.0 - 1.0,
        z: rng.gen::<Float>() * 2.0 - 1.0,
    };
    Vector3::unit_vector(p)
}
