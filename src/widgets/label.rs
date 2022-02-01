pub use crate::cache::font::FontProperty;
use crate::{theme::FG0, *};
pub use fontdue::{
    layout,
    layout::{
        CoordinateSystem, GlyphPosition, GlyphRasterConfig, Layout, LayoutSettings, TextStyle,
    },
    Font, FontResult, FontSettings,
};
use scene::Instruction;
use std::ops::{Deref, DerefMut};
use tiny_skia::*;

const DEFAULT_FONT_SIZE: f32 = 15.;

#[derive(Clone)]
pub struct Label {
    pub(crate) text: String,
    pub(crate) font_size: f32,
    pub(crate) color: Color,
    pub(crate) settings: LayoutSettings,
    pub(crate) fonts: Vec<FontProperty>,
    pub(crate) size: Option<(f32, f32)>,
}

#[derive(Copy, Clone, PartialEq)]
pub struct LabelRef<'s> {
    pub text: &'s str,
    pub font_size: f32,
    pub color: Color,
    pub settings: &'s LayoutSettings,
    pub fonts: &'s [FontProperty],
}

impl Label {
    pub fn as_str(&self) -> &str {
        self.text.as_str()
    }
    pub fn max_width(&self) -> f32 {
        self.settings
            .max_width
            .unwrap_or(self.size.unwrap_or_default().0)
    }
    pub fn max_height(&self) -> f32 {
        self.settings
            .max_height
            .unwrap_or(self.size.unwrap_or_default().1)
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
            size: None,
        }
    }
    pub fn default<T: Into<String>>(text: T) -> Label {
        Label {
            text: text.into(),
            font_size: DEFAULT_FONT_SIZE,
            settings: LayoutSettings::default(),
            fonts: vec![FontProperty::new("sans serif")],
            color: u32_to_source(FG0),
            size: None,
        }
    }
    pub fn as_ref(&self) -> LabelRef {
        LabelRef {
            color: self.color,
            text: self.text.as_str(),
            font_size: self.font_size,
            settings: &self.settings,
            fonts: self.fonts.as_slice(),
        }
    }
    pub fn write(&mut self, s: &str) {
        self.text.push_str(s);
        self.size = None;
    }
    pub fn edit<S: ToString>(&mut self, s: S) {
        let s = s.to_string();
        if s.ne(self.text.as_str()) {
            self.text = s;
            self.size = None;
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
}

impl Geometry for Label {
    fn width(&self) -> f32 {
        self.size.unwrap_or_default().0
    }
    fn height(&self) -> f32 {
        self.size.unwrap_or_default().1
    }
    fn set_width(&mut self, width: f32) -> Result<(), f32> {
        self.settings.max_width = Some(width);
        Err(self.width())
    }
    fn set_height(&mut self, height: f32) -> Result<(), f32> {
        self.settings.max_height = Some(height);
        Err(self.height())
    }
}

impl<D> Widget<D> for Label {
    fn create_node(&mut self, transform: Transform) -> RenderNode {
        let scale = transform.sx.max(transform.sy);
        let label = self.clone().font_size(scale * self.font_size);
        Instruction::new(transform, label).into()
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<D>, _event: Event<'d>) -> Damage {
        if self.size.is_none() {
            let fc: &mut cache::FontCache = ctx.as_mut().as_mut();
            let layout = fc.layout(self.as_ref()).clone();
            self.size = Some(cache::font::get_size(&layout));
            Damage::Partial
        } else {
            Damage::None
        }
    }
}

use crate::post::*;

/// Updates text on Post or on Prepare events.
pub struct Listener<M, F>
where
    F: Fn(String) -> String,
{
    message: M,
    label: Proxy<Label>,
    format: F,
}

impl<M, F> Geometry for Listener<M, F>
where
    F: Fn(String) -> String,
{
    fn width(&self) -> f32 {
        self.label.width()
    }
    fn height(&self) -> f32 {
        self.label.height()
    }
    fn set_width(&mut self, width: f32) -> Result<(), f32> {
        self.label.set_width(width)
    }
    fn set_height(&mut self, height: f32) -> Result<(), f32> {
        self.label.set_height(height)
    }
}

impl<M, F, D> Widget<D> for Listener<M, F>
where
    M: Clone + Copy,
    F: Fn(String) -> String,
    D: Post<M, (), String>,
{
    fn create_node(&mut self, transform: Transform) -> RenderNode {
        Widget::<()>::create_node(&mut self.label, transform)
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<D>, event: Event<'d>) -> Damage {
        match event {
            Event::Sync | Event::Prepare => {
                if let Some(string) = ctx.get(self.message) {
                    let fmt = &self.format;
                    self.label.edit(fmt(string));
                }
            }
            _ => {}
        }
        self.label.sync(ctx, event)
    }
}

impl<M, F> Listener<M, F>
where
    F: Fn(String) -> String,
{
    pub fn new<T: Into<Label>>(label: T, message: M, format: F) -> Self {
        Self {
            message,
            label: Proxy::new(label.into()),
            format,
        }
    }
}

impl<M, F> Deref for Listener<M, F>
where
    F: Fn(String) -> String,
{
    type Target = Label;
    fn deref(&self) -> &Self::Target {
        &self.label
    }
}

impl<M, F> DerefMut for Listener<M, F>
where
    F: Fn(String) -> String,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.label
    }
}
