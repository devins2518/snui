use raqote::*;
use crate::widgets::blend;
use std::cmp::Ordering;

#[derive(Copy, Clone, Debug)]
pub enum Background {
    Transparent,
    Color(SolidSource),
}

impl Background {
    pub fn merge(&mut self, other: Self) {
        if let Background::Color(bsource) = other {
            match self {
                Background::Color(asource) => {
                    let source = blend(&asource.to_u32().to_be_bytes(), &bsource.to_u32().to_be_bytes(), 1.);
                    *asource = SolidSource {
                        a: source[0],
                        r: source[1],
                        g: source[2],
                        b: source[3],
                    };
                }
                Background::Transparent => *self = other,
            }
        }
    }
}

pub struct Scene {
    pub region: Region,
    pub background: Background,
}

impl Scene {
    pub fn default() -> Scene {
        Scene {
            background: Background::Transparent,
            region: Region::new(0., 0., 1., 1.),
        }
    }
    pub fn new(background: Background, region: Region) -> Self {
        Self { background, region }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Region {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Region {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Region {
        Region {
            x, y, width, height
        }
    }
    pub fn merge(&mut self, other: &Self) {
        if self.contains(other.x, other.y) {
            self.width = self.x.max(other.x) + self.width.max(other.width);
            self.height = self.y.max(other.y) + self.height.max(other.height);
        }
    }
    pub fn contains(&self, x: f32, y: f32) -> bool {
        x - self.x <= self.width
        && y - self.y <= self.height
    }
}

impl PartialEq for Region {
    fn eq(&self, other: &Self) -> bool {
        other.x - self.x + other.width <= self.width
        && other.y - self.y + other.height <= self.height
    }
    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl PartialOrd for Region {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.x > other.x + other.width
        || self.y > other.y + other.height  {
            Some(Ordering::Greater)
        } else if self.x + self.width < other.x
        || self.y + self.height < other.y {
            Some(Ordering::Less)
        } else {
            Some(Ordering::Equal)
        }
    }
}

impl Eq for Region {}

impl Ord for Region {
    fn cmp(&self, other: &Self) -> Ordering {
        if self < other {
            Ordering::Less
        } else if self > other {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}
