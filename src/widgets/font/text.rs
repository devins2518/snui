use crate::widgets::font::FontProperty;
use crate::*;
use context::Backend;
pub use fontdue::{
    layout,
    layout::{
        CoordinateSystem, GlyphPosition, GlyphRasterConfig, Layout, LayoutSettings, TextStyle,
    },
    Font, FontResult, FontSettings,
};
use raqote::*;
use scene::Instruction;
use std::hash::{Hash, Hasher};
use widgets::u32_to_source;

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
            fonts: vec![FontProperty::new("Default")],
            source: u32_to_source(FG),
            size: None,
        }
    }
    pub fn default() -> Label {
        Label {
            text: String::new(),
            font_size: 0.,
            fonts: Vec::new(),
            source: u32_to_source(0),
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
    fn set_size(&mut self, width: f32, height: f32) -> Result<(), (f32, f32)> {
        Err(self.size.unwrap_or((0., 0.)))
    }
}

impl Primitive for Label {
    fn same(&self, other: &dyn std::any::Any) -> bool {
        compare(self, other)
    }
    fn to_background(&self) -> Background {
        Background::Transparent
    }
    fn draw(&self, x: f32, y: f32, ctx: &mut Context) {
        if let Some(layout) = ctx.font_cache.layouts.get(self) {
            for gp in layout {
                if let Some(glyph_cache) = ctx
                    .font_cache
                    .fonts
                    .get_mut(&self.fonts[gp.key.font_index as usize])
                {
                    if let Some(pixmap) = glyph_cache.render_glyph(gp) {
                        match &mut ctx.backend {
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
}

impl Widget for Label {
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        RenderNode::Instruction(Instruction::new(x, y, self.clone()))
    }
    fn sync<'d>(&'d mut self, ctx: &mut Context, event: Event) {
        if self.size.is_none() {
            let size = ctx.font_cache.layout_label(self);
            self.size = Some(size);
        }
    }
}
