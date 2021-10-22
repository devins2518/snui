use crate::*;
pub use fontdue::{
    layout,
    Font,
    FontSettings,
    FontResult,
    layout::{CoordinateSystem, GlyphRasterConfig, Layout, LayoutSettings, TextStyle, GlyphPosition},
};
use raqote::*;
use std::clone::Clone;
use std::fs::read;
use std::path::Path;
use std::collections::HashMap;
use crate::widgets::primitives::WidgetShell;

pub fn font_from_path(path: &Path) -> Font {
    let font = read(path).unwrap();
    Font::from_bytes(font, fontdue::FontSettings::default()).unwrap()
}

pub struct GlyphCache {
    pub font: Font,
    glyphs: HashMap<GlyphRasterConfig, Vec<u8>>
}

impl GlyphCache {
    pub fn new(font: Font) -> Self {
        Self {
            font,
            glyphs: HashMap::new()
        }
    }
    pub fn load(path: &Path) -> FontResult<Self> {
        if let Ok(bytes) = read(path) {
            if let Ok(font) = Font::from_bytes(bytes, fontdue::FontSettings::default()) {
                Ok(Self {
                    font,
                    glyphs: HashMap::new()
                })
            } else {
                FontResult::Err("Isn't a font")
            }
        } else {
            FontResult::Err("Invalid path")
        }
    }
    pub fn render_glyph(&mut self, glyph: &GlyphPosition, source: SolidSource) -> Option<Vec<u32>> {
        if !glyph.char_data.is_missing() {
            let pixmap: Vec<u32>;
            if let Some(coverage) = self.glyphs.get(&glyph.key) {
                pixmap = coverage
                    .iter()
                    .map(|a| {
                        if a == &0 {
                            0
                        } else {
                            SolidSource::from_unpremultiplied_argb(*a, source.r, source.g, source.b)
                                .to_u32()
                        }
                    })
                    .collect();
            } else {
                let (_, coverage) = self.font.rasterize_config(glyph.key);
                pixmap = coverage
                    .iter()
                    .map(|a| {
                        if a == &0 {
                            0
                        } else {
                            SolidSource::from_unpremultiplied_argb(*a, source.r, source.g, source.b)
                                .to_u32()
                        }
                    })
                    .collect();
                self.glyphs.insert(glyph.key, coverage);
            }
            return Some(pixmap)
        }
        None
    }
}

pub struct Label {
    width: f32,
    damaged: bool,
    layout: Layout,
    font_size: f32,
    source: SolidSource,
    font: String,
    write_buffer: Option<String>,
    settings: LayoutSettings,
    glyphs: Vec<GlyphPosition>,
}

impl Geometry for Label {
    fn width(&self) -> f32 {
        self.width
    }
    fn height(&self) -> f32 {
        self.layout.height()
    }
}

impl Clone for Label {
    fn clone(&self) -> Self {
        let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
        layout.reset(&self.settings);
        Label {
            width: self.width,
            damaged: true,
            layout,
            font_size: self.font_size,
            source: self.source,
            font: self.font.clone(),
            glyphs: self.glyphs.clone(),
            write_buffer: self.write_buffer.clone(),
            settings: self.settings,
        }
    }
}

impl Drawable for Label {
    fn set_color(&mut self, color: u32) {
        let color = color.to_be_bytes();
        self.source = SolidSource {
            a: color[0],
            r: color[1],
            g: color[2],
            b: color[3],
        };
    }
    fn draw(&self, canvas: &mut Canvas, x: f32, y: f32) {
        if self.damaged {
            canvas.draw_label(x, y, &self.font, &self.glyphs, self.source);
        }
    }
}

impl Widget for Label {
    fn damaged(&self) -> bool {
        self.damaged
    }
    fn roundtrip<'d>(&'d mut self, _wx: f32, _wy: f32, canvas: &mut Canvas, dispatch: &Dispatch) -> Option<Damage> {
        match dispatch {
            Dispatch::Commit => self.damaged = self.damaged == false,
            Dispatch::Prepare => if let Some(font) = canvas.get_font(&self.font) {
                if let Some(text) = self.write_buffer.as_ref() {
                    self.layout.append(&[font], &TextStyle::new(text, self.font_size, 0));
                }
            }
            _ => {}
        }
        None
    }
}

fn create_layout(max_width: Option<f32>, max_height: Option<f32>) -> (LayoutSettings, Layout) {
    let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
    let setting = LayoutSettings {
        x: 0.,
        y: 0.,
        max_width,
        max_height,
        horizontal_align: layout::HorizontalAlign::Left,
        vertical_align: layout::VerticalAlign::Middle,
        wrap_style: layout::WrapStyle::Word,
        wrap_hard_breaks: true
    };
    layout.reset(&setting);
    (setting, layout)
}

fn get_width<U: Copy + Clone>(glyphs: &Vec<GlyphPosition<U>>) -> f32 {
    let mut width = 0;
    for gp in glyphs {
        if width < gp.width + gp.x as usize {
            width = gp.width + gp.x as usize
        }
    }
    width as f32
}

fn create_source(color: u32) -> SolidSource {
    let color = color.to_be_bytes();
    SolidSource {
        a: color[0],
        r: color[1],
        g: color[2],
        b: color[3],
    }
}

impl Label {
    pub fn new(text: &str, font: &str, font_size: f32, color: u32) -> WidgetShell<Label> {
        let (settings, mut layout) = create_layout(None, None);
        WidgetShell::default(Label {
            damaged: true,
            glyphs: layout.glyphs().clone(),
            width: get_width(layout.glyphs()),
            source: create_source(color),
            font: font.to_owned(),
            font_size,
            settings,
            write_buffer: Some(text.to_owned()),
            layout
        })
    }
    pub fn max_width(text: &str, font: &str, font_size: f32, width: f32, color: u32) -> WidgetShell<Label> {
        let (settings, mut layout) = create_layout(Some(width), None);
        WidgetShell::default(Label {
            damaged: true,
            glyphs: layout.glyphs().clone(),
            width: get_width(layout.glyphs()),
            source: create_source(color),
            font: font.to_owned(),
            font_size,
            settings,
            write_buffer: Some(text.to_owned()),
            layout
        })
    }
    pub fn write(&mut self, text: &str) {
        self.write_buffer = Some(text.to_owned());
    }
    pub fn edit(&mut self, text: &str) {
        self.write_buffer = Some(text.to_owned());
        self.layout.clear();
        self.layout.reset(&self.settings);
    }
}
