use crate::*;
use image::imageops::{self, FilterType};
use image::io::Reader as ImageReader;
use image::{Bgra, ImageBuffer};
use std::path::Path;

#[derive(Clone)]
pub struct Image {
    image: ImageBuffer<Bgra<u8>, Vec<u8>>,
}

impl Image {
    pub fn new(image: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let dyn_image = ImageReader::open(image)?.decode()?;

        let image = dyn_image.to_bgra8();

        Ok(Self {
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
            image,
        })
    }

    pub fn thumbnail(&self, width: u32, height: u32) -> Image {
        Image {
            image: imageops::thumbnail(&self.image, width, height),
        }
    }

    pub fn size(&self) -> usize {
        (self.image.width() * self.image.height() * 4) as usize
    }

    pub fn resize(&self, width: u32, height: u32) -> Self {
        Self {
            image: imageops::resize(&self.image, width, height, FilterType::Triangle),
        }
    }
}

impl Geometry for Image {
    fn width(&self) -> f32 {
        self.image.width() as f32
    }
    fn height(&self) -> f32 {
        self.image.height() as f32
    }
}

impl Drawable for Image {
    fn set_color(&mut self, _color: u32) {
        eprintln!("Attempted to perform illegal operation on image!");
    }

    fn draw(&self, canvas: &mut Canvas, x: f32, y: f32) {
        let buf = self.image.as_raw();
        let p = buf.as_ptr();
        let len = buf.len();
        let data= unsafe {
            std::slice::from_raw_parts(p as *mut u32, len / std::mem::size_of::<u32>())
        };
        let image = raqote::Image {
            width: self.image.width() as i32,
            height: self.image.height() as i32,
            data
        };
        canvas.draw_image(x, y, image);
    }
}

impl Widget for Image {
    fn roundtrip<'d>(&'d mut self, wx: f32, wy: f32, canvas: &mut Canvas, dispatch: &Dispatch) {
        if let Dispatch::Commit = dispatch {
        }
    }
}
