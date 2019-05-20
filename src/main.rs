use basic::Model;
use clap::{App, Arg};
use gifski::{self, progress::ProgressBar, Settings};
use image::{self, Pixel};
use imgref::Img;
use rgb::{RGBA, RGBA8};
use std::{fs::File, io::Stdout, path::Path, sync::mpsc, thread};

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

    let mut model = Model::new(image);
    println!("Simplifying image...");
    if matches.is_present("gif") {
        let (mut collector, writer) = gifski::new(SETTINGS).unwrap();
        let (sender, reciever) = mpsc::channel();
        let _ = thread::spawn(move || {
            let mut count = 0;
            while let Ok(img_vec) = reciever.recv() {
                collector.add_frame_rgba(count, img_vec, 2).unwrap();
                count += 1;
            }
        });

        for _ in 0..=iterations {
            let image = model.get_curr_image(pad).unwrap();
            let pixels: Vec<RGBA8> = image
                .pixels()
                .map(|pix| {
                    let ch = pix.channels();
                    RGBA::new(ch[0], ch[1], ch[2], ch[3])
                })
                .collect();
            let img_vec = Img::new(pixels, image.width() as usize, image.height() as usize);
            sender.send(img_vec).unwrap();

            model.split();
        }

        drop(sender);
        println!("Encoding gif...");
        let mut progess = ProgressBar::<Stdout>::new(iterations);
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
    model.render(output_name, pad);
}

fn get_app<'a, 'b>() -> App<'a, 'b> {
    App::new("basic")
        .version("0.2")
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
