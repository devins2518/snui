use crate::cache::font::FontProperty;
use crate::{theme::FG0, *};
use fontdue::layout::LayoutSettings;
use std::ops::{Deref, DerefMut};
use tiny_skia::*;

const DEFAULT_LAYOUT_SETTINGS: LayoutSettings = LayoutSettings {
    x: 0.,
    y: 0.,
    max_width: None,
    max_height: None,
    horizontal_align: fontdue::layout::HorizontalAlign::Left,
    vertical_align: fontdue::layout::VerticalAlign::Top,
    wrap_style: fontdue::layout::WrapStyle::Word,
    wrap_hard_breaks: true,
};

const DEFAULT_FONT_SIZE: f32 = 15.;

pub const TEXT: PixmapPaint = PixmapPaint {
    blend_mode: BlendMode::SourceAtop,
    opacity: 1.0,
    quality: FilterQuality::Bilinear,
};

/// Owned text widget
#[derive(Clone)]
pub struct Label {
    pub(crate) text: String,
    pub(crate) font_size: f32,
    pub(crate) color: Color,
    pub(crate) settings: LayoutSettings,
    pub(crate) fonts: [FontProperty; 2],
    pub(crate) size: Option<Size>,
}

/// A reference to a Label.
///
/// It can also be used to to layout text from non owned data.
#[derive(Copy, Clone, PartialEq)]
pub struct LabelRef<'s> {
    pub text: &'s str,
    pub font_size: f32,
    pub color: Color,
    pub settings: &'s LayoutSettings,
    pub fonts: &'s [FontProperty],
    pub size: Option<Size>,
}

impl<'s> LabelRef<'s> {
    pub fn new(text: &'s str, fonts: &'s [FontProperty]) -> Self {
        LabelRef {
            text,
            font_size: DEFAULT_FONT_SIZE,
            fonts,
            settings: &DEFAULT_LAYOUT_SETTINGS,
            color: to_color(FG0),
            size: None,
        }
    }
}

impl Label {
    pub fn new<T: Into<String>>(text: T) -> Label {
        Label {
            text: text.into(),
            font_size: DEFAULT_FONT_SIZE,
            fonts: Default::default(),
            settings: DEFAULT_LAYOUT_SETTINGS,
            color: to_color(FG0),
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
            size: self.size,
        }
    }
    pub fn as_str(&self) -> &str {
        self.text.as_str()
    }
    pub fn set_color(&mut self, color: u32) {
        self.color = to_color(color);
    }
    pub fn write(&mut self, s: &str) {
        self.text.push_str(s);
        self.size = None;
    }
    pub fn edit(&mut self, s: &str) {
        if s.ne(self.text.as_str()) {
            self.text.replace_range(0.., s);
            self.size = None;
        }
    }
    pub fn primary_font<F: Into<FontProperty>>(mut self, font: F) -> Self {
        self.fonts[0] = font.into();
        self
    }
    pub fn secondary_font<F: Into<FontProperty>>(mut self, font: F) -> Self {
        self.fonts[1] = font.into();
        self
    }
    pub fn color(mut self, color: u32) -> Self {
        self.color = to_color(color);
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

impl PartialEq for Label {
    fn eq(&self, other: &Self) -> bool {
        self.font_size == other.font_size
            && self.text == other.text
            && self.color == other.color
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
            .finish()
    }
}

impl<T> From<T> for Label
where
    T: ToString,
{
    fn from(label: T) -> Self {
        Label::new(label.to_string())
    }
}

impl<'s> Geometry for LabelRef<'s> {
    fn width(&self) -> f32 {
        self.size.unwrap_or_default().width
    }
    fn height(&self) -> f32 {
        self.size.unwrap_or_default().height
    }
}

impl<'s> Primitive for LabelRef<'s> {
    fn draw(&self, context: &mut DrawContext, transform: tiny_skia::Transform) {
        let mut label = *self;
        let mut settings = *label.settings;
        let font_cache = &mut context.cache.font_cache;
        settings.max_width = self.settings.max_width.map(|width| width * transform.sx);
        settings.max_height = self.settings.max_height.map(|height| height * transform.sy);

        let clip_mask = context
            .clipmask
            .as_ref()
            .and_then(|clipmask| (!clipmask.is_empty()).then(|| &**clipmask));

        let x = transform.tx.round();
        let y = transform.ty.round();

        for gp in {
            label.font_size = self.font_size * transform.sy;
            label.settings = &settings;
            font_cache.layout(&label);
            font_cache.layout.glyphs()
        } {
            if let Some(glyph_cache) = font_cache.fonts.get_mut(&self.fonts[gp.font_index]) {
                if let Some(pixmap) = glyph_cache.get(gp) {
                    if let Some(pixmap) =
                        PixmapRef::from_bytes(pixmap, gp.width as u32, gp.height as u32)
                    {
                        match &mut context.backend {
                            Backend::Pixmap(dt) => {
                                dt.draw_pixmap(
                                    (x + gp.x) as i32,
                                    (y + gp.y) as i32,
                                    pixmap,
                                    &TEXT,
                                    Transform::identity(),
                                    clip_mask,
                                );
                            }
                            _ => (),
                        }
                    }
                }
            }
        }
    }
}

impl<T> Widget<T> for Label {
    fn draw_scene(&mut self, mut scene: Scene) {
        scene.insert_primitive(&self.as_ref())
    }
    fn event<'s>(&'s mut self, _ctx: &mut UpdateContext<T>, _event: Event<'s>) -> Damage {
        self.size.map(|_| Damage::None).unwrap_or(Damage::Partial)
    }
    fn update<'s>(&'s mut self, _ctx: &mut UpdateContext<T>) -> Damage {
        self.size.map(|_| Damage::None).unwrap_or(Damage::Partial)
    }
    fn layout(&mut self, ctx: &mut LayoutCtx, constraints: &BoxConstraints) -> Size {
        let fc: &mut cache::FontCache = ctx.as_mut().as_mut();
        if !constraints.is_default() {
            self.settings.max_width = Some(constraints.maximum_width());
            self.settings.max_height = Some(constraints.maximum_height());
        }
        self.size = Some(crate::cache::font::get_size(fc.layout(&self.as_ref())).into());
        self.size.unwrap_or_default()
    }
}

use crate::mail::*;

/// Updates the inner label on sync or configure.
///
/// The text is fetched from the Data using the provided message.
pub struct Listener<M> {
    message: M,
    label: Proxy<Label>,
}

impl<M> Listener<M> {
    pub fn new<T: Into<Label>>(label: T, message: M) -> Self {
        Self {
            message,
            label: Proxy::new(label.into()),
        }
    }
}

impl<M, T> Widget<T> for Listener<M>
where
    M: Clone + Copy,
    T: for<'a, 's> Mail<'a, M, &'s str, &'a str>,
{
    fn draw_scene(&mut self, scene: Scene) {
        Widget::<()>::draw_scene(&mut self.label, scene)
    }
    fn update<'s>(&'s mut self, ctx: &mut UpdateContext<T>) -> Damage {
        if let Some(string) = ctx.send(self.message, self.label.as_str()) {
            self.label.edit(string);
        } else if let Some(string) = ctx.get(self.message) {
            self.label.edit(string);
        }
        self.label.update(ctx)
    }
    fn event<'s>(&'s mut self, ctx: &mut UpdateContext<T>, event: Event<'s>) -> Damage {
        self.label.event(ctx, event)
    }
    fn layout(&mut self, ctx: &mut LayoutCtx, constraints: &BoxConstraints) -> Size {
        Widget::<()>::layout(&mut self.label, ctx, constraints)
    }
}

impl<M> Deref for Listener<M> {
    type Target = Label;
    fn deref(&self) -> &Self::Target {
        &self.label
    }
}

impl<M> DerefMut for Listener<M> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.label
    }
}
