use clap::{App, Arg};
use gifski::{self, progress::NoProgress, Settings};
use image::{self, imageops, GenericImageView};
use std::fs::File;
use std::sync::mpsc;
use std::thread;
use tempdir::TempDir;

use std::path::Path;

use basic::Model;

const IMAGE_BOUNDS: u32 = 256;

const SETTINGS: Settings = Settings {
    width: None,
    height: None,
    quality: 70,
    fast: true,
    once: false,
};

fn main() {
    let app = get_app();
    let matches = app.get_matches();

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
        let temp_dir = TempDir::new("basic").expect("Could not create temp dir");
        let (mut collector, writer) = gifski::new(SETTINGS).unwrap();
        let (sender, reciever) = mpsc::channel();
        let _ = thread::spawn(move || {
            let mut count = 0;
            while let Ok(path) = reciever.recv() {
                collector.add_frame_png_file(count, path, 2).unwrap();
                count += 1;
            }
        });

        for i in 0..=iterations {
            let image = model.get_curr_image(width, height, pad).unwrap();
            let temp_image_path = temp_dir.path().join(i.to_string()).with_extension("png");
            image
                .save(&temp_image_path)
                .expect("Could not save gif frame");
            sender.send(temp_image_path).unwrap();

            model.split();
        }

        drop(sender);
        let mut progess = NoProgress {};
        let gif_file =
            File::create(output_name.with_extension("gif")).expect("Could not create gif");
        writer
            .write(gif_file, &mut progess)
            .expect("Could not encode gif");
    } else {
        for _ in 0..iterations {
            model.split();
        }
    }
    model.render(output_name, width, height, pad);
}

fn get_app<'a, 'b>() -> App<'a, 'b> {
    App::new("basic")
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
                .default_value("1024"),
        )
        .arg(
            Arg::with_name("padding")
                .short("p")
                .long("padding")
                .help("Add padding to quadrants"),
        )
}
