use super::*;

pub enum ReflectanceModel {
    Diffuse(Diffuse),
    Metal(Metal),
    Dielectric(Dielectric),
}

impl ReflectanceModelTrait for ReflectanceModel {
    fn scatter(
        &self,
        dir_in: &Vector3,
        dir_out: &mut Vector3,
        pdf: &mut Float,
        is_specular: &mut bool,
        rng: &mut SmallRng,
    ) -> Color3 {
        use ReflectanceModel::*;

        match self {
            Diffuse(material) => material.scatter(dir_in, dir_out, pdf, is_specular, rng),
            Metal(material) => material.scatter(dir_in, dir_out, pdf, is_specular, rng),
            Dielectric(material) => material.scatter(dir_in, dir_out, pdf, is_specular, rng),
        }
    }

    fn reflectance(&self, dir_in: &Vector3, dir_out: &Vector3) -> Color3 {
        use ReflectanceModel::*;

        match self {
            Diffuse(material) => material.reflectance(dir_in, dir_out),
            Metal(material) => material.reflectance(dir_in, dir_out),
            Dielectric(material) => material.reflectance(dir_in, dir_out),
        }
    }

    fn pdf(&self, dir_in: &Vector3, dir_out: &Vector3) -> Float {
        use ReflectanceModel::*;

        match self {
            Diffuse(material) => material.pdf(dir_in, dir_out),
            Metal(material) => material.pdf(dir_in, dir_out),
            Dielectric(material) => material.pdf(dir_in, dir_out),
        }
    }
}

struct Basis {
    u: Vector3,
    v: Vector3,
    w: Vector3,
}

impl Basis {
    fn to_local(&self, v: &Vector3) -> Vector3 {
        Vector3::new(
            Vector3::dot(v, &self.u),
            Vector3::dot(v, &self.v),
            Vector3::dot(v, &self.w),
        )
    }

    fn from_local(&self, v: &Vector3) -> Vector3 {
        v.x * self.u + v.y * self.v + v.z * self.w
    }

    fn from_normal(normal: &Vector3) -> Basis {
        let w = Vector3::unit_vector(*normal);
        let a = if Float::abs(w.x) > 0.9 {
            Vector3::new(0.0, 1.0, 0.0)
        } else {
            Vector3::new(1.0, 0.0, 0.0)
        };

        let v = Vector3::unit_vector(Vector3::cross(&w, &a));
        let u = Vector3::cross(&w, &v);

        Basis { w, u, v }
    }
}

pub struct Material {
    pub reflectance_model: ReflectanceModel,
}

impl Material {
    /// `ray_in.direction` should be normalized.
    /// As well as `interaction.normal`
    pub fn scatter(
        &self,
        ray_in: &Ray,
        ray_out: &mut Ray,
        pdf: &mut Float,
        is_specular: &mut bool,
        interaction: &Interaction,
        rng: &mut SmallRng,
    ) -> Color3 {
        let basis = Basis::from_normal(&interaction.normal);

        let mut dir_out = Vector3::default();

        let dir_in = -basis.to_local(&ray_in.direction);

        let f = self
            .reflectance_model
            .scatter(&dir_in, &mut dir_out, pdf, is_specular, rng);

        *ray_out = Ray {
            origin: interaction.p,
            direction: basis.from_local(&dir_out),
        };

        f
    }

    #[allow(dead_code)]
    /// `ray_in.direction` should be normalized.
    /// As well as `interaction.normal`
    pub fn reflectance(
        &self,
        ray_in: &Ray,
        ray_out: &mut Ray,
        interaction: &Interaction,
    ) -> Color3 {
        let basis = Basis::from_normal(&interaction.normal);

        let dir_in = -basis.to_local(&ray_in.direction);
        let dir_out = basis.to_local(&ray_out.direction);

        self.reflectance_model.reflectance(&dir_in, &dir_out)
    }
}

impl From<ReflectanceModel> for Material {
    fn from(reflectance_model: ReflectanceModel) -> Material {
        Material { reflectance_model }
    }
}

trait ReflectanceModelTrait {
    /// Scatters the dir_in
    /// Can also be assumed that `dir_in` and `dir_out` is in coordinate where the normal is pointing in the +z
    fn scatter(
        &self,
        dir_in: &Vector3,
        dir_out: &mut Vector3,
        pdf: &mut Float,
        is_specular: &mut bool,
        rng: &mut SmallRng,
    ) -> Color3 {
        *dir_out = sample_cosine_hemisphere(rng);

        if dir_in.z < 0.0 {
            dir_out.z *= -1.0;
        }

        *pdf = self.pdf(dir_in, dir_out);
        *is_specular = false;

        self.reflectance(dir_in, dir_out)
    }

    /// reflectance
    /// it can be assumed that `dir_in` and `dir_out` is in coordinate where the normal is pointing in the +z direction
    fn reflectance(&self, dir_in: &Vector3, dir_out: &Vector3) -> Color3;

    /// PDF of the distribution generated by `scatter`
    fn pdf(&self, dir_in: &Vector3, dir_out: &Vector3) -> Float {
        if dir_in.z * dir_out.z > 0.0 {
            Float::abs(dir_out.z) * FRAC_1_PI
        } else {
            0.0
        }
    }
}

pub struct Diffuse {
    pub albedo: Color3,
}

impl ReflectanceModelTrait for Diffuse {
    fn reflectance(&self, _: &Vector3, _: &Vector3) -> Color3 {
        self.albedo * FRAC_1_PI
    }
}

pub struct Metal {
    pub albedo: Color3,
    pub fuzziness: Float,
}

impl ReflectanceModelTrait for Metal {
    fn scatter(
        &self,
        dir_in: &Vector3,
        dir_out: &mut Vector3,
        pdf: &mut Float,
        is_specular: &mut bool,
        _: &mut SmallRng,
    ) -> Color3 {
        *dir_out = Vector3::new(-dir_in.x, -dir_in.y, dir_in.z);
        *pdf = 1.0;
        *is_specular = true;

        // idk maybe I should use actual fresnel
        // I don't even care rn
        self.albedo / Float::abs(dir_out.z)
    }

    fn reflectance(&self, _: &Vector3, _: &Vector3) -> Color3 {
        Color3::new(0.0, 0.0, 0.0)
    }

    fn pdf(&self, _: &Vector3, _: &Vector3) -> Float {
        0.0
    }
}

pub struct Dielectric {
    pub index_of_refraction: Float,
}

impl ReflectanceModelTrait for Dielectric {
    fn scatter(
        &self,
        dir_in: &Vector3,
        dir_out: &mut Vector3,
        pdf: &mut Float,
        is_specular: &mut bool,
        _: &mut SmallRng,
    ) -> Color3 {
        let refraction_ratio = match dir_in.z > 0.0 {
            true => 1.0 / self.index_of_refraction,
            false => self.index_of_refraction,
        };

        let cos_theta = Float::abs(Float::min(dir_in.z, 1.0));
        let sin_theta = Float::sqrt(1.0 - cos_theta * cos_theta);

        let cannot_refract = (refraction_ratio * sin_theta) > 1.0;

        *dir_out = if cannot_refract {
            Vector3::new(-dir_in.x, -dir_in.y, dir_in.z)
        } else {
            Vector3::refract(
                &-*dir_in,
                &Vector3::flip(&Vector3::new(0.0, 0.0, 1.0), dir_in),
                refraction_ratio,
            )
        };
        *pdf = 1.0;
        *is_specular = true;

        Color3::new(1.0, 1.0, 1.0) / Float::abs(dir_out.z)
    }

    fn reflectance(&self, _: &Vector3, _: &Vector3) -> Color3 {
        Color3::new(0.0, 0.0, 0.0)
    }

    fn pdf(&self, _: &Vector3, _: &Vector3) -> Float {
        0.0
    }
}

fn sample_disk(rng: &mut SmallRng) -> Point3 {
    let offset_x = 2.0 * rng.gen::<f32>() - 1.0;
    let offset_y = 2.0 * rng.gen::<f32>() - 1.0;

    if offset_x == 0.0 && offset_y == 0.0 {
        return Point3::new(0.0, 0.0, 0.0);
    }

    let (r, theta) = if Float::abs(offset_x) > Float::abs(offset_y) {
        (offset_x, FRAC_PI_4 * (offset_y / offset_x))
    } else {
        (offset_y, FRAC_PI_2 - FRAC_PI_4 * (offset_x / offset_y))
    };

    r * Point3::new(Float::cos(theta), Float::sin(theta), 0.0)
}

fn sample_cosine_hemisphere(rng: &mut SmallRng) -> Vector3 {
    let d = sample_disk(rng);
    let z = Float::sqrt(Float::max(0.0, 1.0 - d.x * d.x - d.y * d.y));
    Vector3 { x: d.x, y: d.y, z }
}
