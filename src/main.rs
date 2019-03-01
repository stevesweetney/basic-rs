use image;
use std::collections::BinaryHeap;
use std::env;

const iterations: u32 = 2048;

fn main() {
    let args: Vec<String> = env::args().collect();
    let image_path = &args[1];
    let image =
        image::open(image_path).expect(&format!("Error opening target image {}\n", image_path));

    image.save("./output.png");
}
