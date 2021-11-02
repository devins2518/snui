use crate::*;
use raqote::*;
use std::cmp::Ordering;
use crate::widgets::primitives::{Style, Shape};

#[derive(Copy, Clone, Debug)]
pub enum Background {
    // Image,
    Text,
    Transparent,
    Color(SolidSource),
}

#[derive(Clone, Debug)]
pub struct Scene {
    last: usize,
    shape: Shape,
    region: Region,
    subscenes: Vec<Scene>,
    background: Background,
}

impl Scene {
    fn new(region: Region, background: Background) -> Scene {
        Scene {
            last: 0,
            shape: Shape::Rectangle,
            subscenes: Vec::new(),
            background,
            region,
        }
    }
    fn insert(&mut self, x: f32, y: f32, width: f32, height: f32, background: Background) {
        let region = Region::new(x, y, width, height);
        if self.subscenes.is_empty() {
            self.subscenes.push(Scene::new(region, background));
        } else if self.subscenes[self.last].region.eq(&region) {
            self.subscenes[self.last].insert(x, y, width, height, background);
        } else {
            if let Ok(index) = self.subscenes.binary_search_by(|scene| {
                scene.region.cmp(&region)
            }) {
                self.last = index;
                self.subscenes[index].insert(x, y, width, height, background);
            } else {
                self.subscenes.push(Scene::new(region, background));
            }
        }
    }
    fn get_background(&mut self, x: f32, y: f32, width: f32, height: f32) -> Background {
        let region = Region::new(x, y, width, height);
        if self.subscenes.is_empty() {
            self.background
        } else if self.subscenes[self.last].region.eq(&region) {
            self.subscenes[self.last].background
        } else {
            if let Ok(index) = self.subscenes.binary_search_by(|scene| {
                scene.region.cmp(&region)
            }) {
                self.last = index;
                self.subscenes[index].get_background(x, y, width, height)
            } else {
                self.background
            }
        }
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
    fn new(x: f32, y: f32, width: f32, height: f32) -> Region {
        Region {
            x, y, width, height
        }
    }
    fn contains(&self,x: f32, y: f32) -> bool {
        self.x <= x
        && self.y <= y
        && self.x + self.width >= x
        && self.y + self.height >= y
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
    fn clamp(self, min: Self, max: Self) -> Self
    where Self: Sized, {
        if self > max {
            max
        } else if self < min {
            min
        } else {
            self
        }
    }
    fn max(self, other: Self) -> Self
    where Self: Sized, {
        if self > other {
            self
        } else {
            other
        }
    }
    fn min(self, other: Self) -> Self
    where Self: Sized, {
        if other < self {
            other
        } else {
            self
        }
    }
    fn cmp(&self, other: &Self) -> Ordering {
        if other < self {
            Ordering::Less
        } else if other > self {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}
