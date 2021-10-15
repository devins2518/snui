use crate::*;
use crate::canvas::Backend;
pub use fontdue::{
    layout,
    Font,
    FontSettings,
    layout::{CoordinateSystem, GlyphRasterConfig, Layout, LayoutSettings, TextStyle},
};
use raqote::*;
use std::fs::read;
use std::path::Path;
use crate::widgets::primitives::WidgetShell;

const DRAW_OPTIONS: DrawOptions = DrawOptions {
    blend_mode: BlendMode::SrcOver,
    alpha: 1.,
    antialias: AntialiasMode::Gray,
};

pub fn font_from_path(path: &Path) -> Font {
    let font = read(path).unwrap();
    Font::from_bytes(font, fontdue::FontSettings::default()).unwrap()
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
        // TO-DO have draw_image be a method of Canvas
        canvas.draw_image(
            x.round() + self.position.0,
            y.round() + self.position.1,
            image,
        );
    }
}

#[derive(Clone, Debug)]
pub struct Label {
    damaged: bool,
    width: f32,
    height: f32,
    glyphs: Vec<Glyph>,
}

impl Geometry for Label {
    fn width(&self) -> f32 {
        self.width
    }
    fn height(&self) -> f32 {
        self.height
    }
}

impl Drawable for Label {
    fn set_color(&mut self, color: u32) {
        for glyph in &mut self.glyphs {
            glyph.set_color(color);
        }
    }
    fn draw(&self, canvas: &mut Canvas, x: f32, y: f32) {
        if self.damaged {
            for glyph in &self.glyphs {
                glyph.draw(canvas, x, y);
            }
        }
    }
}

impl Widget for Label {
    fn damaged(&self) -> bool {
        self.damaged
    }
    fn roundtrip<'d>(&'d mut self, _wx: f32, _wy: f32, dispatch: &Dispatch) -> Option<Damage> {
        if let Dispatch::Commit = dispatch {
            self.damaged = self.damaged == false;
        }
        None
    }
}

impl Label {
    pub fn new(text: &str, font: &Font, font_size: f32, color: u32) -> WidgetShell<Label> {
        let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
        layout.append(&[font], &TextStyle::new(text, font_size, 0));
        let mut width = 0;
        let glyphs;
        glyphs = layout
            .glyphs()
            .iter()
            .map(|gp| {
                if width < gp.width + gp.x as usize {
                    width = gp.width + gp.x as usize
                }
                Glyph::new(&font, gp.key, color, gp.x, gp.y)
            })
            .collect();
        WidgetShell::default(Label {
            damaged: true,
            width: width as f32,
            height: layout.height(),
            glyphs,
        })
    }
}

pub struct TextField {
    color: u32,
    damaged: bool,
    font: Font,
    layout: Layout,
    font_size: f32,
    size: (f32, f32),
    glyphs: Vec<Glyph>,
    setting: LayoutSettings,
}

impl TextField {
    pub fn default(path: &Path, font_size: f32, color: u32) -> TextField {
        let font = read(path).unwrap();
        TextField {
            color,
            font_size,
            damaged: true,
            size: (0., 0.),
            glyphs: Vec::new(),
            setting: DEFAULT,
            font: Font::from_bytes(font, fontdue::FontSettings::default()).unwrap(),
            layout: Layout::new(CoordinateSystem::PositiveYDown),
        }
    }
    pub fn new(text: &str, path: &Path, font_size: f32, color: u32) -> WidgetShell<TextField> {
        let font = read(path).unwrap();
        let font = fontdue::Font::from_bytes(font, fontdue::FontSettings::default()).unwrap();
        let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
        layout.append(&[&font], &TextStyle::new(text, font_size, 0));
        let glyphs;
        let size = (
            {
                let mut w = 0;
                glyphs = layout
                    .glyphs()
                    .iter()
                    .map(|gp| {
                        if w < gp.width + gp.x as usize {
                            w = gp.width + gp.x as usize
                        }
                        Glyph::new(&font, gp.key, color, gp.x, gp.y)
                    })
                    .collect();
                w as f32
            },
            layout.height() as f32,
        );
        WidgetShell::default(TextField {
            font,
            color,
            setting: DEFAULT,
            damaged: true,
            font_size,
            glyphs,
            size,
            layout,
        })
    }
    pub fn from(text: &str, font: &[u8], font_size: f32, color: u32) -> WidgetShell<TextField> {
        let font = fontdue::Font::from_bytes(font, fontdue::FontSettings::default()).unwrap();
        let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
        layout.append(&[&font], &TextStyle::new(text, font_size, 0));
        let glyphs;
        let size = (
            {
                let mut w = 0;
                glyphs = layout
                    .glyphs()
                    .iter()
                    .map(|gp| {
                        if w < gp.width + gp.x as usize {
                            w = gp.width + gp.x as usize
                        }
                        Glyph::new(&font, gp.key, color, gp.x, gp.y)
                    })
                    .collect();
                w as f32
            },
            layout.height() as f32,
        );
        WidgetShell::default(TextField {
            font,
            color,
            setting: DEFAULT,
            damaged: true,
            font_size,
            glyphs,
            size,
            layout,
        })
    }
    pub fn max_width<'f>(
        text: &'f str,
        path: &Path,
        font_size: f32,
        width: f32,
        color: u32,
    ) -> WidgetShell<TextField> {
        let font = read(path).unwrap();
        // Parse it into the font type.
        let mut layout_setting = DEFAULT;
        layout_setting.max_width = Some(width);
        let font = fontdue::Font::from_bytes(font, fontdue::FontSettings::default()).unwrap();
        let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
        layout.reset(&layout_setting);
        layout.append(&[&font], &TextStyle::new(text, font_size, 0));
        let glyphs;
        let size = (
            {
                glyphs = layout
                    .glyphs()
                    .iter()
                    .map(|gp| Glyph::new(&font, gp.key, color, gp.x, gp.y))
                    .collect();
                width
            },
            layout.height() as f32,
        );

        WidgetShell::default(TextField {
            font,
            color,
            layout,
            glyphs,
            font_size,
            damaged: true,
            setting: layout_setting,
            size,
        })
    }
    pub fn max_height<'f>(
        text: &'f str,
        path: &Path,
        font_size: f32,
        height: f32,
        color: u32,
    ) -> WidgetShell<TextField> {
        let font = read(path).unwrap();
        // Parse it into the font type.
        let mut layout_setting = DEFAULT;
        layout_setting.max_height = Some(height);
        let font = fontdue::Font::from_bytes(font, fontdue::FontSettings::default()).unwrap();
        let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
        layout.reset(&layout_setting);
        layout.append(&[&font], &TextStyle::new(text, font_size, 0));
        let glyphs;
        let size = (
            {
                let mut w = 0;
                glyphs = layout
                    .glyphs()
                    .iter()
                    .map(|gp| {
                        if w < gp.width + gp.x as usize {
                            w = gp.width + gp.x as usize
                        }
                        Glyph::new(&font, gp.key, color, gp.x, gp.y)
                    })
                    .collect();
                w as f32
            },
            height,
        );

        WidgetShell::default(TextField {
            font,
            color,
            layout,
            glyphs,
            font_size,
            damaged: true,
            setting: layout_setting,
            size,
        })
    }
    pub fn write(&mut self, text: &str) {
        let mut w = 0;
        let color = self.color;
        let font = &self.font;
        self.layout
            .append(&[font], &TextStyle::new(text, self.font_size as f32, 0));
        self.glyphs = self
            .layout
            .glyphs()
            .iter()
            .map(|gp| {
                if w < gp.width + gp.x as usize {
                    w = gp.width + gp.x as usize
                }
                Glyph::new(&font, gp.key, color, gp.x, gp.y)
            })
            .collect();
        self.size = (w as f32, self.layout.height());
    }
    pub fn edit(&mut self, text: &str) {
        let mut w = 0;
        let color = self.color;
        let font = &self.font;
        self.layout.reset(&self.setting);
        self.layout
            .append(&[font], &TextStyle::new(text, self.font_size as f32, 0));
        self.glyphs = self
            .layout
            .glyphs()
            .iter()
            .map(|gp| {
                if w < gp.width + gp.x as usize {
                    w = gp.width + gp.x as usize
                }
                Glyph::new(&font, gp.key, color, gp.x, gp.y)
            })
            .collect();
        self.size = (w as f32, self.layout.height());
    }
}

impl Geometry for TextField {
    fn width(&self) -> f32 {
        self.size.0
    }
    fn height(&self) -> f32 {
        self.size.1
    }
}

impl Drawable for TextField {
    fn set_color(&mut self, color: u32) {
        self.color = color;
    }
    fn draw(&self, canvas: &mut Canvas, x: f32, y: f32) {
        if self.damaged {
            for glyph in &self.glyphs {
                glyph.draw(canvas, x, y);
            }
        }
    }
}

impl Widget for TextField {
    fn damaged(&self) -> bool {
        self.damaged
    }
    fn roundtrip<'d>(&'d mut self, _wx: f32, _wy: f32, dispatch: &Dispatch) -> Option<Damage> {
        if let Dispatch::Commit = dispatch {
            self.damaged = self.damaged == false;
        }
        None
    }
}
