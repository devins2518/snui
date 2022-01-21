use crate::*;
use image::io::Reader as ImageReader;
use std::ops::DerefMut;
use tiny_skia::*;

use scene::Instruction;
use std::path::Path;
use std::sync::Arc;

#[derive(Clone, PartialEq, Debug)]
pub enum Scale {
    Fill,
    Fit,
}

#[derive(Clone)]
pub struct Image {
    image: Arc<[u8]>,
    width: u32,
    height: u32,
    scale: Scale,
    opacity: f32,
    size: (u32, u32),
}

impl PartialEq for Image {
    fn eq(&self, other: &Self) -> bool {
        self.image.as_ptr() == other.image.as_ptr()
            && self.scale == other.scale
            && self.width == other.width
            && self.height == other.height
            && self.opacity == other.opacity
    }
}

impl Image {
    pub fn from_raw(buf: Vec<u8>, width: u32, height: u32) -> Self {
        let image: Arc<[u8]> = buf.into();
        Image {
            image,
            width,
            height,
            opacity: 1.,
            scale: Scale::Fit,
            size: (width, height),
        }
    }
    pub fn new(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let dyn_image = ImageReader::open(path)?.decode()?.to_bgra8();

        let (width, height) = dyn_image.dimensions();
        let image: Arc<[u8]> = dyn_image.into_raw().into();

        Ok(Image {
            image,
            width,
            height,
            opacity: 1.,
            scale: Scale::Fit,
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
            opacity: 1.,
            scale: Scale::Fit,
        })
    }
    pub fn set_scale(&mut self, scale: Scale) {
        self.scale = scale;
    }
    pub fn set_opacity(&mut self, opacity: f32) {
        self.opacity = opacity
    }
    pub fn scale(&self) -> (f32, f32) {
        match &self.scale {
            Scale::Fit => (
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
    fn get_texture(&self) -> scene::Texture {
        scene::Texture::Image(scene::Coords::new(0., 0.), self.clone())
    }
    fn apply_texture(&self, texture: scene::Texture) -> scene::PrimitiveType {
        widgets::shapes::Rectangle::empty(self.width(), self.height()).apply_texture(texture)
    }
    fn primitive_type(&self) -> scene::PrimitiveType {
        self.clone().into()
    }
    fn contains(&self, region: &scene::Region) -> bool {
        scene::Region::new(0., 0., self.width(), self.height()).fit(region)
    }
    fn draw_with_transform_clip(
        &self,
        ctx: &mut DrawContext,
        transform: tiny_skia::Transform,
        clip: Option<&tiny_skia::ClipMask>,
    ) {
        if let Backend::Pixmap(dt) = ctx.deref_mut() {
            let (sx, sy) = self.scale();
            dt.draw_pixmap(
                0,
                0,
                PixmapRef::from_bytes(self.image.as_ref(), self.size.0, self.size.1).unwrap(),
                &PixmapPaint {
                    blend_mode: BlendMode::SourceOver,
                    opacity: self.opacity,
                    quality: FilterQuality::Bilinear
                },
                transform.pre_scale(sx, sy),
                clip,
            );
        }
    }
}

impl<M> Widget<M> for Image {
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        RenderNode::Instruction(Instruction::new(x, y, self.clone()))
    }
    fn sync<'d>(&'d mut self, _ctx: &mut SyncContext<M>, _event: Event<M>) -> Damage {
        Damage::None
    }
}

impl AsRef<[u8]> for Image {
    fn as_ref(&self) -> &[u8] {
        self.image.as_ref()
    }
}

impl std::fmt::Debug for Image {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Image")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("scale", &self.scale)
            .field("size", &self.image.len())
            .finish()
    }
}
