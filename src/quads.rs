use super::{average_color_from_image, Color, RcImage};
use std::cell::{Ref, RefCell, RefMut};
use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use std::rc::Rc;

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct RcQuad(Rc<RefCell<Quad>>);

impl RcQuad {
    pub fn new(q: Quad) -> RcQuad {
        RcQuad(Rc::new(RefCell::new(q)))
    }

    pub fn borrow(&self) -> Ref<Quad> {
        self.0.borrow()
    }

    pub fn borrow_mut(&mut self) -> RefMut<Quad> {
        self.0.borrow_mut()
    }

    pub fn to_inner(&self) -> Quad {
        (*self.0).clone().into_inner()
    }
}

#[derive(Clone)]
pub struct Quad {
    pub left: u32,
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
    pub color: Color,
    pub error: f32,
    pub children: Vec<RcQuad>,
    pub image: RcImage,
}

impl Quad {
    pub fn new(left: u32, top: u32, right: u32, bottom: u32, image: RcImage) -> Self {
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

    pub fn area(&self) -> u32 {
        self.width() * self.height()
    }

    pub fn height(&self) -> u32 {
        self.bottom - self.top
    }

    pub fn width(&self) -> u32 {
        self.right - self.left
    }

    pub fn split(&mut self) {
        let mid_x = self.left + self.width() / 2;
        let mid_y = self.top + self.height() / 2;

        let tl = Quad::new(self.left, self.top, mid_x, mid_y, self.image.clone());
        let tr = Quad::new(mid_x, self.top, self.right, mid_y, self.image.clone());
        let bl = Quad::new(self.left, mid_y, mid_x, self.bottom, self.image.clone());
        let br = Quad::new(mid_x, mid_y, self.right, self.bottom, self.image.clone());

        self.children.clear();
        self.children.push(RcQuad::new(tl));
        self.children.push(RcQuad::new(tr));
        self.children.push(RcQuad::new(bl));
        self.children.push(RcQuad::new(br));
    }

    pub fn get_leaf_nodes(self) -> Vec<RcQuad> {
        let mut leaves = Vec::new();
        Self::leaves(RcQuad::new(self), &mut leaves);

        leaves
    }

    pub fn leaves(quad: RcQuad, leaves: &mut Vec<RcQuad>) {
        if quad.borrow().children.is_empty() {
            leaves.push(quad);
            return;
        }

        for child in quad.borrow().children.iter().cloned() {
            Self::leaves(child, leaves)
        }
    }

    pub fn score(&self) -> f64 {
        f64::from(self.error) * f64::from(self.area()).powf(0.25)
    }
}

impl Ord for Quad {
    fn cmp(&self, other: &Quad) -> Ordering {
        match (is_small(self), is_small(other)) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            _ => {
                let score = self.score();
                let other_score = other.score();

                match score.partial_cmp(&other_score) {
                    Some(ordering) => ordering,
                    None => Ordering::Greater,
                }
            }
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

fn is_small(q: &Quad) -> bool {
    const SMALL_SIZE: u32 = 4;
    (q.width() < SMALL_SIZE) || (q.height() < SMALL_SIZE)
}
