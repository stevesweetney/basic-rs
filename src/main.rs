use image::{self, DynamicImage, GenericImageView, Pixel};
use std::collections::BinaryHeap;
use std::env;

const iterations: u32 = 2048;

type Color = (u32, u32, u32);


fn averge_color_from_image(image: &DynamicImage) -> (Color, f32) {
    let mut histogram = [0; 768];
    for p in image.pixels() {
        let pix = p.2.to_rgb();
        let channels = pix.channels();

        let red = channels[0] as usize;
        let green = channels[1] as usize;
        let blue = channels[2] as usize;

        histogram[red] += 1;
        histogram[green + 256] += 1;
        histogram[blue + 512] += 1;
    }

    let (red, re) = weighted_average(&histogram[0..=255]);
    let (blue, be) = weighted_average(&histogram[256..=511]);
    let (green, ge) = weighted_average(&histogram[512..=767]);

    ((red, green, blue), re * 0.3 + ge * 0.6 + be + 0.1)
}

fn weighted_average(histogram: &[u32]) -> (u32, f32) {
    let mut weighted_sum = 0;
    let mut total = 0;
    for (idx, c) in histogram.iter().enumerate() {
        weighted_sum += (idx as u32) * c;
        total += c;
    }
    let value = weighted_sum / total;
    // root mean square error
    let mut error = 0;
    for (idx, c) in histogram.iter().enumerate() {
        error += c * (value - idx as u32).pow(2);
    }

    (value, ((error / total) as f32).sqrt())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let image_path = &args[1];
    let image =
        image::open(image_path).expect(&format!("Error opening target image {}\n", image_path));

    image.save("./output.png");
}
