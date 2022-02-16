use crate::*;
use image::io::Reader as ImageReader;

use crate::widgets::shapes::Rectangle;
use crate::widgets::shapes::Style;
use std::ops::Deref;
use std::path::{Path, PathBuf};

use crate::cache::RawImage;

#[derive(Clone, PartialEq, Debug, Hash, Eq)]
pub enum Scale {
    Fill,
    Fit,
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
    pub fn new_with_size<P>(path: P, width: u32, height: u32) -> Self
    where
        P: AsRef<Path>,
    {
        Self {
            path: path.as_ref().to_path_buf(),
            size: Some((width, height)),
            inner: None,
        }
    }
}

impl Geometry for Image {
    fn width(&self) -> f32 {
        if let Some(image) = self.inner.as_ref() {
            image.width()
        } else {
            0.
        }
    }
    fn height(&self) -> f32 {
        if let Some(image) = self.inner.as_ref() {
            image.height()
        } else {
            0.
        }
    }
}

impl GeometryExt for Image {
    fn set_width(&mut self, width: f32) {
        match self.size.as_mut() {
            Some(size) => size.0 = width as u32,
            None => self.size = Some((width as u32, width as u32)),
        }
    }
    fn set_height(&mut self, height: f32) {
        match self.size.as_mut() {
            Some(size) => size.1 = height as u32,
            None => self.size = Some((height as u32, height as u32)),
        }
    }
}

impl<D> Widget<D> for Image {
    fn draw_scene(&mut self, scene: Scene) {
        todo!()
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<D>, _: Event<'d>) -> Damage {
        if self.inner.is_none() {
            if let Ok(raw) = ctx.cache.image_cache.get(self.path.as_path()) {
                let mut inner = InnerImage::from(raw);
                if let Some((width, height)) = self.size.take() {
                    inner.width = width as f32;
                    inner.height = height as f32;
                }
                self.inner = Some(inner);
            } else {
                // Creates an empty InnerImage so we don't request an image again.
                self.inner = Some(InnerImage::from_raw(Vec::with_capacity(0), 0, 0));
            }
        }
        Damage::None
    }
    fn layout(&mut self, ctx: &mut LayoutCtx, constraints: &BoxConstraints) -> Size {
        self.inner
            .as_mut()
            .map(|inner| Widget::<()>::layout(inner, ctx, constraints))
            .unwrap_or_default()
    }
}

#[derive(Clone, PartialEq)]
pub struct InnerImage {
    raw: RawImage,
    scale: Scale,
    pub(crate) width: f32,
    pub(crate) height: f32,
}

impl From<RawImage> for InnerImage {
    fn from(raw: RawImage) -> Self {
        Self {
            scale: Scale::Fit,
            width: raw.width(),
            height: raw.height(),
            raw,
        }
    }
}

impl From<InnerImage> for RawImage {
    fn from(image: InnerImage) -> Self {
        image.raw.clone()
    }
}

impl InnerImage {
    pub fn from_raw(buf: Vec<u8>, width: u32, height: u32) -> Self {
        Self {
            raw: RawImage::from_raw(buf, width, height).unwrap(),
            scale: Scale::Fit,
            width: width as f32,
            height: height as f32,
        }
    }
    pub fn new(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let dyn_image = ImageReader::open(path)?.decode()?.to_bgra8();

        let (width, height) = dyn_image.dimensions();
        let image = dyn_image.into_raw();

        Ok(Self {
            raw: RawImage::from_raw(image, width, height).unwrap(),
            scale: Scale::Fit,
            width: width as f32,
            height: height as f32,
        })
    }
    pub fn set_scale(&mut self, scale: Scale) {
        self.scale = scale;
    }
    pub fn scale(&self) -> (f32, f32) {
        match &self.scale {
            Scale::Fit => (
                self.width as f32 / self.raw.width(),
                self.height as f32 / self.raw.height(),
            ),
            Scale::Fill => {
                let ratio = (self.width as f32 / self.raw.width())
                    .max(self.height as f32 / self.raw.height());
                (ratio, ratio)
            }
        }
    }
}

impl Geometry for InnerImage {
    fn width(&self) -> f32 {
        self.width as f32
    }
    fn height(&self) -> f32 {
        self.height as f32
    }
}

impl<D> Widget<D> for InnerImage {
    fn draw_scene(&mut self, scene: Scene) {
        todo!()
    }
    fn sync<'d>(&'d mut self, _: &mut SyncContext<D>, _event: Event) -> Damage {
        Damage::None
    }
    fn layout(&mut self, _ctx: &mut LayoutCtx, constraints: &BoxConstraints) -> Size {
        self.width = self
            .width
            .clamp(constraints.minimum_width(), constraints.maximum_width());
        self.height = self
            .width
            .clamp(constraints.minimum_height(), constraints.maximum_height());
        (self.width, self.height).into()
    }
}

impl Deref for InnerImage {
    type Target = RawImage;
    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}

impl std::fmt::Debug for InnerImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Image")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("scale", &self.scale)
            .finish()
    }
}
