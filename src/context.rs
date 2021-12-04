use crate::font::FontCache;
use crate::*;
use data::*;
use scene::*;
use std::ops::{Deref, DerefMut};
use tiny_skia::*;
use widgets::text::Label;

pub const PIX_PAINT: PixmapPaint = PixmapPaint {
    blend_mode: BlendMode::SourceOver,
    opacity: 1.0,
    quality: FilterQuality::Nearest,
};

pub const TEXT: PixmapPaint = PixmapPaint {
    blend_mode: BlendMode::SourceAtop,
    opacity: 1.0,
    quality: FilterQuality::Bilinear,
};

pub enum Backend<'b> {
    Pixmap(PixmapMut<'b>),
    Dummy,
}

pub struct SyncContext<'c> {
    draw: bool,
    model: &'c mut dyn Controller,
    pub font_cache: &'c mut FontCache,
}

pub struct DrawContext<'c> {
    backend: Backend<'c>,
    font_cache: &'c mut FontCache,
    pending_damage: &'c mut Vec<Region>,
}

impl<'b> Geometry for Backend<'b> {
    fn width(&self) -> f32 {
        match self {
            Backend::Dummy => 0.,
            Backend::Pixmap(dt) => dt.width() as f32,
        }
    }
    fn height(&self) -> f32 {
        match self {
            Backend::Dummy => 0.,
            Backend::Pixmap(dt) => dt.height() as f32,
        }
    }
    fn set_width(&mut self, _width: f32) -> Result<(), f32> {
        Err(self.width())
    }
    fn set_height(&mut self, _height: f32) -> Result<(), f32> {
        Err(self.height())
    }
}

impl<'b> Deref for Backend<'b> {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        match self {
            Backend::Pixmap(dt) => dt.as_ref().data(),
            _ => panic!("Dummy backend cannot return a slice"),
        }
    }
}

impl<'c> DerefMut for Backend<'c> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Backend::Pixmap(dt) => dt.data_mut(),
            _ => panic!("Dummy backend cannot return a slice"),
        }
    }
}

impl<'c> SyncContext<'c> {
    pub fn sync(&mut self) {}
    pub fn request_draw(&mut self) {
        self.draw = true;
    }
    pub fn damage(self) -> bool {
        self.draw
    }
    pub fn new(model: &'c mut impl Controller, font_cache: &'c mut FontCache) -> Self {
        Self {
            draw: false,
            model,
            font_cache,
        }
    }
}

impl<'c> Controller for SyncContext<'c> {
    fn deserialize(&mut self, token: u32) -> Result<(), ControllerError> {
        self.model.deserialize(token)
    }
    fn get<'m>(&'m self, msg: Message) -> Result<Data<'m>, ControllerError> {
        self.model.get(msg)
    }
    fn serialize(&mut self, msg: Message) -> Result<u32, ControllerError> {
        self.model.serialize(msg)
    }
    fn send<'m>(&'m mut self, msg: Message) -> Result<Data<'m>, ControllerError> {
        self.model.send(msg)
    }
    fn sync(&self) -> Result<(), ControllerError> {
        self.model.sync()
    }
}

impl<'c> DrawContext<'c> {
    pub fn new(
        backend: Backend<'c>,
        font_cache: &'c mut FontCache,
        pending_damage: &'c mut Vec<Region>,
    ) -> Self {
        Self {
            backend,
            font_cache,
            pending_damage,
        }
    }
    pub fn damage_region(&mut self, bg: &Background, region: &Region) {
        match bg {
            Background::Color(color) => match &mut self.backend {
                Backend::Pixmap(dt) => dt
                    .fill_rect(
                        region.into(),
                        &Paint {
                            shader: Shader::SolidColor(*color),
                            blend_mode: BlendMode::SourceAtop,
                            anti_alias: false,
                            force_hq_pipeline: false,
                        },
                        Transform::identity(),
                        None,
                    )
                    .unwrap(),
                _ => {}
            },
            Background::LinearGradient {
                start,
                end,
                angle: _,
                stops,
                mode,
            } => {
                if let Backend::Pixmap(dt) = &mut self.backend {
                    if let Some(grad) = LinearGradient::new(
                        start.into(),
                        end.into(),
                        stops.as_ref().to_vec(),
                        *mode,
                        Transform::identity(),
                    ) {
                        dt.fill_rect(
                            region.into(),
                            &Paint {
                                shader: grad,
                                blend_mode: BlendMode::SourceAtop,
                                anti_alias: false,
                                force_hq_pipeline: false,
                            },
                            Transform::identity(),
                            None,
                        );
                    }
                }
            }
            Background::Image(coords, image) => {
                let crop =
                    Region::new(coords.x, coords.y, image.width(), image.height()).crop(region);
                let (sx, sy) = image.scale();
                let source = image.pixmap();
                if let Backend::Pixmap(dt) = &mut self.backend {
                    let mut clip = ClipMask::new();
                    clip.set_path(
                        image.width() as u32,
                        image.height() as u32,
                        &PathBuilder::from_rect((&crop).into()),
                        FillRule::Winding,
                        false,
                    );
                    dt.draw_pixmap(
                        coords.x as i32,
                        coords.y as i32,
                        source,
                        &PIX_PAINT,
                        Transform::from_scale(sx, sy),
                        Some(&clip),
                    );
                }
            }
            Background::Composite(base, overlay) => {
                self.damage_region(base.as_ref(), &region);
                self.damage_region(overlay.as_ref(), &region);
            }
            _ => {}
        }
        self.pending_damage.push(*region);
    }
    pub fn flush(&mut self) {
        self.pending_damage.clear();
    }
    pub fn draw_label(&mut self, label: &Label, x: f32, y: f32) {
        if let Some(layout) = label.get_layout() {
            for gp in layout.as_ref() {
                if let Some(glyph_cache) = self
                    .font_cache
                    .fonts
                    .get_mut(&label.fonts()[gp.key.font_index as usize])
                {
                    if let Some(pixmap) = glyph_cache.render_glyph(gp, label.get_color()) {
                        if let Some(pixmap) = PixmapRef::from_bytes(
                            unsafe {
                                std::slice::from_raw_parts(
                                    pixmap.as_ptr() as *mut u8,
                                    pixmap.len() * std::mem::size_of::<u32>(),
                                )
                            },
                            gp.width as u32,
                            gp.height as u32,
                        ) {
                            match &mut self.backend {
                                Backend::Pixmap(dt) => {
                                    dt.draw_pixmap(
                                        (x.round() + gp.x) as i32,
                                        (y.round() + gp.y) as i32,
                                        pixmap,
                                        &TEXT,
                                        Transform::from_scale(1., 1.),
                                        None,
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
}

impl<'c> Deref for DrawContext<'c> {
    type Target = Backend<'c>;
    fn deref(&self) -> &Self::Target {
        &self.backend
    }
}

impl<'c> DerefMut for DrawContext<'c> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.backend
    }
}
