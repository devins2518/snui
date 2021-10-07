use crate::*;
use crate::widgets::render;
use image::imageops::{self, FilterType};
use image::io::Reader as ImageReader;
use image::{Bgra, ImageBuffer};
use std::path::Path;

#[derive(Clone)]
pub struct Image {
    pub damaged: bool,
    image: ImageBuffer<Bgra<u8>, Vec<u8>>,
}

impl Image {
    pub fn new(image: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let dyn_image = ImageReader::open(image)?.decode()?;

        let image = dyn_image.to_bgra8();

        Ok(Self {
            damaged: true,
            image,
        })
    }

    pub fn new_with_size(
        image: &Path,
        width: u32,
        height: u32,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let dyn_image = ImageReader::open(image)?.decode()?;
        let scaled_image = dyn_image.resize_to_fill(width, height, FilterType::Triangle);

        let image = scaled_image.to_bgra8();

        Ok(Self {
            damaged: true,
            image,
        })
    }

    pub fn thumbnail(&self, width: u32, height: u32) -> Image {
        Image {
            damaged: true,
            image: imageops::thumbnail(&self.image, width, height),
        }
    }

    pub fn size(&self) -> usize {
        (self.image.width() * self.image.height() * 4) as usize
    }

    pub fn resize(&self, width: u32, height: u32) -> Self {
        Self {
            damaged: true,
            image: imageops::resize(&self.image, width, height, FilterType::Triangle),
        }
    }
}

impl Geometry for Image {
    fn width(&self) -> u32 {
        self.image.width() as u32
    }
    fn height(&self) -> u32 {
        self.image.height() as u32
    }
}

impl Drawable for Image {
    fn set_color(&mut self, _color: u32) {
        eprintln!("Attempted to perform illegal operation on image!");
    }

    fn draw(&self, canvas: &mut Canvas, x: u32, y: u32) {
        canvas.push(x, y, self, false);
        render(canvas, self.image.as_raw(), self.image.width(), x, y);
    }
}

impl Widget for Image {
    fn damaged(&self) -> bool {
        self.damaged
    }
    fn roundtrip<'d>(
        &'d mut self,
        _widget_x: u32,
        _widget_y: u32,
        _dispatched: &Dispatch,
    ) -> Option<Damage> {
        None
    }
}
