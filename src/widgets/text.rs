pub use crate::font::FontProperty;
use crate::{style::FG0, *};
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

#[derive(Clone)]
pub struct Label {
    text: String,
    font_size: f32,
    color: Color,
    settings: LayoutSettings,
    fonts: Vec<FontProperty>,
    layout: Option<Rc<[GlyphPosition]>>,
    size: (f32, f32),
}

impl Label {
    pub fn get_text(&self) -> &str {
        self.text.as_str()
    }
    pub fn get_font_size(&self) -> f32 {
        self.font_size
    }
    pub fn fonts(&self) -> &[FontProperty] {
        &self.fonts
    }
    pub fn max_width(&self) -> f32 {
        self.settings.max_width.unwrap_or(self.size.0)
    }
    pub fn max_height(&self) -> f32 {
        self.settings.max_height.unwrap_or(self.size.1)
    }
    pub fn set_color(&mut self, color: u32) {
        self.color = u32_to_source(color);
    }
    pub fn get_color(&self) -> Color {
        self.color
    }
    pub fn get_settings(&self) -> &LayoutSettings {
        &self.settings
    }
    pub fn get_layout(&self) -> Option<&Rc<[GlyphPosition]>> {
        self.layout.as_ref()
    }
}

impl PartialEq for Label {
    fn eq(&self, other: &Self) -> bool {
        self.font_size == other.font_size
            && self.text == other.text
            && self.color == other.color
            && self.settings == other.settings
            && self.fonts.eq(&other.fonts)
    }
}

impl Eq for Label {}

impl std::fmt::Debug for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Label")
            .field("text", &self.text)
            .field("font_size", &self.font_size)
            .field("color", &self.color)
            .field("fonts", &self.fonts)
            .field("dimension", &self.size)
            .finish()
    }
}

impl From<&str> for Label {
    fn from(text: &str) -> Self {
        Label::default(text, 20.)
    }
}

impl Label {
    pub fn new(text: &str, font_size: f32) -> Label {
        Label {
            text: String::from(text),
            font_size,
            fonts: Vec::new(),
            settings: LayoutSettings::default(),
            color: u32_to_source(FG0),
            layout: None,
            size: (0., 0.),
        }
    }
    pub fn font<F: Into<FontProperty>>(mut self, font: F) -> Self {
        self.fonts.push(font.into());
        self
    }
    pub fn color(mut self, color: u32) -> Self {
        self.color = u32_to_source(color);
        self
    }
    pub fn settings(mut self, settings: LayoutSettings) -> Self {
        self.settings = settings;
        self
    }
    pub fn default(text: &str, font_size: f32) -> Label {
        Label {
            text: String::from(text),
            font_size,
            settings: LayoutSettings::default(),
            fonts: vec![FontProperty::new("sans serif")],
            color: u32_to_source(FG0),
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
        Err(self.size.0)
    }
    fn set_height(&mut self, height: f32) -> Result<(), f32> {
        self.settings.max_height = Some(height);
        Err(self.size.1)
    }
}

impl<M> Widget<M> for Label {
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        RenderNode::Instruction(Instruction::new(x, y, self.clone()))
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<M>, _event: Event<'d, M>) -> Damage {
        if self.layout.is_none() {
            let layout = ctx.font_cache.layout(self).glyphs().clone();
            self.size = font::get_size(&layout);
            self.layout = Some(layout.into());
            Damage::Some
        } else {
            Damage::None
        }
    }
}

pub struct Text {
    label: Label,
    buffer: Option<String>,
    layout: Layout,
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

impl From<&str> for Text {
    fn from(text: &str) -> Self {
        let label: Label = text.into();
        label.into()
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
        self.buffer = None;
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

impl<M> Widget<M> for Text {
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        RenderNode::Instruction(Instruction::new(x, y, self.label.clone()))
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<M>, event: Event<'d, M>) -> Damage {
        if let Some(string) = &self.buffer {
            ctx.font_cache.write(&mut self.layout, &self.label, string);
            let glyphs = self.layout.glyphs().clone();
            self.label.size = font::get_size(&glyphs);
            self.label.layout = Some(glyphs.into());
            self.buffer = None;
            Damage::Some
        } else {
            self.label.sync(ctx, event)
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

use crate::controller::Controller;

// Updates text on messages with a matching id or on Frame.
// The retreived Data will replace all occurences of `{}` in the format.
pub struct Listener<M: PartialEq + TryInto<String>> {
    message: Option<M>,
    poll: bool,
    format: Option<String>,
    text: Text,
}

impl<M: PartialEq + TryInto<String>> Geometry for Listener<M> {
    fn width(&self) -> f32 {
        self.text.width()
    }
    fn height(&self) -> f32 {
        self.text.height()
    }
    fn set_width(&mut self, width: f32) -> Result<(), f32> {
        self.text.set_width(width)
    }
    fn set_height(&mut self, height: f32) -> Result<(), f32> {
        self.text.set_height(height)
    }
}

impl<M: PartialEq + TryInto<String>> Widget<M> for Listener<M> {
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        RenderNode::Instruction(Instruction::new(x, y, self.text.label.clone()))
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<M>, event: Event<'d, M>) -> Damage {
        if let Some(message) = self.message.as_ref() {
            if self.poll {
                if let Ok(msg) = ctx.get(message) {
                    if let Ok(string) = msg.try_into() {
                        if let Some(format) = self.format.as_ref() {
                            self.text.edit(format.replace("{}", &string).as_str());
                        } else {
                            self.text.edit(format!("{}", string).as_str());
                        }
                    }
                }
            } else {
                match event {
                    Event::Message(msg) => {
                        if self.message.as_ref() == Some(msg) {
                            if let Ok(msg) = ctx.get(message) {
                                if let Ok(string) = msg.try_into() {
                                    if let Some(format) = self.format.as_ref() {
                                        self.text.edit(format.replace("{}", &string).as_str());
                                    } else {
                                        self.text.edit(format!("{}", string).as_str());
                                    }
                                }
                            }
                        }
                    }
                    Event::Frame => {
                        if let Ok(msg) = ctx.get(message) {
                            if let Ok(string) = msg.try_into() {
                                if let Some(format) = self.format.as_ref() {
                                    self.text.edit(format.replace("{}", &string).as_str());
                                } else {
                                    self.text.edit(format!("{}", string).as_str());
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        self.text.sync(ctx, event)
    }
}

impl<M: PartialEq + TryInto<String>> From<Text> for Listener<M> {
    fn from(text: Text) -> Self {
        Self {
            message: None,
            text,
            poll: false,
            format: None,
        }
    }
}

impl<M: PartialEq + TryInto<String>> From<Label> for Listener<M> {
    fn from(label: Label) -> Self {
        Self {
            message: None,
            text: label.into(),
            poll: false,
            format: None,
        }
    }
}

impl<M: PartialEq + TryInto<String>> Listener<M> {
    pub fn message(mut self, message: M) -> Self {
        self.message = Some(message);
        self
    }
    pub fn format(mut self, format: &str) -> Self {
        self.format = Some(format.to_string());
        self
    }
    pub fn poll(mut self) -> Self {
        self.poll = true;
        self
    }
}

impl<M: PartialEq + TryInto<String>> Deref for Listener<M> {
    type Target = Text;
    fn deref(&self) -> &Self::Target {
        &self.text
    }
}

impl<M: PartialEq + TryInto<String>> DerefMut for Listener<M> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.text
    }
}
