#![warn(clippy::all)]
#![warn(rust_2018_idioms)]

#[macro_use]
extern crate lazy_static;

type Float = f32;

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use std::io::{self, Write};
use std::sync::mpsc;

mod bounds;
mod camera;
mod hittable;
mod material;
mod ray;
mod ray_color;
mod scene;
mod thread_pool;
mod transforms;
mod vector;

use camera::Camera;
use hittable::*;
use ray::Ray;
use ray_color::ray_color;
use scene::*;
use std::time::Instant;
use thread_pool::ThreadPool;
use vector::{Color3, Point3, Vector3};

const ASPECT_RATIO: Float = 16.0 / 9.0;
const WIDTH: usize = 1920;
const HEIGHT: usize = (WIDTH as Float / ASPECT_RATIO) as usize;
const SAMPLES_PER_PIXEL: usize = 1024;
const NUM_CPU: usize = 2;
const TILE_WIDTH: usize = 16;
const TILE_HEIGHT: usize = 16;
const MAX_DEPTH: usize = 16;

const FRAC_PI_4: Float = std::f64::consts::FRAC_PI_4 as Float;
const FRAC_PI_2: Float = std::f64::consts::FRAC_PI_2 as Float;
const FRAC_1_PI: Float = std::f64::consts::FRAC_1_PI as Float;

fn main() {
    let earlier = Instant::now();

    let (tx, rx) = mpsc::channel();

    let mut thread_pool = ThreadPool::new(NUM_CPU, || {
        let tx = tx.clone();
        Box::new(move |mut tile: Tile| {
            let mut small_rng = SmallRng::from_entropy();

            let width = usize::min(TILE_WIDTH, WIDTH - tile.x);
            let height = usize::min(TILE_HEIGHT, HEIGHT - tile.y);

            let scale = 1.0 / SAMPLES_PER_PIXEL as Float;

            for i in 0..width {
                for j in 0..height {
                    let mut pixel_color = Color3::default();
                    for _ in 0..SAMPLES_PER_PIXEL {
                        let x = tile.x + i;
                        let y = tile.y + j;

                        let u = (x as Float + small_rng.gen::<Float>()) / (WIDTH - 1) as Float;
                        let v = (y as Float + small_rng.gen::<Float>()) / (HEIGHT - 1) as Float;
                        let ray = CAMERA.get_ray(u, v, &mut small_rng);

                        let pixel = ray_color(&ray, &WORLD, &mut small_rng, MAX_DEPTH);

                        if pixel.is_normal() {
                            pixel_color += pixel;
                        }
                    }

                    pixel_color *= scale;
                    tile.set(i, j, pixel_color);
                }
            }

            tx.send(tile).unwrap();
        })
    });

    let div_up = |a, b| {
        if a % b == 0 {
            a / b
        } else {
            a / b + 1
        }
    };

    for i in 0..div_up(WIDTH, TILE_WIDTH) {
        for j in 0..div_up(HEIGHT, TILE_HEIGHT) {
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

    let total_tiles = div_up(WIDTH, TILE_WIDTH) * div_up(HEIGHT, TILE_HEIGHT);
    let mut remaining = total_tiles;

    for tile in rx {
        remaining -= 1;
        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
        println!("{}/{} tiles remaining...", remaining, total_tiles);
        println!(
            "Estimated {} seconds remaining...",
            ((Instant::now().duration_since(earlier).as_nanos() as f64)
                / ((total_tiles - remaining) as f64))
                * (remaining as f64)
                / 1_000_000_000f64
        );

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
        "Took {} seconds",
        Instant::now().duration_since(earlier).as_nanos() as f64 / 1_000_000_000.0
    )
}

#[inline]
fn index(i: usize, j: usize) -> usize {
    (j * WIDTH + i) * 4
}

struct Tile {
    x: usize,
    y: usize,
    buffer: Vec<Float>,
}

impl Tile {
    #[inline]
    fn index(i: usize, j: usize) -> usize {
        (j * TILE_WIDTH + i) * 4
    }

    #[inline]
    pub fn set(&mut self, i: usize, j: usize, rgb: Color3) {
        // apply gamma correction
        self.buffer[Tile::index(i, j)] = Float::sqrt(rgb[0]);
        self.buffer[Tile::index(i, j) + 1] = Float::sqrt(rgb[1]);
        self.buffer[Tile::index(i, j) + 2] = Float::sqrt(rgb[2]);
        self.buffer[Tile::index(i, j) + 3] = 1.0;
    }
}
