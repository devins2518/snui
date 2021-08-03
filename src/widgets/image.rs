use crate::*;
use crate::widgets::render;
use image::imageops::{self, FilterType};
use image::io::Reader as ImageReader;
use image::{Bgra, ImageBuffer};
use std::path::Path;
use std::rc::Rc;

#[derive(Clone)]
pub struct Image {
    image: Rc<ImageBuffer<Bgra<u8>, Vec<u8>>>,
}

impl Image {
    pub fn new(image: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let dyn_image = ImageReader::open(image)?.decode()?;

        let image = dyn_image.to_bgra8();

        Ok(Self { image: Rc::new(image) })
    }

    pub fn new_with_size(image: &Path, width: u32, height: u32) -> Result<Self, Box<dyn std::error::Error>> {
        let dyn_image = ImageReader::open(image)?.decode()?;
        let scaled_image = dyn_image.resize_to_fill(width, height, FilterType::Triangle);

        let image = scaled_image.to_bgra8();

        Ok(Self { image: Rc::new(image) })
    }

    pub fn thumbnail(&self, width: u32, height: u32) -> Image {
        Image {
            image: Rc::new(imageops::thumbnail(self.image.as_ref(), width, height)),
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
        self.image = Rc::new(imageops::resize(self.image.as_ref(), width, height, FilterType::Triangle));
        Ok(())
    }
    // TODO
    fn contains(
        &mut self,
        _widget_x: u32,
        _widget_y: u32,
        _x: u32,
        _y: u32,
        _event: Input,
    ) -> Damage {
        Damage::None
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
        Rc::get_mut(&mut self.image).unwrap().iter_mut().into_slice()
    }
    fn composite(&mut self, surface: &(impl Canvas + Geometry), x: u32, y: u32) {
        let width = self.get_width();
        render(self.get_mut_buf(), surface, width as usize, x, y);
    }
}

impl Widget for Image {
    fn send_action<'s>(&'s mut self, _action: Action, _event_loop: &mut Vec<Damage>, _widget_x: u32, _widget_y: u32) {}
}
