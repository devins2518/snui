use crate::*;
use lyon_geom::euclid::{point2, vec2, Angle};
use raqote::*;
use scene::*;
use std::ops::{Deref, DerefMut};
use widgets::font::*;
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

enum Data<'d> {
    Null,
    Int(i32),
    Uint(u32),
    Float(f32),
    Boolean(bool),
    String(&'d str),
    Node(RenderNode),
    Any(&'d dyn std::any::Any)
}

struct Message<'m> (
    &'m str,
    Data<'m>
);

enum ReceiveError {
    Block
}

const CAPABILITIES: [&str; 1] = ["example"];

trait Model {
    // These interface are from the pov
    // of the widgets
    fn receive<'m>(&'m self, msg: Message) -> Option<Data<'m>>;
    fn send<'m>(&'m mut self, msg: Message) -> Result<Data<'m>, ReceiveError>;
    fn interface(&self) -> &[&str];
}

pub enum Backend<'b> {
    Raqote(&'b mut DrawTarget),
    Dummy,
}

struct DefaultModel {}

impl Model for DefaultModel {
    fn receive<'m>(&'m self, msg: Message) -> Option<Data<'m>> {
        None
    }
    fn send<'m>(&'m mut self, msg: Message) -> Result<Data<'m>, ReceiveError> {
        Ok(Data::Null)
    }
    fn interface(&self) -> &[&str] {&CAPABILITIES}
}

pub struct SyncContext<'c> {
    model: Option<&'c mut dyn Model>,
    pub font_cache: &'c mut FontCache,
    pub render_node: Option<&'c mut RenderNode>,
}

pub struct DrawContext<'c> {
    pub backend: Backend<'c>,
    pub font_cache: &'c mut FontCache,
    pending_damage: &'c mut Vec<Region>,
}

impl<'c> SyncContext<'c> {
    pub fn new(render_node: Option<&'c mut RenderNode>, font_cache: &'c mut FontCache) -> Self {
        Self {
            model: None,
            render_node,
            font_cache
        }
    }
}

impl<'c> DrawContext<'c> {
    pub fn new(backend: Backend<'c>, font_cache: &'c mut FontCache, pending_damage: &'c mut Vec<Region>) -> Self {
        Self {
            backend,
            font_cache,
            pending_damage
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
}

impl<'c> Geometry for DrawContext<'c> {
    fn set_size(&mut self, width: f32, height: f32) -> Result<(), (f32, f32)> {
        match &mut self.backend {
            Backend::Raqote(dt) => {
                **dt = DrawTarget::new(width as i32, height as i32);
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

impl<'c> Deref for DrawContext<'c> {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        match &self.backend {
            Backend::Raqote(dt) => dt.get_data_u8(),
            _ => panic!("Dummy backend cannot return a slice"),
        }
    }
}

impl<'c> DerefMut for DrawContext<'c> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match &mut self.backend {
            Backend::Raqote(dt) => dt.get_data_u8_mut(),
            _ => panic!("Dummy backend cannot return a slice"),
        }
    }
}
