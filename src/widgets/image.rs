use crate::*;
use image::io::Reader as ImageReader;

use scene::Instruction;
use std::path::Path;
use std::sync::Arc;

#[derive(Clone, PartialEq)]
pub struct Image {
    image: Arc<[u8]>,
    width: u32,
    height: u32,
    size: (u32, u32),
}

impl Image {
    pub fn new(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let dyn_image = ImageReader::open(path)?.decode()?.to_bgra8();

        let (width, height) = dyn_image.dimensions();
        let image: Arc<[u8]> = dyn_image.into_raw().into();

        Ok(Image {
            image,
            width,
            height,
            size: (width, height),
        })
    }
    pub fn new_with_size(
        path: &Path,
        width: u32,
        height: u32,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let dyn_image = ImageReader::open(path)?.decode()?.to_bgra8();

        let size = dyn_image.dimensions();
        let image: Arc<[u8]> = dyn_image.into_raw().into();

        Ok(Image {
            image,
            width,
            height,
            size,
        })
    }
    pub fn scale(&self) -> (f32, f32) {
        (
            self.size.0 as f32 / self.width as f32,
            self.size.1 as f32 / self.height as f32,
        )
    }
    pub fn as_image<'i>(&'i self) -> raqote::Image<'i> {
        let p = self.image.as_ptr();
        let len = self.image.len();
        let data =
            unsafe { std::slice::from_raw_parts(p as *mut u32, len / std::mem::size_of::<u32>()) };
        raqote::Image {
            width: self.size.0 as i32,
            height: self.size.1 as i32,
            data,
        }
    }
}

impl Geometry for Image {
    fn width(&self) -> f32 {
        self.width as f32
    }
    fn height(&self) -> f32 {
        self.height as f32
    }
    fn set_width(&mut self, width: f32) -> Result<(), f32> {
        if width.is_sign_positive() {
            self.width = width as u32;
            return Ok(());
        }
        Err(self.width as f32)
    }
    fn set_height(&mut self, height: f32) -> Result<(), f32> {
        if height.is_sign_positive() {
            self.height = height as u32;
            return Ok(());
        }
        Err(self.height as f32)
    }
}

impl Primitive for Image {
    fn draw(&self, x: f32, y: f32, ctx: &mut DrawContext) {
        let p = self.image.as_ptr();
        let len = self.image.len();
        let data =
            unsafe { std::slice::from_raw_parts(p as *mut u32, len / std::mem::size_of::<u32>()) };
        let image = raqote::Image {
            width: self.size.0 as i32,
            height: self.size.1 as i32,
            data,
        };
        ctx.draw_image_with_size(x, y, image, self.width as f32, self.height as f32);
    }
}

impl Widget for Image {
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        RenderNode::Instruction(Instruction::new(x, y, self.clone()))
    }
    fn sync<'d>(&'d mut self, _ctx: &mut SyncContext, _event: Event) {}
}

impl std::fmt::Debug for Image {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Image")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("size", &self.image.len())
            .finish()
    }
}
