use crate::*;
use image::io::Reader as ImageReader;

use std::ops::Deref;
use std::path::{Path, PathBuf};

use crate::cache::RawImage;

use super::shapes::{Rectangle, Style};

#[derive(Clone, PartialEq, Debug, Hash, Eq)]
pub enum Scale {
    Fill,
    KeepAspect,
}

pub struct Image {
    path: PathBuf,
    size: Option<(u32, u32)>,
    inner: Option<InnerImage>,
}

impl Image {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            size: None,
            inner: None,
        }
    }
}

impl Geometry for Image {
    fn width(&self) -> f32 {
        self.inner
            .as_ref()
            .map(|inner| inner.width())
            .unwrap_or_default()
    }
    fn height(&self) -> f32 {
        self.inner
            .as_ref()
            .map(|inner| inner.height())
            .unwrap_or_default()
    }
}

impl<T> Widget<T> for Image {
    fn draw_scene(&mut self, scene: Scene) {
        if let Some(image) = self.inner.as_mut() {
            Widget::<()>::draw_scene(image, scene)
        }
    }
    fn event<'s>(&'s mut self, ctx: &mut SyncContext<T>, event: Event<'s>) -> Damage {
        Damage::None
    }
    fn update<'s>(&'s mut self, ctx: &mut SyncContext<T>) -> Damage {
        Damage::None
    }
    fn layout(&mut self, ctx: &mut LayoutCtx, constraints: &BoxConstraints) -> Size {
        if self.inner.is_none() {
            if let Ok(raw) = ctx.cache.image_cache.get(self.path.as_path()) {
                let mut inner = InnerImage::from(raw);
                if let Some((width, height)) = self.size.take() {
                    inner.image.set_width(width as f32);
                    inner.image.set_height(height as f32);
                }
                self.inner = Some(inner);
            } else {
                // Creates an empty InnerImage so we don't request an image again.
                self.inner = Some(InnerImage::from_raw(Vec::with_capacity(0), 0, 0));
            }
        }
        self.inner
            .as_mut()
            .map(|inner| Widget::<()>::layout(inner, ctx, constraints))
            .unwrap_or_default()
    }
}

#[derive(Clone, PartialEq)]
pub struct InnerImage {
    scale: Scale,
    image: Rectangle,
}

impl From<RawImage> for InnerImage {
    fn from(raw: RawImage) -> Self {
        Self {
            scale: Scale::KeepAspect,
            image: Rectangle::new(raw.width(), raw.height()).texture(raw),
        }
    }
}

impl From<InnerImage> for RawImage {
    fn from(image: InnerImage) -> Self {
        match image.image.texture {
            scene::Texture::Image(raw) => raw,
            _ => unreachable!(),
        }
    }
}

impl InnerImage {
    pub fn from_raw(buf: Vec<u8>, width: u32, height: u32) -> Self {
        RawImage::from_raw(buf, width, height).unwrap().into()
    }
    pub fn new(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let dyn_image = ImageReader::open(path)?.decode()?.to_bgra8();

        let (width, height) = dyn_image.dimensions();
        let image = dyn_image.into_raw();

        Ok(RawImage::from_raw(image, width, height).unwrap().into())
    }
    pub fn set_scale(&mut self, scale: Scale) {
        self.scale = scale;
    }
}

impl Geometry for InnerImage {
    fn width(&self) -> f32 {
        self.image.width()
    }
    fn height(&self) -> f32 {
        self.image.height()
    }
}

impl<T> Widget<T> for InnerImage {
    fn draw_scene(&mut self, scene: Scene) {
        Widget::<()>::draw_scene(&mut self.image, scene)
    }
    fn event<'s>(&'s mut self, ctx: &mut SyncContext<T>, event: Event<'s>) -> Damage {
        Damage::None
    }
    fn update<'s>(&'s mut self, ctx: &mut SyncContext<T>) -> Damage {
        Damage::None
    }
    fn layout(&mut self, ctx: &mut LayoutCtx, constraints: &BoxConstraints) -> Size {
        match self.scale {
            Scale::KeepAspect => {
                let raw = match self.image.texture {
                    scene::Texture::Image(ref raw) => raw,
                    _ => unreachable!(),
                };
                let ratio = raw.height() / raw.width();
                let constraints = constraints.with_max(
                    constraints.maximum_width(),
                    constraints.maximum_width() * ratio,
                );
                Widget::<()>::layout(&mut self.image, ctx, &constraints)
            }
            Scale::Fill => Widget::<()>::layout(&mut self.image, ctx, constraints),
        }
    }
}

impl Deref for InnerImage {
    type Target = RawImage;
    fn deref(&self) -> &Self::Target {
        match self.image.texture {
            scene::Texture::Image(ref raw) => raw,
            _ => unreachable!(),
        }
    }
}

impl std::fmt::Debug for InnerImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Image")
            .field("width", &self.width())
            .field("height", &self.height())
            .field("scale", &self.scale)
            .finish()
    }
}
