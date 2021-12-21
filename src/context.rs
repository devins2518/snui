use crate::font::FontCache;
use crate::*;
use data::*;
use scene::*;
use std::ops::{Deref, DerefMut};
use tiny_skia::*;
use widgets::text::Label;

pub(crate) mod canvas {
    use crate::scene::*;
    use crate::widgets::shapes::*;
    use crate::*;

    // Helper to draw using the retained mode API
    pub struct Canvas {
        coords: Coords,
        region: Region,
        steps: Vec<Instruction>,
    }

    impl Canvas {
        pub fn new(region: Region) -> Self {
            Canvas {
                coords: Coords::new(region.x, region.y),
                steps: Vec::new(),
                region,
            }
        }
        pub fn draw<P: Into<PrimitiveType>>(&mut self, x: f32, y: f32, p: P) {
            let x = x + self.coords.x;
            let y = y + self.coords.y;
            self.steps.push(Instruction::new(x, y, p.into()))
        }
        pub fn draw_rectangle<B: Into<Background>>(
            &mut self,
            x: f32,
            y: f32,
            width: f32,
            height: f32,
            bg: B,
        ) {
            let x = x + self.coords.x;
            let y = y + self.coords.y;
            let rect = Rectangle::empty(width, height).background(bg);
            self.steps.push(Instruction::new(x, y, rect))
        }
        pub fn draw_at_angle<P: Into<PrimitiveType> + Primitive>(
            &mut self,
            x: f32,
            y: f32,
            p: P,
            angle: f32,
        ) {
            let x = x + self.coords.x;
            let y = y + self.coords.y;
            let w = p.width();
            let h = p.height();
            self.steps.push(
                Instruction::new(x, y, p.into()).transform(Transform::from_rotate_at(
                    angle,
                    x + w / 2.,
                    y + h / 2.,
                )),
            )
        }
        pub fn finish(self) -> RenderNode {
            RenderNode::Draw {
                region: self.region,
                steps: self.steps,
            }
        }
        // pub fn draw_oval(&mut self, x: f32, y: f32, width: f32, height: f32) {
        // }
    }
}

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

// Available rendering Backends
pub enum Backend<'b> {
    Pixmap(PixmapMut<'b>),
    Dummy,
}

pub struct SyncContext<'c> {
    controller: &'c mut dyn Controller,
    pub font_cache: &'c mut FontCache,
}

pub struct DrawContext<'c> {
    pub(crate) backend: Backend<'c>,
    pub(crate) font_cache: &'c mut FontCache,
    pub(crate) pending_damage: &'c mut Vec<Region>,
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
    pub fn new(controller: &'c mut impl Controller, font_cache: &'c mut FontCache) -> Self {
        Self {
            controller,
            font_cache,
        }
    }
}

impl<'c> Controller for SyncContext<'c> {
    fn deserialize(&mut self, token: u32) -> Result<(), ControllerError> {
        self.controller.deserialize(token)
    }
    fn get<'m>(&'m self, msg: Message) -> Result<Data<'m>, ControllerError> {
        self.controller.get(msg)
    }
    fn serialize(&mut self, msg: Message) -> Result<u32, ControllerError> {
        self.controller.serialize(msg)
    }
    fn send<'m>(&'m mut self, msg: Message) -> Result<Data<'m>, ControllerError> {
        self.controller.send(msg)
    }
    fn sync(&mut self) -> Result<Message<'static>, ControllerError> {
        self.controller.sync()
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
    pub fn commit(&mut self, region: Region) {
        if let Some(r) = self.pending_damage.last_mut() {
            if region.intersect(r) {
                let merge = r.merge(&region);
                *r = merge;
            } else {
                self.pending_damage.push(region);
            }
        } else {
            self.pending_damage.push(region);
        }
    }
    pub fn damage_region(&mut self, bg: &Background, mut region: Region, composite: bool) {
        if !composite {
            if let Some(last) = self.pending_damage.last() {
                if last.contains(region.x, region.y) {
                    let taken = last.merge(&region).substract(*last);
                    region = taken;
                }
            }
            self.pending_damage.push(region);
        }
        match bg {
            Background::Color(color) => match &mut self.backend {
                Backend::Pixmap(dt) => {
                    dt.fill_rect(
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
                    .unwrap();
                }
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
                    Region::new(coords.x, coords.y, image.width(), image.height()).crop(&region);
                let (sx, sy) = image.scale();
                let source = image.pixmap();
                if let Backend::Pixmap(dt) = &mut self.backend {
                    let mut clip = ClipMask::new();
                    clip.set_path(
                        dt.width(),
                        dt.height(),
                        &PathBuilder::from_rect((&crop).into()),
                        FillRule::Winding,
                        false,
                    );
                    dt.draw_pixmap(
                        0,
                        0,
                        source,
                        &PIX_PAINT,
                        Transform::from_scale(sx, sy).post_translate(coords.x, coords.y),
                        Some(&clip),
                    )
                    .unwrap();
                }
            }
            Background::Composite(layers) => {
                for layer in layers {
                    self.damage_region(layer, region, true);
                }
            }
            Background::Transparent => match &mut self.backend {
                Backend::Pixmap(dt) => {
                    dt.fill_rect(
                        region.into(),
                        &Paint {
                            shader: Shader::SolidColor(Color::TRANSPARENT),
                            blend_mode: BlendMode::Clear,
                            anti_alias: false,
                            force_hq_pipeline: false,
                        },
                        Transform::identity(),
                        None,
                    )
                    .unwrap();
                }
                _ => {}
            },
        }
    }
    pub fn flush(&mut self) {
        self.pending_damage.clear();
    }
    pub fn draw_label(&mut self, label: &Label, x: f32, y: f32) {
        if let Some(layout) = label.get_layout() {
            for gp in layout.as_ref() {
                if let Some(glyph_cache) =
                    self.font_cache.fonts.get_mut(&label.fonts()[gp.font_index])
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
                                        Transform::identity(),
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
