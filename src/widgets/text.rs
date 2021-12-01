pub use crate::font::FontProperty;
use crate::*;
pub use fontdue::{
    layout,
    layout::{
        CoordinateSystem, GlyphPosition, GlyphRasterConfig, Layout, LayoutSettings, TextStyle,
    },
    Font, FontResult, FontSettings,
};
use scene::Instruction;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use tiny_skia::*;
use widgets::u32_to_source;

#[derive(Clone)]
pub struct Label {
    text: String,
    font_size: f32,
    source: Color,
    settings: LayoutSettings,
    fonts: Vec<FontProperty>,
    layout: Option<Rc<Vec<GlyphPosition>>>,
    size: (f32, f32),
}

pub struct Text {
    label: Label,
    buffer: Option<String>,
    layout: Layout,
}

impl Label {
    pub fn text(&self) -> &str {
        self.text.as_str()
    }
    pub fn font_size(&self) -> f32 {
        self.font_size
    }
    pub fn fonts(&self) -> &[FontProperty] {
        &self.fonts
    }
    pub fn max_width(&self) -> f32 {
        self.settings.max_width.unwrap_or_default().max(self.size.0)
    }
    pub fn max_height(&self) -> f32 {
        self.settings
            .max_height
            .unwrap_or_default()
            .max(self.size.1)
    }
    pub fn set_color(&mut self, color: u32) {
        self.source = u32_to_source(color);
    }
    pub fn source(&self) -> Color {
        self.source
    }
    pub fn settings(&self) -> &LayoutSettings {
        &self.settings
    }
    pub fn layout(&self) -> Option<&Rc<Vec<GlyphPosition>>> {
        self.layout.as_ref()
    }
}

impl PartialEq for Label {
    fn eq(&self, other: &Self) -> bool {
        self.font_size == other.font_size
            && self.text == other.text
            && self.source == other.source
            && self.settings == other.settings
            && self.fonts.eq(&other.fonts)
    }
    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl Eq for Label {}

impl std::fmt::Debug for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Label")
            .field("text", &self.text)
            .field("font_size", &self.font_size)
            .field("source", &self.source)
            .field("fonts", &self.fonts)
            .finish()
    }
}

impl Label {
    pub fn new(text: &str, font_size: f32) -> Label {
        Label {
            text: String::from(text),
            font_size,
            fonts: Vec::new(),
            settings: LayoutSettings::default(),
            source: u32_to_source(FG),
            layout: None,
            size: (0., 0.),
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
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.set_size(width, height).unwrap();
        self
    }
    pub fn default(text: &str, font_size: f32) -> Label {
        Label {
            text: String::from(text),
            font_size,
            settings: LayoutSettings::default(),
            fonts: vec![FontProperty::new("Default")],
            source: u32_to_source(FG),
            layout: None,
            size: (0., 0.),
        }
    }
}

impl Geometry for Label {
    fn width(&self) -> f32 {
        self.size.0
    }
    fn height(&self) -> f32 {
        self.size.1
    }
    fn set_width(&mut self, width: f32) -> Result<(), f32> {
        self.settings.max_width = Some(width);
        self.layout = None;
        Err(self.size.0)
    }
    fn set_height(&mut self, height: f32) -> Result<(), f32> {
        self.settings.max_height = Some(height);
        self.layout = None;
        Err(self.size.1)
    }
}

impl Widget for Label {
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        RenderNode::Instruction(Instruction::new(x, y, self.clone()))
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext, _event: Event) {
        if self.layout.is_none() {
            let layout = ctx.font_cache.layout(self).glyphs().clone();
            self.size = font::get_size(&layout);
            self.layout = Some(Rc::new(layout));
        }
    }
}

impl From<Label> for Text {
    fn from(label: Label) -> Self {
        Text {
            label,
            buffer: None,
            layout: Layout::new(CoordinateSystem::PositiveYDown),
        }
    }
}

impl Text {
    pub fn write(&mut self, s: &str) {
        self.label.text.push_str(s);
        if let Some(buf) = self.buffer.as_mut() {
            buf.push_str(s);
        } else {
            self.buffer = Some(s.to_string());
        }
    }
    pub fn edit(&mut self, s: &str) {
        if s.ne(self.label.text.as_str()) {
            self.label.layout = None;
        }
        self.label.text = s.to_string();
    }
}

impl Geometry for Text {
    fn height(&self) -> f32 {
        self.label.height()
    }
    fn width(&self) -> f32 {
        self.label.width()
    }
    fn set_width(&mut self, width: f32) -> Result<(), f32> {
        self.label.set_width(width)
    }
    fn set_height(&mut self, height: f32) -> Result<(), f32> {
        self.label.set_height(height)
    }
}

impl Widget for Text {
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        self.label.create_node(x, y)
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext, event: Event) {
        if let Some(string) = &self.buffer {
            ctx.font_cache.write(&mut self.layout, &self.label, string);
            let glyphs = self.layout.glyphs().clone();
            self.label.size = font::get_size(&glyphs);
            self.label.layout = Some(Rc::new(glyphs));
            self.buffer = None;
        } else {
            self.label.sync(ctx, event);
        }
    }
}

impl Deref for Text {
    type Target = Label;
    fn deref(&self) -> &Self::Target {
        &self.label
    }
}

impl DerefMut for Text {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.label
    }
}
