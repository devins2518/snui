use crate::font::FontCache;
use crate::*;
use data::*;
use raqote::*;
use scene::*;
use std::ops::{Deref, DerefMut};
use widgets::text::Label;
use widgets::u32_to_source;

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

pub enum Backend<'b> {
    Raqote(&'b mut DrawTarget),
    Dummy,
}

pub struct SyncContext<'c> {
    model: &'c mut dyn Controller,
    pub font_cache: &'c mut FontCache,
}

pub struct DrawContext<'c> {
    backend: Backend<'c>,
    font_cache: &'c mut FontCache,
    pending_damage: &'c mut Vec<Region>,
}

impl<'b> Geometry for Backend<'b> {
    fn width(&self) -> f32 {
        match self {
            Backend::Dummy => 0.,
            Backend::Raqote(dt) => dt.width() as f32,
        }
    }
    fn height(&self) -> f32 {
        match self {
            Backend::Dummy => 0.,
            Backend::Raqote(dt) => dt.height() as f32,
        }
    }
    fn set_width(&mut self, width: f32) -> Result<(), f32> {
        match self {
            Backend::Dummy => Err(0.),
            Backend::Raqote(dt) => {
                **dt = DrawTarget::new(width as i32, dt.height());
                Ok(())
            }
        }
    }
    fn set_height(&mut self, height: f32) -> Result<(), f32> {
        match self {
            Backend::Dummy => Err(0.),
            Backend::Raqote(dt) => {
                **dt = DrawTarget::new(dt.width(), height as i32);
                Ok(())
            }
        }
    }
    fn set_size(&mut self, width: f32, height: f32) -> Result<(), (f32, f32)> {
        match self {
            Backend::Dummy => Err((0., 0.)),
            Backend::Raqote(dt) => {
                **dt = DrawTarget::new(width as i32, height as i32);
                Ok(())
            }
        }
    }
}

impl<'b> Deref for Backend<'b> {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        match &self {
            Backend::Raqote(dt) => dt.get_data_u8(),
            _ => panic!("Dummy backend cannot return a slice"),
        }
    }
}

impl<'c> DerefMut for Backend<'c> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Backend::Raqote(dt) => dt.get_data_u8_mut(),
            _ => panic!("Dummy backend cannot return a slice"),
        }
    }
}

impl<'c> SyncContext<'c> {
    pub fn new(model: &'c mut impl Controller, font_cache: &'c mut FontCache) -> Self {
        Self { model, font_cache }
    }
}

impl<'c> Controller for SyncContext<'c> {
    fn deserialize(&mut self, token: u32) -> Result<(), ControllerError> {
        self.model.deserialize(token)
    }
    fn get<'m>(&'m self, msg: Message) -> Result<Data<'m>, ControllerError> {
        self.model.get(msg)
    }
    fn serialize(&mut self, msg: Message) -> Result<u32, ControllerError> {
        self.model.serialize(msg)
    }
    fn send<'m>(&'m mut self, msg: Message) -> Result<Data<'m>, ControllerError> {
        self.model.send(msg)
    }
}

impl<'c> DrawContext<'c> {
    pub fn new(
        backend: Backend<'c>,
        font_cache: &'c mut FontCache,
        pending_damage: &'c mut Vec<Region>,
    ) -> Self {
        Self {
            backend,
            font_cache,
            pending_damage,
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
            _ => unreachable!(),
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
    pub fn draw_image(&mut self, x: f32, y: f32, image: Image) {
        match &mut self.backend {
            Backend::Raqote(dt) => dt.draw_image_at(x, y, &image, &DRAW_OPTIONS),
            _ => {}
        }
    }
    pub fn draw_label(&mut self, label: &Label, x: f32, y: f32) {
        if let Some(layout) = self.font_cache.layouts.get(label) {
            for gp in layout {
                if let Some(glyph_cache) = self
                    .font_cache
                    .fonts
                    .get_mut(&label.fonts[gp.key.font_index as usize])
                {
                    if let Some(pixmap) = glyph_cache.render_glyph(gp) {
                        match &mut self.backend {
                            Backend::Raqote(dt) => dt.draw_image_at(
                                x.round() + gp.x,
                                y.round() + gp.y,
                                &Image {
                                    data: &pixmap,
                                    width: gp.width as i32,
                                    height: gp.height as i32,
                                },
                                &DrawOptions {
                                    blend_mode: BlendMode::SrcAtop,
                                    alpha: 1.,
                                    antialias: AntialiasMode::Gray,
                                },
                            ),
                            _ => {}
                        }
                    }
                }
            }
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
}

impl<'c> Deref for DrawContext<'c> {
    type Target = Backend<'c>;
    fn deref(&self) -> &Self::Target {
        &self.backend
    }
}

impl<'c> DerefMut for DrawContext<'c> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.backend
    }
}
