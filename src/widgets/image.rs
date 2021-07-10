use crate::snui::*;
use image::io::Reader as ImageReader;
use image::{ImageBuffer, Rgba};
use std::error::Error;
use std::path::Path;

pub struct Image {
    image: ImageBuffer<Rgba<u8>, Vec<u8>>,
    x: u32,
    y: u32,
}

impl Image {
    pub fn new(image: &Path) -> Result<Self, Box<dyn Error>> {
        let dyn_image = ImageReader::open(image)?.decode()?;

        let image = dyn_image.to_rgba8();

        Ok(Self { image, x: 0, y: 0 })
    }
}

impl Geometry for Image {
    fn get_width(&self) -> u32 {
        self.image.width()
    }
    fn get_height(&self) -> u32 {
        self.image.height()
    }
    fn get_location(&self) -> (u32, u32) {
        (self.x, self.y)
    }
    fn set_location(&mut self, x: u32, y: u32) {
        self.x = x;
        self.y = y;
    }
    // TODO
    fn contains(&mut self, _x: u32, _y: u32, _event: Input) -> Damage {
        Damage::None
    }
}

impl Drawable for Image {
    fn set_content(&mut self, content: Content) {
        match content {
            Content::Empty | Content::Transparent => {
                self.image =
                    ImageBuffer::from_pixel(self.get_width(), self.get_height(), Rgba([0, 0, 0, 0]))
            }
            Content::Pixel(pixel) => {
                let arr = pixel.to_ne_bytes();
                self.image = ImageBuffer::from_pixel(self.get_width(), self.get_height(), Rgba(arr))
            }
            _ => eprintln!("Attempted to perform illegal operation on image!"),
        }
    }

    fn draw(&self, canvas: &mut super::Surface, x: u32, y: u32) {
        for (dx, dy, pixel) in self.image.enumerate_pixels() {
            let pixel = u32::from_ne_bytes(pixel.0);
            canvas.set(x + dx, y + dy, Content::Pixel(pixel));
        }
    }
}

impl Widget for Image {}
