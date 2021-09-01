use super::*;

pub fn ray_color(ray: &Ray, world: &HittableList, rng: &mut SmallRng, depth: usize) -> Color3 {
    let mut beta = Color3::new(1.0, 1.0, 1.0);
    let mut ray = *ray;

    ray.direction = Vector3::unit_vector(ray.direction);

    for bounces in 0..depth {
        let mut interaction = Interaction::default();
        if world.hit(&ray, 0.001, Float::INFINITY, &mut interaction) {
            let mut next_ray = Ray::default();
            let mut pdf = 0.0;
            let mut is_specular = false;
            let reflectance = interaction.material.unwrap().scatter(
                &ray,
                &mut next_ray,
                &mut pdf,
                &mut is_specular,
                &interaction,
                rng,
            );

            beta *= reflectance
                * Float::abs(Vector3::dot(&interaction.normal, &next_ray.direction))
                / pdf;

            ray = next_ray;
        } else {
            let unit_direction = Vector3::unit_vector(ray.direction);
            let t = 0.5 * (unit_direction.y + 1.0);
            let environment =
                (1.0 - t) * Color3::new(1.0, 1.0, 1.0) + t * Color3::new(0.5, 0.7, 1.0);

            return environment * beta;
        }
        if bounces > 3 {
            let q = Float::max(0.05, 1.0 - beta.luminance());
            if rng.gen::<f32>() < q {
                break;
            }
            beta /= 1.0 - q;
        }
    }

    Color3::new(0.0, 0.0, 0.0)
}
