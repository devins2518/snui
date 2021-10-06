pub mod shapes;

use raqote::*;
use shapes::*;

pub enum Style {
    Fill(SolidSource),
    Border(SolidSource, f32),
    Empty
}

pub struct BorderedRectangle {
    rect: Rectangle,
    border: Rectangle,
}
