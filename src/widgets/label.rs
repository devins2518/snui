use crate::*;
use fontdue::{
    layout,
    layout::{CoordinateSystem, GlyphRasterConfig, Layout, LayoutSettings, TextStyle},
    Font,
};
use raqote::*;
use std::rc::Rc;
use std::fs::read;
use std::path::Path;
use std::cell::RefCell;

const DRAW_OPTIONS: DrawOptions = DrawOptions {
    blend_mode: BlendMode::SrcOver,
    alpha: 1.,
    antialias: AntialiasMode::Gray
};

#[derive(Clone)]
pub struct Label {
    color: u32,
    damaged: bool,
    font_size: f32,
    size: (f32, f32),
    font: fontdue::Font,
    layout: Rc<RefCell<Layout>>,
}

#[derive(Debug, Clone)]
pub struct Glyph {
    color: u32,
    position: (f32, f32),
    config: GlyphRasterConfig,
    coverage: Vec<u8>,
    metrics: fontdue::Metrics,
}

static DEFAULT: LayoutSettings = LayoutSettings {
    x: 0.0,
    y: 0.0,
    max_width: None,
    max_height: None,
    horizontal_align: layout::HorizontalAlign::Left,
    vertical_align: layout::VerticalAlign::Middle,
    wrap_style: layout::WrapStyle::Word,
    wrap_hard_breaks: false,
};

impl Glyph {
    fn new<'f>(font: &'f Font, config: GlyphRasterConfig, color: u32, x: f32, y: f32) -> Glyph {
        let (metrics, coverage) = font.rasterize_config(config);
        Glyph {
            position: (x.round(), y.round()),
            color,
            config,
            coverage,
            metrics,
        }
    }
}

impl Drawable for Glyph {
    fn set_color(&mut self, color: u32) {
        self.color = color;
    }
    fn draw(&self, canvas: &mut Canvas, x: f32, y: f32) {
        let color = self.color.to_be_bytes();
        let mut source = SolidSource {
            a: color[0],
            r: color[1],
            g: color[2],
            b: color[3],
        };
        let pixmap: Vec<u32> = self
            .coverage
            .iter()
            .map(|a| {
                if a == &0 {
                    0
                } else {
                    source.a = *a;
                    SolidSource::from_unpremultiplied_argb(*a, color[1], color[2], color[3])
                        .to_u32()
                }
            })
            .collect();
        let image = Image {
            width: self.metrics.width as i32,
            height: self.metrics.height as i32,
            data: &pixmap,
        };
        canvas.target.draw_image_at(
            x.round() + self.position.0,
            y.round() + self.position.1,
            &image,
            &DRAW_OPTIONS,
        );
    }
}

impl Label {
    pub fn default(path: &Path, font_size: f32, color: u32) -> Label {
        let font = read(path).unwrap();
        Label {
            color,
            damaged: true,
            size: (0., 0.),
            font_size,
            font: Font::from_bytes(font, fontdue::FontSettings::default()).unwrap(),
            layout: Rc::new(RefCell::new(Layout::new(CoordinateSystem::PositiveYDown))),
        }
    }
    pub fn new(text: &str, path: &Path, font_size: f32, color: u32) -> Label {
        let font = read(path).unwrap();
        let font = fontdue::Font::from_bytes(font, fontdue::FontSettings::default()).unwrap();
        let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
        layout.append(&[&font], &TextStyle::new(text, font_size, 0));
        Label {
            font,
            color,
            damaged: true,
            font_size,
            size: (
                {
                    let mut w = 0;
                    for gp in layout.glyphs().iter() {
                        if w < gp.width + gp.x as usize {
                            w = gp.width + gp.x as usize
                        }
                    }
                    w as f32
                },
                layout.height() as f32,
            ),
            layout: Rc::new(RefCell::new(layout)),
        }
    }
    pub fn from(text: &str, font: &[u8], font_size: f32, color: u32) -> Label {
        let font = fontdue::Font::from_bytes(font, fontdue::FontSettings::default()).unwrap();
        let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
        layout.append(&[&font], &TextStyle::new(text, font_size, 0));
        Label {
            font,
            color,
            damaged: true,
            font_size,
            size: (
                {
                    let mut w = 0;
                    for gp in layout.glyphs().iter() {
                        if w < gp.width + gp.x as usize {
                            w = gp.width + gp.x as usize
                        }
                    }
                    w as f32
                },
                layout.height() as f32,
            ),
            layout: Rc::new(RefCell::new(layout)),
        }
    }
    pub fn max_width<'f>(
        text: &'f str,
        path: &Path,
        font_size: f32,
        width: f32,
        color: u32,
    ) -> Label {
        let font = read(path).unwrap();
        // Parse it into the font type.
        let mut layout_setting = DEFAULT;
        layout_setting.max_width = Some(width);
        let font = fontdue::Font::from_bytes(font, fontdue::FontSettings::default()).unwrap();
        let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
        layout.reset(&layout_setting);
        layout.append(&[&font], &TextStyle::new(text, font_size, 0));

        Label {
            font,
            color,
            damaged: true,
            font_size,
            size: (width, layout.height() as f32),
            layout: Rc::new(RefCell::new(layout)),
        }
    }
    pub fn max_height<'f>(
        text: &'f str,
        path: &Path,
        font_size: f32,
        height: f32,
        color: u32,
    ) -> Label {
        let font = read(path).unwrap();
        // Parse it into the font type.
        let mut layout_setting = DEFAULT;
        layout_setting.max_height = Some(height);
        let font = fontdue::Font::from_bytes(font, fontdue::FontSettings::default()).unwrap();
        let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
        layout.reset(&layout_setting);
        layout.append(&[&font], &TextStyle::new(text, font_size, 0));

        Label {
            font,
            color,
            damaged: true,
            font_size,
            size: (
                {
                    let mut w = 0;
                    for gp in layout.glyphs().iter() {
                        if w < gp.width + gp.x as usize {
                            w = gp.width + gp.x as usize
                        }
                    }
                    w as f32
                },
                layout.height() as f32,
            ),
            layout: Rc::new(RefCell::new(layout)),
        }
    }
    pub fn write(&mut self, text: &str) {
        let font = &self.font;
        let mut layout = self.layout.borrow_mut();
        layout.append(&[font], &TextStyle::new(text, self.font_size as f32, 0));
        self.size.0 = 0.;
        for gp in layout.glyphs().iter() {
            if self.size.0 < gp.width as f32 + gp.x as f32 {
                self.size.0 = gp.width as f32 + gp.x as f32
            }
        }
        self.size.1 = layout.height() as f32;
    }
    pub fn edit(&mut self, text: &str) {
        let font = &self.font;
        let mut layout = self.layout.borrow_mut();
        layout.reset(&DEFAULT);
        layout.append(&[font], &TextStyle::new(text, self.font_size as f32, 0));
        self.size.0 = 0.;
        for gp in layout.glyphs().iter() {
            if self.size.0 < gp.width as f32 + gp.x as f32 {
                self.size.0 = gp.width as f32 + gp.x as f32
            }
        }
        self.size.1 = layout.height() as f32;
    }
}

impl Geometry for Label {
    fn width(&self) -> f32 {
        self.size.0
    }
    fn height(&self) -> f32 {
        self.size.1
    }
}

impl Drawable for Label {
    fn set_color(&mut self, color: u32) {
        self.color = color;
    }
    fn draw(&self, canvas: &mut Canvas, x: f32, y: f32) {
        canvas.push(x, y, self, false);
        for glyph in self.layout.borrow_mut().glyphs() {
            Glyph::new(&self.font, glyph.key, self.color, glyph.x, glyph.y).draw(canvas, x, y);
        }
    }
}

impl Widget for Label {
    fn damaged(&self) -> bool {
        self.damaged
    }
    fn roundtrip<'d>(
        &'d mut self,
        _widget_x: f32,
        _widget_y: f32,
        _dispatched: &Dispatch,
    ) -> Option<Damage> {
        None
    }
}
