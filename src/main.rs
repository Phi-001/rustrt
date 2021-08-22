#![warn(clippy::all)]
#![warn(rust_2018_idioms)]

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use std::io::{self, Write};
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;

mod camera;
mod hittable;
mod ray;
mod render;
mod vector;

use camera::Camera;
use hittable::*;
use ray::Ray;
use vector::{Color3, Point3, Vector3};

const ASPECT_RATIO: f32 = 16.0 / 9.0;
const WIDTH: usize = 600;
const HEIGHT: usize = (WIDTH as f32 / ASPECT_RATIO) as usize;
const SAMPLES_PER_PIXEL: usize = 16; // Has to be multiple of NUM_CPU to avoid it do bad thing
const NUM_CPU: usize = 4; // I have 4 but I don't want my PC freezing while doing other stuff

fn main() {
    let mut world = HittableList::default();
    world.add(Hittable::Sphere(Sphere {
        position: Point3::new(0.0, 0.0, -1.0),
        radius: 0.5,
    }));
    world.add(Hittable::Sphere(Sphere {
        position: Point3::new(0.0, -100.4, -1.0),
        radius: 100.0,
    }));

    let world = Arc::new(world);

    let camera = Camera::new();

    let image_mutex = Arc::new(Mutex::new([0u8; WIDTH * HEIGHT * 4]));

    let thread_image = Arc::clone(&image_mutex);

    thread::spawn(move || {
        let earlier = std::time::Instant::now();

        let (tx, rx) = channel();

        for id in 0..NUM_CPU {
            let camera = camera.clone();
            let world = Arc::clone(&world);
            let tx = tx.clone();
            thread::spawn(move || {
                let mut image = vec![0f32; WIDTH * HEIGHT * 4];

                let mut small_rng = SmallRng::from_entropy();

                for i in 0..SAMPLES_PER_PIXEL / NUM_CPU {
                    for j in 0..HEIGHT {
                        for i in 0..WIDTH {
                            let u = (i as f32 + small_rng.gen::<f32>()) / (WIDTH - 1) as f32;
                            let v = (j as f32 + small_rng.gen::<f32>()) / (HEIGHT - 1) as f32;
                            let ray = camera.get_ray(u, v);

                            let pixel = ray_color(&ray, &world);

                            add(&mut *image, i, j, pixel);
                        }
                    }

                    let scale = 1.0 / (i + 1) as f32;

                    tx.send((
                        image
                            .iter()
                            .map(|col| (f32::clamp(*col * scale, 0.0, 0.999)))
                            .collect::<Vec<f32>>(),
                        id,
                    ))
                    .unwrap();
                }
            });
        }

        drop(tx);

        let mut images = vec![vec![0f32; WIDTH * HEIGHT * 4]; NUM_CPU];

        let mut remaining = SAMPLES_PER_PIXEL;

        for message in rx {
            remaining -= 1;

            print!("\r");
            print!("{} iterations remaining...", remaining);

            io::stdout().flush().unwrap();

            let (image, id) = message;
            images[id] = image;

            if remaining % NUM_CPU != 0 {
                continue;
            }

            let scale = 256.0 / NUM_CPU as f32;

            for (i, element) in thread_image.lock().unwrap().iter_mut().enumerate() {
                let mut val = 0.0;
                for image in images.iter() {
                    val += image[i];
                }

                *element = (val * scale) as u8;
            }
        }

        println!("\rDone!                     ");
        println!(
            "Took {} ms",
            std::time::Instant::now().duration_since(earlier).as_nanos() as f64 / 1_000_000f64
        );

        io::stdout().flush().unwrap();
    });

    render::render(image_mutex, WIDTH as u32, HEIGHT as u32);
}

#[inline]
fn index(i: usize, j: usize) -> usize {
    (j * WIDTH + i) * 4
}

#[inline]
fn add(image: &mut [f32], i: usize, j: usize, rgb: Color3) {
    image[index(i, j)] += rgb[0];
    image[index(i, j) + 1] += rgb[1];
    image[index(i, j) + 2] += rgb[2];
    image[index(i, j) + 3] = 1.0;
}

fn ray_color(ray: &Ray, world: &HittableList) -> Color3 {
    let mut interaction = Interaction::default();
    if world.hit(ray, 0.0, f32::INFINITY, &mut interaction) {
        return 0.5 * (interaction.normal + Color3::new(1.0, 1.0, 1.0));
    }

    let unit_direction = Vector3::unit_vector(ray.direction);
    let t = 0.5 * (unit_direction.y + 1.0);
    (1.0 - t) * Color3::new(1.0, 1.0, 1.0) + t * Color3::new(0.5, 0.7, 1.0)
}
