use image::{self, imageops, DynamicImage, GenericImage, GenericImageView, Pixel, RgbaImage};
use std::{cell::RefCell, collections::BinaryHeap, path::Path, rc::Rc};

mod quads;
use quads::{Quad, RcQuad};

pub type Color = (u8, u8, u8);
pub type RcImage = Rc<RefCell<DynamicImage>>;

pub struct Model {
    padding: bool,
    width: u32,
    height: u32,
    quads: BinaryHeap<RcQuad>,
}

impl Model {
    pub fn new(target: DynamicImage, padding: bool) -> Self {
        let (height, width) = (target.height(), target.width());
        let target = Rc::new(RefCell::new(target));

        let q = Quad::new(0, 0, width, height, target.clone());
        let root = RcQuad::new(q);

        let mut quads = BinaryHeap::new();
        quads.push(root.clone());
        Self {
            padding,
            height,
            width,
            quads,
        }
    }

    pub fn split(&mut self) {
        if let Some(mut quad) = self.quads.pop() {
            let mut quad = quad.borrow_mut();
            quad.split();

            for child in &quad.children {
                self.quads.push(child.clone())
            }
        }
    }

    pub fn get_curr_image(&self) -> Option<RgbaImage> {
        let padding = if self.padding { 1 } else { 0 };
        let mut result = RgbaImage::new(self.width + padding, self.height + padding);

        let mut coords = Vec::new();
        for quad in &self.quads {
            let quad = quad.borrow();
            let mut cropped = imageops::crop(
                &mut result,
                quad.left + padding,
                quad.top + padding,
                quad.width() - padding,
                quad.height() - padding,
            );

            coords.extend(cropped.pixels().map(|(x, y, _)| (x, y)));
            for (x, y) in &coords {
                let p = cropped.get_pixel_mut(*x, *y);
                let ch = p.channels_mut();
                ch[0] = quad.color.0;
                ch[1] = quad.color.1;
                ch[2] = quad.color.2;
                ch[3] = 255;
            }

            coords.clear();
        }
        Some(result)
    }

    pub fn render<P: AsRef<Path>>(&self, result_name: P) {
        match self.get_curr_image() {
            Some(image) => image.save(&result_name).expect("Error saving image"),
            _ => panic!("Could not render image"),
        };
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn create_model() {
        let image_path = "./input.jpg";

        let image = image::open(image_path)
            .unwrap_or_else(|_| panic!("Error opening target image {}\n", image_path));

        let (width, height) = image.dimensions();

        let model = Model::new(image, false);

        assert_eq!(model.width, width);
        assert_eq!(model.height, height);
        assert_eq!(model.quads.len(), 1);
    }
    #[test]
    fn split() {
        let image_path = "./input.jpg";
        let image = image::open(image_path)
            .unwrap_or_else(|_| panic!("Error opening target image {}\n", image_path));

        let mut model = Model::new(image, false);
        model.split();

        assert_eq!(model.quads.len(), 4);
    }
}
