use super::*;

#[derive(Clone)]
pub struct Camera {
    origin: Point3,
    lower_left_corner: Point3,
    horizontal: Vector3,
    vertical: Vector3,
    u: Vector3,
    v: Vector3,
    w: Vector3,
    lens_radius: Float,
}

impl Camera {
    pub fn new(
        look_from: Point3,
        look_at: Point3,
        view_up: Vector3,
        fov: Float,
        aspect_ratio: Float,
        aperture: Float,
        focus_dist: Float,
    ) -> Self {
        let theta = degrees_to_radians(fov);
        let h = Float::tan(theta / 2.0);
        let viewport_height = 2.0 * h;
        let viewport_width = aspect_ratio * viewport_height;

        let w = Vector3::unit_vector(look_from - look_at);
        let u = Vector3::unit_vector(Vector3::cross(&view_up, &w));
        let v = Vector3::cross(&w, &u);

        let origin = look_from;
        let horizontal = focus_dist * viewport_width * u;
        let vertical = focus_dist * viewport_height * v;
        let lower_left_corner = origin - horizontal / 2.0 - vertical / 2.0 - focus_dist * w;

        Camera {
            origin,
            horizontal,
            vertical,
            lower_left_corner,
            w,
            u,
            v,
            lens_radius: aperture / 2.0,
        }
    }

    pub fn get_ray(&self, s: Float, t: Float, rng: &mut SmallRng) -> Ray {
        let rd = self.lens_radius * random_in_unit_disk(rng);
        let offset = self.u * rd.x + self.v * rd.y;

        Ray {
            origin: self.origin + offset,
            direction: self.lower_left_corner + s * self.horizontal + t * self.vertical
                - self.origin
                - offset,
        }
    }
}

static PI: Float = std::f64::consts::PI as Float;

fn random_in_unit_disk(rng: &mut SmallRng) -> Vector3 {
    let r = Float::sqrt(rng.gen());
    let theta = 2.0 * PI * rng.gen::<Float>();
    Vector3::new(r * Float::cos(theta), r * Float::sin(theta), 0.0)
}

fn degrees_to_radians(degrees: Float) -> Float {
    degrees * (PI / 180.0)
}
