use crate::*;
use crate::widgets::render;
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

        Ok(Self { image })
    }

    pub fn new_with_size(
        image: &Path,
        width: u32,
        height: u32,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let dyn_image = ImageReader::open(image)?.decode()?;
        let scaled_image = dyn_image.resize_to_fill(width, height, FilterType::Triangle);

        let image = scaled_image.to_bgra8();

        Ok(Self { image })
    }

    pub fn thumbnail(&self, width: u32, height: u32) -> Image {
        Image {
            image: imageops::thumbnail(&self.image, width, height),
        }
    }

    pub fn size(&self) -> usize {
        (self.image.width() * self.image.height() * 4) as usize
    }
}

impl Geometry for Image {
    fn get_width(&self) -> u32 {
        self.image.width()
    }
    fn get_height(&self) -> u32 {
        self.image.height()
    }
    fn resize(&mut self, width: u32, height: u32) -> Result<(), Error> {
        if width != self.image.width() || height != self.image.height() {
            self.image = imageops::resize(&self.image, width, height, FilterType::Triangle);
        }
        Ok(())
    }
}

impl Drawable for Image {
    fn set_color(&mut self, _color: u32) {
        eprintln!("Attempted to perform illegal operation on image!");
    }

    fn draw(&self, canvas: &mut [u8], width: u32, x: u32, y: u32) {
        render(canvas, self, width as usize, x, y);
    }
}

impl Canvas for Image {
    fn size(&self) -> usize {
        let size = self.get_width() * self.get_height() * 4;
        size as usize
    }
    fn get_buf(&self) -> &[u8] {
        self.image.as_raw()
    }
    fn get_mut_buf(&mut self) -> &mut [u8] {
        self.image.iter_mut().into_slice()
    }
    fn composite(&mut self, surface: &(impl Canvas + Geometry), x: u32, y: u32) {
        let width = self.get_width();
        render(self.get_mut_buf(), surface, width as usize, x, y);
    }
}

impl Widget for Image {
    fn roundtrip<'d>(
        &'d mut self,
        _widget_x: u32,
        _widget_y: u32,
        _dispatched: Dispatch,
    ) -> Option<Damage> {
        None
    }
}
