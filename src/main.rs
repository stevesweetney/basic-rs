use clap::{App, Arg};
use image::{self, imageops, GenericImageView};

use basic::Model;

const IMAGE_BOUNDS: u32 = 256;

const DEFAULT_ITERATIONS: u32 = 1024;

fn main() {
    let default_iters = DEFAULT_ITERATIONS.to_string();
    let matches = App::new("basic")
        .version("0.1")
        .author("Steve S.")
        .about("Making images simpler")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("FILE")
                .help("Path to input image")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("FILE")
                .help("Name of the output image")
                .takes_value(true)
                .default_value("output.png"),
        )
        .arg(
            Arg::with_name("iterations")
                .long("iters")
                .help("Set the number of times the algorithm will run")
                .value_name("NUM")
                .takes_value(true)
                .default_value(&default_iters),
        )
        .arg(
            Arg::with_name("padding")
                .short("p")
                .long("padding")
                .help("Add padding to quadrants"),
        )
        .get_matches();

    let image_path = matches.value_of("input").unwrap();
    let output_name = matches.value_of("output").unwrap();
    let iterations = matches
        .value_of("iterations")
        .and_then(|iter| iter.parse().ok())
        .unwrap();
    let pad = matches.is_present("padding");

    let image = image::open(image_path)
        .unwrap_or_else(|_| panic!("Error opening target image {}\n", image_path));

    let (width, height) = image.dimensions();

    let mut model = Model::new(image.resize(IMAGE_BOUNDS, IMAGE_BOUNDS, imageops::Nearest));

    for _ in 0..iterations {
        model.split();
    }

    model.render(output_name, width, height, pad);
}
