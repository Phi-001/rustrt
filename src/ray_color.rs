use super::*;

pub fn ray_color(ray: &Ray, world: &HittableList, rng: &mut SmallRng, depth: usize) -> Color3 {
    if depth == 0 {
        return Color3::new(0.0, 0.0, 0.0);
    }

    let mut interaction = Interaction::default();
    if world.hit(ray, 0.001, Float::INFINITY, &mut interaction) {
        let mut scattered = Ray::default();
        let mut attenuation = Color3::default();
        if let Some(material) = interaction.material {
            if material.scatter(ray, &interaction, rng, &mut attenuation, &mut scattered) {
                return attenuation * ray_color(&scattered, world, rng, depth - 1);
            }
        }
        return Color3::new(0.0, 0.0, 0.0);
    }

    let unit_direction = Vector3::unit_vector(ray.direction);
    let t = 0.5 * (unit_direction.y + 1.0);
    (1.0 - t) * Color3::new(1.0, 1.0, 1.0) + t * Color3::new(0.5, 0.7, 1.0)
}
