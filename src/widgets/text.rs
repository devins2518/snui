pub use crate::cache::font::FontProperty;
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

const DEFAULT_FONT_SIZE: f32 = 15.;

#[derive(Clone)]
pub struct Label {
    pub(crate) text: String,
    pub(crate) font_size: f32,
    pub(crate) color: Color,
    pub(crate) settings: LayoutSettings,
    pub(crate) fonts: Vec<FontProperty>,
    pub(crate) layout: Option<Rc<[GlyphPosition]>>,
    pub(crate) size: (f32, f32),
}

impl Label {
    pub fn get_text(&self) -> &str {
        self.text.as_str()
    }
    pub fn get_font_size(&self) -> f32 {
        self.font_size
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
        Label::default(text)
    }
}

impl Label {
    pub fn new<T: Into<String>>(text: T) -> Label {
        Label {
            text: text.into(),
            font_size: DEFAULT_FONT_SIZE,
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
    pub fn font_size(mut self, font_size: f32) -> Self {
        self.font_size = font_size;
        self
    }
    pub fn settings(mut self, settings: LayoutSettings) -> Self {
        self.settings = settings;
        self
    }
    pub fn default<T: Into<String>>(text: T) -> Label {
        Label {
            text: text.into(),
            font_size: DEFAULT_FONT_SIZE,
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
    fn create_node(&mut self, transform: Transform) -> RenderNode {
        if transform.sx != 1. || transform.sy != 1. {
            let mut label = self
                .clone()
                .font_size(transform.sx.min(transform.sy) * self.font_size);
            label.layout = None;
            Instruction::new(transform, label).into()
        } else {
            Instruction::new(transform, self.clone()).into()
        }
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<M>, _event: Event<'d, M>) -> Damage {
        if self.layout.is_none() {
            let layout = ctx.font_cache().layout(self).clone();
            self.size = cache::font::get_size(&layout);
            self.layout = Some(layout.into());
            Damage::Partial
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
    pub fn write<S: ToString>(&mut self, s: S) {
        let s = s.to_string();
        self.label.text.push_str(&s);
        if let Some(buf) = self.buffer.as_mut() {
            buf.push_str(&s);
        } else {
            self.buffer = Some(s);
        }
    }
    pub fn edit<S: ToString>(&mut self, s: S) {
        let s = s.to_string();
        if s.ne(self.label.text.as_str()) {
            self.label.layout = None;
        }
        self.buffer = None;
        self.label.text = s;
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
    fn create_node(&mut self, transform: Transform) -> RenderNode {
        Widget::<()>::create_node(&mut self.label, transform)
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<M>, event: Event<'d, M>) -> Damage {
        if let Some(string) = &self.buffer {
            ctx.font_cache()
                .write(&mut self.layout, &self.label, string);
            let glyphs = self.layout.glyphs().clone();
            self.label.size = cache::font::get_size(&glyphs);
            self.label.layout = Some(glyphs.into());
            self.buffer = None;
            Damage::Partial
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

/// Updates text on Message or on Prepare events.
pub struct Listener<M: TryInto<String>> {
    message: Option<M>,
    format: Option<String>,
    text: Text,
}

impl<M: TryInto<String>> Geometry for Listener<M> {
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

impl<M: TryInto<String>> Widget<M> for Listener<M> {
    fn create_node(&mut self, transform: Transform) -> RenderNode {
        Widget::<()>::create_node(&mut self.text, transform)
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<M>, event: Event<'d, M>) -> Damage {
        match event {
            Event::Message(_) | Event::Prepare => {
                if let Some(message) = self.message.as_ref() {
                    if let Ok(msg) = ctx.get(message) {
                        if let Ok(string) = msg.try_into() {
                            if let Some(format) = self.format.as_ref() {
                                self.text.edit(format.replace("{}", &string));
                            } else {
                                self.text.edit(format!("{}", string));
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        self.text.sync(ctx, event)
    }
}

impl<M, T> From<T> for Listener<M>
where
    M: TryInto<String>,
    T: Into<Text>,
{
    fn from(text: T) -> Self {
        Self {
            message: None,
            text: text.into(),
            format: None,
        }
    }
}

impl<M: TryInto<String>> Listener<M> {
    pub fn message(mut self, message: M) -> Self {
        self.message = Some(message);
        self
    }
    pub fn format(mut self, format: &str) -> Self {
        self.format = Some(format.to_string());
        self
    }
}

impl<M: TryInto<String>> Deref for Listener<M> {
    type Target = Text;
    fn deref(&self) -> &Self::Target {
        &self.text
    }
}

impl<M: TryInto<String>> DerefMut for Listener<M> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.text
    }
}
