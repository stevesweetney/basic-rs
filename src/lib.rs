use image::{self, imageops, DynamicImage, GenericImage, GenericImageView, Pixel, RgbImage};
use std::cell::RefCell;
use std::collections::{BinaryHeap, HashSet};
use std::rc::Rc;

mod quads;
use quads::{Quad, RcQuad};

pub type Color = (u8, u8, u8);
pub type RcImage = Rc<RefCell<DynamicImage>>;

pub struct Model {
    width: u32,
    height: u32,
    quads: BinaryHeap<RcQuad>,
    leaves: HashSet<RcQuad>,
}

impl Model {
    pub fn new(target: DynamicImage) -> Self {
        let (height, width) = (target.height(), target.width());
        let target = Rc::new(RefCell::new(target));

        let q = Quad::new(0, 0, width, height, target.clone());
        let root = RcQuad::new(q);

        let mut quads = BinaryHeap::new();
        let mut leaves = HashSet::new();
        quads.push(root.clone());
        leaves.insert(root.clone());
        Self {
            height,
            width,
            quads,
            leaves,
        }
    }

    pub fn split(&mut self) {
        if let Some(mut quad) = self.quads.pop() {
            self.leaves.remove(&quad);
            let mut quad = quad.borrow_mut();
            quad.split();

            for child in &quad.children {
                self.leaves.insert(child.clone());
                self.quads.push(child.clone())
            }
        }
    }

    pub fn render(&self, result_name: &str, result_width: u32, result_height: u32, pad: bool) {
        let padding = if pad { 1 } else { 0 };
        let mut result = RgbImage::new(self.width + padding, self.height + padding);

        for quad in &self.leaves {
            let quad = quad.borrow();
            let mut cropped = imageops::crop(
                &mut result,
                quad.left + padding,
                quad.top + padding,
                quad.width() - padding,
                quad.height() - padding,
            );

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

        resized.save(result_name).expect("Error saving output.png");
    }
}

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
    let (green, ge) = weighted_average(&histogram[256..=511]);
    let (blue, be) = weighted_average(&histogram[512..=767]);

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
