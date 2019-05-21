use basic::Model;
use criterion::{criterion_group, criterion_main, Criterion};
use image;

const DEFAULT_ITERATIONS: u32 = 1024;

fn basic_use() {
    let image_path = "./input.jpg";
    let iterations = DEFAULT_ITERATIONS;

    let image = image::open(image_path)
        .unwrap_or_else(|_| panic!("Error opening target image {}\n", image_path));

    let mut model = Model::new(image, false);

    for _ in 0..iterations {
        model.split();
    }

    model.render("output.png");
}

fn benchmark(c: &mut Criterion) {
    c.bench_function("basic use - input.jpg", |b| b.iter(|| basic_use()));
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
