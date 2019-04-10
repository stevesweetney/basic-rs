use image::{self, DynamicImage, GenericImageView, Pixel};
use std::cell::RefCell;
use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use std::collections::BinaryHeap;
use std::env;
use std::rc::Rc;

const iterations: u32 = 2048;

const DEFAULT_ITERATIONS: u32 = 1024;
type RcQuad = Rc<RefCell<Quad>>;
type RcImage = Rc<RefCell<DynamicImage>>;

struct Quad {
    left: u32,
    top: u32,
    right: u32,
    bottom: u32,
    color: Color,
    error: f32,
    children: Vec<RcQuad>,
    image: RcImage,
}

impl Quad {
    fn new(left: u32, top: u32, right: u32, bottom: u32, image: RcImage) -> Self {
        let cropped_image = image
            .borrow_mut()
            .crop(left, top, right - left, bottom - top);
        let (color, error) = averge_color_from_image(&cropped_image);

        Self {
            left,
            top,
            right,
            bottom,
            color,
            error,
            children: Vec::new(),
            image,
        }
    }

    fn area(&self) -> u32 {
        (self.right - self.left) * (self.bottom - self.top)
    }

    fn split(&mut self) {
        let mid_x = self.left + (self.right - self.left) / 2;
        let mid_y = self.top + (self.bottom - self.top) / 2;

        let tl = Quad::new(self.left, self.top, mid_x, mid_y, self.image.clone());
        let tr = Quad::new(mid_x, self.top, self.right, mid_y, self.image.clone());
        let bl = Quad::new(self.left, mid_y, mid_x, self.bottom, self.image.clone());
        let br = Quad::new(mid_x, mid_y, self.right, self.bottom, self.image.clone());

        self.children.clear();
        self.children.push(Rc::new(RefCell::new(tl)));
        self.children.push(Rc::new(RefCell::new(tr)));
        self.children.push(Rc::new(RefCell::new(bl)));
        self.children.push(Rc::new(RefCell::new(br)));
    }

    fn get_leaf_nodes(self) -> Vec<RcQuad> {
        let mut leaves = Vec::new();
        Self::leaves(Rc::new(RefCell::new(self)), &mut leaves);

        leaves
    }

    fn leaves(quad: RcQuad, leaves: &mut Vec<RcQuad>) {
        if quad.borrow().children.len() == 0 {
            leaves.push(quad);
            return;
        }

        for child in quad.borrow().children.iter().cloned() {
            Self::leaves(child, leaves)
        }
    }

    fn score(&self) -> f64 {
        (self.error as f64) * (self.area() as f64).powf(0.25)
    }
}

impl Ord for Quad {
    fn cmp(&self, other: &Quad) -> Ordering {
        let score = self.score();
        let other_score = other.score();

        match score.partial_cmp(&other_score) {
            Some(ordering) => ordering,
            None => Ordering::Greater,
        }
    }
}

impl PartialOrd for Quad {
    fn partial_cmp(&self, other: &Quad) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Quad {
    fn eq(&self, other: &Quad) -> bool {
        self.top == other.top
            && self.left == other.left
            && self.right == other.right
            && self.bottom == other.bottom
    }
}

impl Eq for Quad {}

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
    let image_path = match args.get(1) {
        Some(path) => path,
        None => panic!("An image path must be specified"),
    };

    let iterations = match args.get(2) {
        Some(iters) => iters.parse().unwrap(),
        None => DEFAULT_ITERATIONS,
    };

    let image =
        image::open(image_path).expect(&format!("Error opening target image {}\n", image_path));

    image.save("./output.png");
}
