use crate::Geometry;
use image::io::Reader as ImageReader;
use std::clone::Clone;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tiny_skia::PixmapRef;

#[derive(Clone, PartialEq, Debug)]
pub struct RawImage {
    image: Arc<[u8]>,
    width: u32,
    height: u32,
}

pub struct ImageCache {
    cache: HashMap<PathBuf, RawImage>,
}

impl Default for ImageCache {
    fn default() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }
}

use std::borrow::Borrow;

impl RawImage {
    pub fn from_raw(buf: Vec<u8>, width: u32, height: u32) -> Option<Self> {
        let image: Arc<[u8]> = buf.into();
        if width * height * 4 == image.len() as u32 {
            Some(Self {
                image,
                width,
                height,
            })
        } else {
            None
        }
    }
    pub fn pixmap(&self) -> PixmapRef {
        PixmapRef::from_bytes(self.image.as_ref(), self.width as u32, self.height as u32).unwrap()
    }
}

impl Geometry for RawImage {
    fn width(&self) -> f32 {
        self.width as f32
    }
    fn height(&self) -> f32 {
        self.height as f32
    }
}

impl AsRef<[u8]> for RawImage {
    fn as_ref(&self) -> &[u8] {
        self.image.as_ref()
    }
}

impl ImageCache {
    pub fn try_get<P>(&self, path: P) -> Option<&RawImage>
    where
        P: AsRef<Path>,
    {
        self.cache.get(path.as_ref())
    }
    /// Tries to retreive the image from the cache.
    /// If it fails it will attempt to load it from the given path.
    pub fn get<P>(&mut self, path: P) -> Result<RawImage, Box<dyn std::error::Error>>
    where
        P: AsRef<Path>,
    {
        match self.cache.get(path.as_ref()) {
            Some(image) => Ok(image.clone()),
            None => {
                let dyn_image = ImageReader::open(path.borrow())?.decode()?.to_bgra8();

                let (width, height) = dyn_image.dimensions();
                let image: Arc<[u8]> = dyn_image.into_raw().into();

                let raw = RawImage {
                    image,
                    width,
                    height,
                };

                self.cache.insert(path.as_ref().to_path_buf(), raw.clone());

                Ok(raw)
            }
        }
    }
}
