use crate::*;
use image::io::Reader as ImageReader;
use std::ops::{Deref, DerefMut};
use tiny_skia::*;

use scene::Instruction;
use std::path::Path;
use std::sync::Arc;

#[derive(Clone, PartialEq)]
enum Scale {
    Fill,
    Size,
}

#[derive(Clone, PartialEq)]
pub struct Image {
    image: Arc<[u8]>,
    width: u32,
    height: u32,
    scale: Scale,
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
            scale: Scale::Fill,
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
            scale: Scale::Fill,
        })
    }
    pub fn scale(&self) -> (f32, f32) {
        match &self.scale {
            Scale::Size => (
                self.width as f32 / self.size.0 as f32,
                self.height as f32 / self.size.1 as f32,
            ),
            Scale::Fill => {
                let ratio = (self.width as f32 / self.size.0 as f32)
                    .max(self.height as f32 / self.size.1 as f32);
                (ratio, ratio)
            }
        }
    }
    pub fn pixmap(&self) -> PixmapRef {
        PixmapRef::from_bytes(self.image.as_ref(), self.size.0, self.size.1).unwrap()
    }
    pub fn as_ref(&self) -> &[u8] {
        self.image.as_ref()
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
        if let Backend::Pixmap(dt) = ctx.deref_mut() {
            let (sw, sh) = self.scale();
            dt.draw_pixmap(
                x as i32,
                y as i32,
                PixmapRef::from_bytes(self.image.as_ref(), self.size.0, self.size.1).unwrap(),
                &crate::context::PIX_PAINT,
                Transform::from_scale(sw, sh),
                None,
            );
        }
        // ctx.draw_image_with_size(x, y, image, self.width as f32, self.height as f32);
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
