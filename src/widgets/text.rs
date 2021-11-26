use crate::*;
pub use fontdue::{
    layout,
    layout::{
        CoordinateSystem, GlyphPosition, GlyphRasterConfig, Layout, LayoutSettings, TextStyle,
    },
    Font, FontResult, FontSettings,
};
use raqote::*;
use scene::Instruction;
use widgets::u32_to_source;
use std::hash::{Hash, Hasher};
pub use crate::font::FontProperty;

#[derive(Debug, Clone)]
pub struct Label {
    pub text: String,
    pub font_size: f32,
    pub source: SolidSource,
    pub fonts: Vec<FontProperty>,
    size: Option<(f32, f32)>,
}

impl Hash for Label {
    fn hash<H: Hasher>(&self, state: &mut H) {
        ((self.font_size * 100.) as u32).hash(state);
        self.text.hash(state);
        self.source.a.hash(state);
        self.source.r.hash(state);
        self.source.g.hash(state);
        self.source.b.hash(state);
        for font in &self.fonts {
            font.hash(state);
        }
    }
}

impl PartialEq for Label {
    fn eq(&self, other: &Self) -> bool {
        self.font_size == other.font_size
            && self.text == other.text
            && self.source == other.source
            && self.fonts.len() > 0
            && other.fonts.len() > 0
            && self.fonts[0] == other.fonts[0]
    }
    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl Eq for Label {}

impl Label {
    pub fn new(text: &str, font_size: f32) -> Label {
        Label {
            text: String::from(text),
            font_size,
            fonts: Vec::new(),
            source: u32_to_source(FG),
            size: None,
        }
    }
    pub fn font_property(mut self, font: FontProperty) -> Self {
        self.fonts.push(font);
        self
    }
    pub fn color(mut self, color: u32) -> Self {
        self.source = u32_to_source(color);
        self
    }
    pub fn default(text: &str, font_size: f32) -> Label {
        Label {
            text: String::from(text),
            font_size,
            fonts: vec![FontProperty::new("Default")],
            source: u32_to_source(FG),
            size: None,
        }
    }
}

impl Geometry for Label {
    fn width(&self) -> f32 {
        if let Some((width, _)) = self.size {
            return width;
        }
        0.
    }
    fn height(&self) -> f32 {
        if let Some((_, height)) = self.size {
            return height;
        }
        0.
    }
    fn set_width(&mut self, width: f32) -> Result<(), f32> {
        Err(if let Some(size) = &self.size {
            size.0
        } else { 0. })
    }
    fn set_height(&mut self, height: f32) -> Result<(), f32> {
        Err(if let Some(size) = &self.size {
            size.1
        } else { 0. })
    }
}

impl Widget for Label {
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        RenderNode::Instruction(Instruction::new(x, y, self.clone()))
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext, _event: Event) {
        if self.size.is_none() {
            let size = ctx.font_cache.layout_label(self);
            self.size = Some(size);
        }
    }
}
