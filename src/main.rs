use clap::{App, Arg};
use image::{self, imageops, Frame, GenericImageView};
use std::fs::File;
use std::path::Path;

use basic::Model;

const IMAGE_BOUNDS: u32 = 256;

const DEFAULT_ITERATIONS: u32 = 1024;

fn encode_frames_to_gif<P: AsRef<Path>>(name: P, frames: Vec<Frame>) {
    let file_out = File::create(name).unwrap();
    let mut encoder = image::gif::Encoder::new(file_out);
    encoder.encode_frames(frames.into_iter()).unwrap();
}

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
            Arg::with_name("gif")
                .short("g")
                .long("gif")
                .help("Create a gif"),
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
    let output_name = matches
        .value_of("output")
        .map(|output| Path::new(output))
        .unwrap();
    let iterations = matches
        .value_of("iterations")
        .and_then(|iter| iter.parse().ok())
        .unwrap();
    let pad = matches.is_present("padding");

    let image = image::open(image_path)
        .unwrap_or_else(|_| panic!("Error opening target image {}\n", image_path));

    let (width, height) = image.dimensions();

    let mut model = Model::new(image.resize(IMAGE_BOUNDS, IMAGE_BOUNDS, imageops::Nearest));

    if matches.is_present("gif") {
        let mut frames = Vec::with_capacity(iterations as usize);
        for _ in 0..iterations {
            model.split();
            frames.push(model.get_frame(width, height, pad).unwrap())
        }
        encode_frames_to_gif(output_name.with_extension("gif"), frames);
    } else {
        for _ in 0..iterations {
            model.split();
        }
    }
    model.render(output_name, width, height, pad);
}
