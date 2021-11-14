use crate::*;
use raqote::*;
use scene::*;
use widgets::font::*;
use widgets::u32_to_source;
use std::ops::{Deref, DerefMut};
use lyon_geom::euclid::{point2, vec2, Angle};

const ATOP_OPTIONS: DrawOptions = DrawOptions {
    alpha: 1.,
    blend_mode: BlendMode::SrcAtop,
    antialias: AntialiasMode::Gray,
};

const DRAW_OPTIONS: DrawOptions = DrawOptions {
    blend_mode: BlendMode::SrcOver,
    alpha: 1.,
    antialias: AntialiasMode::Gray,
};

pub enum Backend {
    Raqote(DrawTarget),
    Dummy,
}

// Very WIP
// I'm considering making a bigger context from which both
// the SyncContext and DrawContext are derived.
pub struct SyncContext<'c> {
    pub font_cache: &'c mut FontCache,
}

pub struct DrawContext {
    pub backend: Backend,
    pub font_cache: FontCache,
    pending_damage: Vec<Region>,
}

impl DrawContext {
    pub fn new(backend: Backend) -> Self {
        Self {
            backend,
            pending_damage: Vec::new(),
            font_cache: FontCache::new(),
        }
    }
    pub fn damage_region(&mut self, bg: &Background, region: &Region) {
        match bg {
            Background::Color(source) => match &mut self.backend {
                Backend::Raqote(dt) => dt.fill_rect(
                    region.x,
                    region.y,
                    region.width,
                    region.height,
                    &Source::Solid(*source),
                    &ATOP_OPTIONS,
                ),
                _ => {}
            },
            Background::Transparent => match &mut self.backend {
                Backend::Raqote(dt) => dt.fill_rect(
                    region.x,
                    region.y,
                    region.width,
                    region.height,
                    &Source::Solid(u32_to_source(0)),
                    &DrawOptions {
                        alpha: 1.,
                        antialias: AntialiasMode::Gray,
                        blend_mode: BlendMode::SrcIn,
                    },
                ),
                _ => {}
            },
            // To-do
            _ => unreachable!()
        }
        self.pending_damage.push(*region);
    }
    pub fn clear(&mut self) {
        match &mut self.backend {
            Backend::Raqote(dt) => {
                dt.clear(u32_to_source(0));
            }
            _ => {}
        }
        self.flush();
    }
    pub fn is_damaged(&self) -> bool {
        !self.pending_damage.is_empty()
    }
    pub fn flush(&mut self) {
        self.pending_damage.clear();
        self.font_cache.layouts.clear();
    }
    pub fn report_damage(&self) -> &[Region] {
        &self.pending_damage
    }
    pub fn draw_image(&mut self, x: f32, y: f32, image: Image) {
        match &mut self.backend {
            Backend::Raqote(dt) => dt.draw_image_at(x, y, &image, &DRAW_OPTIONS),
            _ => {}
        }
    }
    pub fn draw_image_with_size(&mut self, x: f32, y: f32, image: Image, width: f32, height: f32) {
        match &mut self.backend {
            Backend::Raqote(dt) => {
                dt.draw_image_with_size_at(width, height, x, y, &image, &DRAW_OPTIONS)
            }
            _ => {}
        }
    }
    pub fn sync(&mut self) -> SyncContext<'_> {
        SyncContext {
            font_cache: &mut self.font_cache
        }
    }
}

impl Geometry for DrawContext {
    fn set_size(&mut self, width: f32, height: f32) -> Result<(), (f32, f32)> {
        match &mut self.backend {
            Backend::Raqote(dt) => {
                *dt = DrawTarget::new(width as i32, height as i32);
            }
            _ => {}
        }
        Ok(())
    }
    fn width(&self) -> f32 {
        match &self.backend {
            Backend::Raqote(dt) => dt.width() as f32,
            _ => 0.,
        }
    }
    fn height(&self) -> f32 {
        match &self.backend {
            Backend::Raqote(dt) => dt.height() as f32,
            _ => 0.,
        }
    }
}

impl Deref for DrawContext {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        match &self.backend {
            Backend::Raqote(dt) => dt.get_data_u8(),
            _ => panic!("Dummy backend cannot return a slice"),
        }
    }
}

impl DerefMut for DrawContext {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match &mut self.backend {
            Backend::Raqote(dt) => dt.get_data_u8_mut(),
            _ => panic!("Dummy backend cannot return a slice"),
        }
    }
}
