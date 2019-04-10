use image::{self, imageops, DynamicImage, GenericImage, GenericImageView, Pixel, RgbImage};
use std::cell::RefCell;
use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use std::collections::BinaryHeap;
use std::rc::Rc;

type Color = (u8, u8, u8);
type RcQuad = Rc<RefCell<Quad>>;
type RcImage = Rc<RefCell<DynamicImage>>;

pub struct Model {
    width: u32,
    height: u32,
    quads: BinaryHeap<RcQuad>,
    root: Option<RcQuad>,
}

impl Model {
    pub fn new(target: DynamicImage) -> Self {
        let (height, width) = (target.height(), target.width());
        let target = Rc::new(RefCell::new(target));

        let q = Quad::new(0, 0, width, height, target.clone());
        let root = Rc::new(RefCell::new(q));

        let mut quads = BinaryHeap::new();
        quads.push(root.clone());
        Self {
            root: Some(root),
            height,
            width,
            quads,
        }
    }

    pub fn split(&mut self) {
        if let Some(quad) = self.quads.pop() {
            let mut quad = quad.borrow_mut();
            quad.split();

            for child in &quad.children {
                self.quads.push(child.clone())
            }
        }
    }

    pub fn render(&mut self, result_width: u32, result_height: u32) {
        if let Some(root) = self.root.take() {
            let mut result = RgbImage::new(self.width, self.height);
            let root = (*root).clone().into_inner();

            for quad in root.get_leaf_nodes() {
                let quad = quad.borrow();
                let height = quad.bottom - quad.top;
                let width = quad.right - quad.left;

                let mut cropped = imageops::crop(&mut result, quad.left, quad.top, width, height);

                let coords: Vec<_> = cropped.pixels().map(|(x, y, _)| (x, y)).collect();

                for (x, y) in coords {
                    let p = cropped.get_pixel_mut(x, y);
                    let ch = p.channels_mut();
                    ch[0] = quad.color.0;
                    ch[1] = quad.color.1;
                    ch[2] = quad.color.2;
                }
            }

            let resized = imageops::resize(&result, result_width, result_height, imageops::Nearest);

            resized
                .save("./output.png")
                .expect("Error saving output.png");
        }
    }
}

#[derive(Clone)]
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
        let (color, error) = average_color_from_image(&cropped_image);

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
        if quad.borrow().children.is_empty() {
            leaves.push(quad);
            return;
        }

        for child in quad.borrow().children.iter().cloned() {
            Self::leaves(child, leaves)
        }
    }

    fn score(&self) -> f64 {
        f64::from(self.error) * f64::from(self.area()).powf(0.25)
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

fn average_color_from_image(image: &DynamicImage) -> (Color, f32) {
    let mut histogram = [0; 768];
    for (_x, _y, pix) in image.pixels() {
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

    (
        (red as u8, green as u8, blue as u8),
        re * 0.3 + ge * 0.6 + be + 0.1,
    )
}

fn weighted_average(histogram: &[u32]) -> (u32, f32) {
    let mut weighted_sum = 0;
    let mut total = 0;

    for (idx, c) in histogram.iter().enumerate() {
        weighted_sum += (idx as u32) * c;
        total += c;
    }

    if total == 0 {
        return (0, 0.0);
    }
    let value = weighted_sum / total;
    // root mean square error
    let mut error = 0;
    for (idx, c) in histogram.iter().enumerate() {
        error += u64::from(*c) * (i64::from(value) - idx as i64).pow(2) as u64;
    }

    (value, ((error / u64::from(total)) as f32).sqrt())
}
