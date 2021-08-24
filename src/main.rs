#![warn(clippy::all)]
#![warn(rust_2018_idioms)]

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use std::io::{self, Write};
use std::sync::mpsc;
use std::sync::Arc;

mod camera;
mod hittable;
mod ray;
mod thread_pool;
mod vector;

use camera::Camera;
use hittable::*;
use ray::Ray;
use std::time::Instant;
use thread_pool::ThreadPool;
use vector::{Color3, Point3, Vector3};

const ASPECT_RATIO: f32 = 16.0 / 9.0;
const WIDTH: usize = 600;
const HEIGHT: usize = (WIDTH as f32 / ASPECT_RATIO) as usize;
const SAMPLES_PER_PIXEL: usize = 16;
const NUM_CPU: usize = 4;
const TILE_WIDTH: usize = 16;
const TILE_HEIGHT: usize = 16;

fn main() {
    let earlier = Instant::now();

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

    let camera = Arc::new(Camera::new());

    let (tx, rx) = mpsc::channel();

    let mut thread_pool = ThreadPool::new(NUM_CPU, || {
        let tx = tx.clone();
        let camera = Arc::clone(&camera);
        let world = Arc::clone(&world);
        Box::new(move |mut tile: Tile| {
            let mut small_rng = SmallRng::from_entropy();

            let width = usize::min(TILE_WIDTH, WIDTH - tile.x);
            let height = usize::min(TILE_HEIGHT, HEIGHT - tile.y);

            for i in 0..width {
                for j in 0..height {
                    for _ in 0..SAMPLES_PER_PIXEL {
                        let x = tile.x + i;
                        let y = tile.y + j;

                        let u = (x as f32 + small_rng.gen::<f32>()) / (WIDTH - 1) as f32;
                        let v = (y as f32 + small_rng.gen::<f32>()) / (HEIGHT - 1) as f32;
                        let ray = camera.get_ray(u, v);

                        let pixel = ray_color(&ray, &world);

                        tile.add(i, j, pixel);
                    }
                }
            }

            let scale = 1.0 / SAMPLES_PER_PIXEL as f32;

            for ele in tile.buffer.iter_mut() {
                *ele *= scale;
            }

            tx.send(tile).unwrap();
        })
    });

    for i in 0..WIDTH / TILE_WIDTH {
        for j in 0..HEIGHT / TILE_HEIGHT {
            thread_pool.push_que(Tile {
                x: i * TILE_WIDTH,
                y: j * TILE_HEIGHT,
                buffer: vec![0.0; TILE_WIDTH * TILE_HEIGHT * 4],
            });
        }
    }

    std::thread::spawn(move || {
        // use std::time::Instant;
        // let earlier = Instant::now();
        thread_pool.execute_que();
        // println!(
        //     "{} milliseconds",
        //     Instant::now().duration_since(earlier).as_nanos() as f64 / 1_000_000.0
        // );
    });

    let mut image = vec![0u8; WIDTH * HEIGHT * 4];

    let mut remaining = (WIDTH / TILE_HEIGHT) * (HEIGHT / TILE_WIDTH);

    for tile in rx {
        remaining -= 1;
        print!("\r");
        print!("{} tiles remaining...", remaining);

        io::stdout().flush().unwrap();

        let width = usize::min(TILE_WIDTH, WIDTH - tile.x);
        let height = usize::min(TILE_HEIGHT, HEIGHT - tile.y);

        for i in 0..width {
            for j in 0..height {
                for k in 0..4 {
                    image[index(tile.x + i, tile.y + j) + k] =
                        (256.0 * tile.buffer[Tile::index(i, j) + k].clamp(0.0, 0.999)) as u8;
                }
            }
        }

        if remaining == 0 {
            break;
        }
    }

    print!("\r{esc}[K", esc = 27 as char);
    println!("Done!");

    let path = std::path::Path::new("renders/image.png");
    let prefix = path.parent().unwrap();
    std::fs::create_dir_all(prefix).unwrap();

    let image_buffer: image::ImageBuffer<image::Rgba<u8>, _> =
        image::ImageBuffer::from_raw(WIDTH as u32, HEIGHT as u32, image).unwrap();

    let dynamic_image = image::DynamicImage::ImageRgba8(image_buffer);

    let dynamic_image = dynamic_image.flipv();

    dynamic_image
        .save_with_format(path, image::ImageFormat::Png)
        .unwrap();

    println!(
        "Took {} milliseconds",
        Instant::now().duration_since(earlier).as_nanos() as f64 / 1_000_000.0
    )
}

#[inline]
fn index(i: usize, j: usize) -> usize {
    (j * WIDTH + i) * 4
}

struct Tile {
    x: usize,
    y: usize,
    buffer: Vec<f32>,
}

impl Tile {
    #[inline]
    fn index(i: usize, j: usize) -> usize {
        (j * TILE_WIDTH + i) * 4
    }

    #[inline]
    pub fn add(&mut self, i: usize, j: usize, rgb: Color3) {
        self.buffer[Tile::index(i, j)] += rgb[0];
        self.buffer[Tile::index(i, j) + 1] += rgb[1];
        self.buffer[Tile::index(i, j) + 2] += rgb[2];
        self.buffer[Tile::index(i, j) + 3] += 1.0;
    }
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
