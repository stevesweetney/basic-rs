use image::{self, imageops, GenericImageView};
use std::env;

use basic::Model;

const IMAGE_BOUNDS: u32 = 256;

const DEFAULT_ITERATIONS: u32 = 1024;

fn main() {
    let args: Vec<String> = env::args().collect();
    let image_path = match args.get(1) {
        Some(path) => path,
        None => panic!("An image path must be specified"),
    };

    let iterations = match args.get(2) {
        Some(iters) => iters.parse().unwrap(),
        None => DEFAULT_ITERATIONS,
    };

    let image = image::open(image_path)
        .unwrap_or_else(|_| panic!("Error opening target image {}\n", image_path));

    let (width, height) = image.dimensions();

    let mut model = Model::new(image.resize(IMAGE_BOUNDS, IMAGE_BOUNDS, imageops::Nearest));

    for _ in 0..iterations {
        model.split();
    }

    model.render(width, height);
}
