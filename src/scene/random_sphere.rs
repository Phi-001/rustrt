use super::*;

use material::*;

lazy_static! {
    static ref MATERIAL_GROUND: Material = {
        let material = Diffuse {
            albedo: Color3::new(0.8, 0.8, 0.0),
        };

        Material::Diffuse(material)
    };
    static ref MATERIAL_CENTER: Material = {
        let material = Diffuse {
            albedo: Color3::new(0.1, 0.2, 0.5),
        };

        Material::Diffuse(material)
    };
    static ref MATERIAL_LEFT: Material = {
        let material = Dielectric {
            index_of_refraction: 1.5,
        };

        Material::Dielectric(material)
    };
    static ref MATERIAL_RIGHT: Material = {
        let material = Metal {
            albedo: Color3::new(0.8, 0.6, 0.2),
            fuzziness: 0.0,
        };

        Material::Metal(material)
    };
    static ref MATERIAL_LIST: Vec<Material> = {
        let mut rng = SmallRng::from_seed([126; 32]);
        // let mut rng = SmallRng::from_entropy();
        let num_sphere = 22 * 22;
        let mut vec = Vec::with_capacity(num_sphere);
        for _ in 0..num_sphere {
            let choose_mat = rng.gen::<f32>();

            vec.push(if choose_mat < 0.8 {
                let albedo = Color3::random(&mut rng) * Color3::random(&mut rng);
                Material::Diffuse(Diffuse { albedo })
            } else if choose_mat < 0.95 {
                let albedo = Color3::random(&mut rng) * 0.5 + Color3::new(0.5, 0.5, 0.5);
                let fuzziness = rng.gen::<Float>() * 0.5;
                Material::Metal(Metal { albedo, fuzziness })
            } else {
                Material::Dielectric(Dielectric {
                    index_of_refraction: 1.5,
                })
            });
        }
        vec
    };
    pub static ref WORLD: HittableList = {
        let mut world = HittableList::default();
        let mut rng = SmallRng::from_seed([123; 32]);
        // let mut rng = SmallRng::from_entropy();

        world.add(Hittable::Sphere(Sphere {
            position: Point3::new(0.0, -1000.0, 0.0),
            radius: 1000.0,
            material: &MATERIAL_GROUND,
        }));

        for a in -11..11 {
            for b in -11..11 {
                let material = ((a + 11) * 22 + (b + 11)) as usize;
                let a = a as Float;
                let b = b as Float;
                let position = Vector3::new(
                    a + 0.9 * rng.gen::<Float>(),
                    0.2,
                    b + 0.9 * rng.gen::<Float>(),
                );

                if (position - Point3::new(4.0, 0.2, 0.0)).length() > 0.9 {
                    world.add(Hittable::Sphere(Sphere {
                        position,
                        radius: 0.2,
                        material: &MATERIAL_LIST[material],
                    }));
                }
            }
        }

        world.add(Hittable::Sphere(Sphere {
            position: Point3::new(0.0, 1.0, 0.0),
            radius: 1.0,
            material: &MATERIAL_CENTER,
        }));

        world.add(Hittable::Sphere(Sphere {
            position: Point3::new(-4.0, 1.0, 0.0),
            radius: 1.0,
            material: &MATERIAL_LEFT,
        }));

        world.add(Hittable::Sphere(Sphere {
            position: Point3::new(4.0, 1.0, 0.0),
            radius: 1.0,
            material: &MATERIAL_RIGHT,
        }));

        world.init();

        world
    };
    pub static ref CAMERA: Camera = {
        let look_from = Point3::new(13.0, 2.0, 3.0);
        let look_at = Point3::new(0.0, 0.0, 0.0);
        let view_up = Vector3::new(0.0, 1.0, 0.0);
        let fov = 20.0;
        let dist_to_focus = 10.0;
        let aperture = 0.1;

        Camera::new(
            look_from,
            look_at,
            view_up,
            fov,
            ASPECT_RATIO,
            aperture,
            dist_to_focus,
        )
    };
}
