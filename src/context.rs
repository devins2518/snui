use crate::cache::*;
use crate::*;
use scene::*;
use std::ops::{Deref, DerefMut};
use widgets::label::LabelRef;
use widgets::window::WindowRequest;

pub(crate) mod canvas {
    use crate::scene::*;
    use crate::widgets::shapes::*;
    use crate::*;
    use std::ops::{Deref, DerefMut};

    // Helper to draw using the retained mode API
    pub struct Canvas {
        transform: Transform,
        inner: InnerCanvas,
    }

    #[derive(Clone, PartialEq)]
    pub struct InnerCanvas {
        width: f32,
        height: f32,
        steps: Vec<Instruction>,
    }

    impl InnerCanvas {
        pub fn new(width: f32, height: f32) -> Self {
            Self {
                width,
                height,
                steps: Vec::new(),
            }
        }
    }

    impl Deref for Canvas {
        type Target = InnerCanvas;
        fn deref(&self) -> &Self::Target {
            &self.inner
        }
    }

    impl DerefMut for Canvas {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.inner
        }
    }

    impl Canvas {
        pub fn new(transform: Transform, width: f32, height: f32) -> Self {
            if transform.is_scale_translate() {
                Canvas {
                    transform,
                    inner: InnerCanvas::new(width, height),
                }
            } else {
                panic!("Canvas' transformations can only be scale and translate")
            }
        }
        pub fn draw<P: Into<PrimitiveType>>(&mut self, transform: Transform, p: P) {
            self.inner.steps.push(Instruction {
                transform,
                primitive: p.into(),
            })
        }
        pub fn draw_at<P: Into<PrimitiveType>>(&mut self, x: f32, y: f32, p: P) {
            self.inner.steps.push(Instruction {
                transform: Transform::from_translate(x, y),
                primitive: p.into(),
            })
        }
        pub fn draw_rectangle<B: Into<Texture>>(
            &mut self,
            transform: Transform,
            width: f32,
            height: f32,
            texture: B,
        ) {
            let rect = Rectangle::empty(width, height).background(texture);
            self.inner.steps.push(Instruction {
                transform,
                primitive: rect.into(),
            })
        }
        pub fn draw_at_angle<P: Into<PrimitiveType> + Primitive>(
            &mut self,
            x: f32,
            y: f32,
            p: P,
            angle: f32,
        ) {
            let w = p.width();
            let h = p.height();
            let transform = Transform::from_translate(x, y);
            self.steps.push(Instruction {
                transform: transform.pre_concat(Transform::from_rotate_at(angle, w / 2., h / 2.)),
                primitive: p.into(),
            })
        }
        pub fn finish(mut self) -> RenderNode {
            match self.inner.steps.len() {
                0 => RenderNode::None,
                1 => {
                    let mut i = self.inner.steps.remove(0);
                    i.transform = i.transform.post_concat(self.transform);
                    i.into()
                }
                _ => Instruction::other(self.transform, self.inner).into(),
            }
        }
    }

    impl std::fmt::Debug for InnerCanvas {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Canvas")
                .field("width", &self.width)
                .field("height", &self.height)
                .finish()
        }
    }

    impl Geometry for InnerCanvas {
        fn height(&self) -> f32 {
            self.height
        }
        fn width(&self) -> f32 {
            self.width
        }
    }

    impl Primitive for InnerCanvas {
        fn apply_texture(&self, _: scene::Texture) -> scene::PrimitiveType {
            PrimitiveType::Other(Box::new(self.clone()))
        }
        fn draw_with_transform_clip(
            &self,
            ctx: &mut DrawContext,
            transform: tiny_skia::Transform,
            clip: Option<&tiny_skia::ClipMask>,
        ) {
            for s in &self.steps {
                let t = s.transform.post_concat(transform);
                s.primitive.draw_with_transform_clip(ctx, t, clip);
            }
        }
        fn contains(&self, region: &scene::Region) -> bool {
            Region::new(0., 0., self.width, self.height).rfit(&region)
        }
        fn primitive_type(&self) -> scene::PrimitiveType {
            PrimitiveType::Other(Box::new(self.clone()))
        }
        fn get_texture(&self) -> scene::Texture {
            Texture::Transparent
        }
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

/// Available rendering Backends
pub enum Backend<'b> {
    /// A wrapper around a buffer from TinySkia
    Pixmap(PixmapMut<'b>),
    /// Doesn't do anything. Meant for testing
    Dummy,
}

pub struct SyncContext<'c, D> {
    data: &'c mut D,
    pub(crate) cursor: Option<Cursor>,
    pub(crate) window_request: Option<WindowRequest>,
    pub(crate) cache: &'c mut Cache,
}

pub struct DrawContext<'c> {
    pub path_builder: PathBuilder,
    pub(crate) backend: Backend<'c>,
    pub(crate) cache: &'c mut Cache,
    pub(crate) pending_damage: Vec<Region>,
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

impl<'c, D> SyncContext<'c, D> {
    pub fn new(data: &'c mut D, cache: &'c mut Cache) -> Self {
        Self {
            data,
            cache,
            cursor: None,
            window_request: None,
        }
    }
    /// Close the application window
    pub fn close(&mut self) {
        self.window_request = Some(WindowRequest::Close);
    }
    pub fn minimize(&mut self) {
        self.window_request = Some(WindowRequest::Minimize);
    }
    pub fn maximize(&mut self) {
        self.window_request = Some(WindowRequest::Maximize);
    }
    /// Launch a system menu
    pub fn menu(&mut self, x: f32, y: f32, serial: u32) {
        self.window_request = Some(WindowRequest::Menu(x, y, serial));
    }
    /// Move the application window
    pub fn drag(&mut self, serial: u32) {
        self.window_request = Some(WindowRequest::Move(serial));
    }
    pub fn set_title<S: ToString>(&mut self, title: S) {
        self.window_request = Some(WindowRequest::Title(title.to_string()));
    }
    pub fn set_cursor(&mut self, cursor: Cursor) {
        self.cursor = Some(cursor);
    }
}

impl<'c, D> Deref for SyncContext<'c, D> {
    type Target = D;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<'c, D> DerefMut for SyncContext<'c, D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<'c, D> AsMut<Cache> for SyncContext<'c, D> {
    fn as_mut(&mut self) -> &mut Cache {
        self.cache
    }
}

impl<'c> DrawContext<'c> {
    pub fn new(backend: Backend<'c>, cache: &'c mut Cache) -> Self {
        Self {
            cache,
            backend,
            pending_damage: Vec::new(),
            path_builder: PathBuilder::new(),
        }
    }
    /// Replaces the inner PathBuilder.
    /// Must be reset.
    pub fn path_builder(&mut self) -> PathBuilder {
        std::mem::replace(&mut self.path_builder, PathBuilder::with_capacity(0, 0))
    }
    pub fn reset(&mut self, path_builder: PathBuilder) {
        self.path_builder = path_builder;
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
    /// Damages a region of the buffer in preparation of a draw.
    pub fn damage_region(&mut self, texture: &Texture, region: Region, composite: bool) {
        if !composite {
            if let Some(last) = self.pending_damage.last() {
                if last.contains(region.x, region.y) {
                    for region in last.merge(&region).substract(*last) {
                        if !region.null() {
                            self.damage_region(texture, region, composite);
                        }
                    }
                    return;
                }
            }
            self.pending_damage.push(region);
        }
        match texture {
            Texture::Color(color) => match &mut self.backend {
                Backend::Pixmap(dt) => {
                    dt.fill_rect(
                        region.into(),
                        &Paint {
                            shader: Shader::SolidColor(*color),
                            blend_mode: BlendMode::SourceOver,
                            anti_alias: false,
                            force_hq_pipeline: false,
                        },
                        Transform::identity(),
                        None,
                    );
                }
                _ => {}
            },
            Texture::LinearGradient {
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
                                blend_mode: BlendMode::SourceOver,
                                anti_alias: false,
                                force_hq_pipeline: false,
                            },
                            Transform::identity(),
                            None,
                        );
                    }
                }
            }
            Texture::Image(coords, image) => {
                let crop =
                    Region::new(coords.x, coords.y, image.width(), image.height()).crop(&region);
                let (sx, sy) = image.scale();
                let source = image.pixmap();
                if let Backend::Pixmap(dt) = &mut self.backend {
                    dt.fill_rect(
                        crop.into(),
                        &Paint {
                            shader: Pattern::new(
                                source,
                                SpreadMode::Pad,
                                FilterQuality::Bilinear,
                                1.0,
                                Transform::from_scale(sx, sy).post_translate(coords.x, coords.y),
                            ),
                            anti_alias: false,
                            force_hq_pipeline: false,
                            blend_mode: BlendMode::SourceOver,
                        },
                        Transform::identity(),
                        None,
                    );
                }
            }
            Texture::Composite(layers) => {
                for layer in layers {
                    self.damage_region(layer, region, true);
                }
            }
            Texture::Transparent => match &mut self.backend {
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
                    );
                }
                _ => {}
            },
        }
    }
    pub fn damage_queue(&self) -> &[Region] {
        self.pending_damage.as_slice()
    }
    /// Empties the text layout to render it.
    pub fn finish(&mut self, x: f32, y: f32, fonts: &[FontProperty]) {
        let font_cache = &mut self.cache.font_cache;
        let layout = font_cache.layout.glyphs();
        for gp in layout {
            if let Some(glyph_cache) = font_cache
                .fonts
                .get_mut(&fonts[gp.font_index])
            {
                if let Some(pixmap) = glyph_cache.render_glyph(gp) {
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
    pub fn draw_label(&mut self, x: f32, y: f32, label: LabelRef) {
        let font_cache = &mut self.cache.font_cache;
        for gp in {
            font_cache.layout(label);
            font_cache.layout.glyphs()
        } {
            if let Some(glyph_cache) = font_cache
                .fonts
                .get_mut(&label.fonts[gp.font_index])
            {
                if let Some(pixmap) = glyph_cache.render_glyph(gp) {
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
