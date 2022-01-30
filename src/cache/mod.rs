pub mod font;
pub mod image;

pub use self::image::*;
pub use font::*;

pub struct Cache {
    pub(crate) font_cache: FontCache,
    pub(crate) image_cache: ImageCache,
}

impl AsRef<ImageCache> for Cache {
    fn as_ref(&self) -> &ImageCache {
        &self.image_cache
    }
}

impl AsMut<FontCache> for Cache {
    fn as_mut(&mut self) -> &mut FontCache {
        &mut self.font_cache
    }
}

impl AsMut<ImageCache> for Cache {
    fn as_mut(&mut self) -> &mut ImageCache {
        &mut self.image_cache
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self {
            font_cache: FontCache::new(),
            image_cache: ImageCache::default(),
        }
    }
}

pub fn get_image<C, P>(cache: &mut C, path: P) -> Option<RawImage>
where
    C: AsMut<Cache>,
    P: AsRef<std::path::Path>,
{
    cache.as_mut().image_cache.get(path).ok()
}
