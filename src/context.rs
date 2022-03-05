//! Contexts provided to snui widgets.

use crate::*;
use crate::{cache::*, widgets::Alignment};
use scene::*;
use std::ops::{Deref, DerefMut};

const WINDOW_STATE: [WindowState; 0] = [];

/// Available rendering Backends
pub enum Backend<'b> {
    /// A wrapper around a buffer from TinySkia
    Pixmap {
        pixmap: PixmapMut<'b>,
        clipmask: Option<&'b mut ClipMask>,
    },
    /// Doesn't do anything. Meant for testing
    Dummy,
}

/// A context provided to the sync methods of widgets.
///
/// The context dereferences the Data so widgets can interact with it.
/// It also contains the cache which is used for text layouting and fetching images.
pub struct UpdateContext<'a, 'b, T> {
    updated: bool,
    data: &'b mut T,
    pub(crate) cache: &'a mut Cache,
    pub(crate) window: &'a mut dyn WindowHandle<T>,
}

/// A context provided to primitives during draw.
///
/// It does all the text rendering along and gives access to the backend which primitive can use to draw.
pub struct DrawContext<'c> {
    transform: Transform,
    path_builder: Option<PathBuilder>,
    pub(crate) backend: Backend<'c>,
    pub(crate) cache: &'c mut Cache,
    pub(crate) pending_damage: Vec<Region>,
}

/// A context provided during layout.
///
/// Currently it only holds the Cache.
pub struct LayoutCtx<'c> {
    /// Forces Proxies to propagate the layout
    pub(crate) force: bool,
    pub(crate) cache: &'c mut Cache,
}

impl<'c> AsMut<Cache> for LayoutCtx<'c> {
    fn as_mut(&mut self) -> &mut Cache {
        self.cache
    }
}

impl<'c> LayoutCtx<'c> {
    pub fn new(cache: &'c mut Cache) -> LayoutCtx {
        LayoutCtx {
            force: false,
            cache,
        }
    }
    /// Creates a new instance of the LayoutCtx
    /// with the force field enabled.
    pub fn force<'s, 'n>(&'s mut self) -> LayoutCtx<'n>
    where
        's: 'n,
    {
        LayoutCtx {
            force: true,
            cache: self.cache,
        }
    }
}

pub enum Menu<T> {
    System {
        position: Coords,
        serial: u32,
    },
    Popup {
        data: T,
        size: Size,
        offset: Coords,
        anchor: (Alignment, Alignment),
        widget: Box<dyn Widget<T>>,
    },
}

/// A handle to the window state.
pub trait WindowHandle<T> {
    /// Closes the window.
    ///
    /// This will terminate the application.
    fn close(&mut self) {}
    fn minimize(&mut self) {}
    fn maximize(&mut self) {}
    /// Show a context menu
    fn show_menu(&mut self, _menu: Menu<T>) {}
    /// Move the window.
    ///
    /// The serial is provided by Event.Pointer.
    fn _move(&mut self, _serial: u32) {}
    fn set_title(&mut self, _title: String) {}
    fn set_cursor(&mut self, _cursor: Cursor) {}
    /// Retreive the state of the window.
    fn get_state(&self) -> &[WindowState] {
        &WINDOW_STATE
    }
}

impl<T> WindowHandle<T> for () {}

impl<'b> Geometry for Backend<'b> {
    fn width(&self) -> f32 {
        match self {
            Backend::Dummy => 0.,
            Backend::Pixmap { pixmap, .. } => pixmap.width() as f32,
        }
    }
    fn height(&self) -> f32 {
        match self {
            Backend::Dummy => 0.,
            Backend::Pixmap { pixmap, .. } => pixmap.height() as f32,
        }
    }
}

impl<'b> Deref for Backend<'b> {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        match self {
            Backend::Pixmap { pixmap, .. } => pixmap.as_ref().data(),
            _ => panic!("Dummy backend cannot return a slice"),
        }
    }
}

impl<'c> DerefMut for Backend<'c> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Backend::Pixmap { pixmap, .. } => pixmap.data_mut(),
            _ => panic!("Dummy backend cannot return a slice"),
        }
    }
}

impl<'a, 'b, T> UpdateContext<'a, 'b, T> {
    pub fn new(data: &'b mut T, cache: &'a mut Cache, window: &'a mut dyn WindowHandle<T>) -> Self {
        Self {
            data,
            cache,
            window,
            updated: false,
        }
    }
    /// Return a reference to the WindowHandle
    pub fn window(&mut self) -> &mut dyn WindowHandle<T> {
        &mut *self.window
    }
    /// Creates a custom popup passed to the WindowHandle
    pub fn create_popup<F>(&mut self, mut builder: F)
    where
        F: FnMut(&T, LayoutCtx) -> Menu<T>,
    {
        let layout = LayoutCtx::new(self.cache);
        let menu = builder(self.data, layout);
        self.window.show_menu(menu);
    }
    /// Creates a new instance of an UpdateContext with different data
    pub fn fork<'c, 'd>(&'c mut self, data: &'d mut T) -> UpdateContext<'c, 'd, T> {
        UpdateContext::new(data, self.cache, self.window)
    }
    /// Puts the context in an updated state
    pub fn update(&mut self) {
        self.updated = true;
    }
    pub fn updated(&self) -> bool {
        self.updated
    }
}

impl<'a, 'b, T> Deref for UpdateContext<'a, 'b, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl<'a, 'b, T> DerefMut for UpdateContext<'a, 'b, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.updated = true;
        self.data
    }
}

impl<'a, 'b, T> AsMut<Cache> for UpdateContext<'a, 'b, T> {
    fn as_mut(&mut self) -> &mut Cache {
        self.cache
    }
}

use tiny_skia::Transform;

impl<'c> DrawContext<'c> {
    pub fn new(backend: Backend<'c>, cache: &'c mut Cache) -> Self {
        Self {
            cache,
            backend,
            pending_damage: Vec::new(),
            transform: Transform::identity(),
            path_builder: Some(PathBuilder::new()),
        }
    }
    pub fn with_clipmask(mut self, t_clipmask: &'c mut ClipMask) -> Self {
        match &mut self.backend {
            Backend::Pixmap { clipmask, .. } => {
                *clipmask = Some(t_clipmask);
            }
            _ => panic!("Cannot attach clipmask to Dummy backend"),
        }
        self
    }
    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }
    pub fn draw<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Self, PathBuilder) -> Path,
    {
        let pb = self.path_builder.take().unwrap();
        self.path_builder = Some(f(self, pb).clear());
    }
    pub(crate) fn transform(&self) -> Transform {
        self.transform
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
    pub(crate) fn set_clip(&mut self, region: &Region) {
        let width = self.width();
        let height = self.height();
        match &mut self.backend {
            Backend::Pixmap { clipmask, .. } => {
                if let Some(clipmask) = clipmask {
                    let mut pb = self.path_builder.take().unwrap();
                    pb.push_rect(region.x, region.y, region.width, region.height);
                    let path = pb
                        .finish()
                        .and_then(|path| path.transform(self.transform))
                        .unwrap();
                    clipmask.set_path(width as u32, height as u32, &path, FillRule::Winding, false);
                    self.path_builder = Some(path.clear());
                }
            }
            _ => {}
        }
    }
    /// This method is usually called when you want to clean up an area to draw on it.
    pub(crate) fn clear(&mut self, background: &Background, region: Region) {
        let blend;
        if let Some(background) = background.previous {
            self.clear(background, region);
            // Could perhaps be Source
            blend = BlendMode::SourceOver;
        } else {
            if let Some(last) = self.pending_damage.last() {
                if last.contains(region.x, region.y) {
                    for region in last.merge(&region).substract(*last) {
                        self.clear(background, region);
                    }
                }
            }
            blend = BlendMode::Source;
            self.commit(region);
        }
        match &mut self.backend {
            Backend::Pixmap { pixmap, clipmask } => {
                let clip_mask = clipmask
                    .as_ref()
                    .and_then(|clipmask| (!clipmask.is_empty()).then(|| &**clipmask));
                match background.texture() {
                    Texture::Color(color) => {
                        pixmap.fill_rect(
                            region.into(),
                            &Paint {
                                shader: Shader::SolidColor(*color),
                                blend_mode: blend,
                                anti_alias: false,
                                force_hq_pipeline: false,
                            },
                            self.transform,
                            clip_mask,
                        );
                    }
                    Texture::LinearGradient(gradient) => {
                        let background_region = background.region();
                        let start = match gradient.orientation {
                            Orientation::Horizontal => background_region.start(),
                            Orientation::Vertical => background_region.top_anchor(),
                        };
                        let end = match gradient.orientation {
                            Orientation::Horizontal => background_region.end(),
                            Orientation::Vertical => background_region.bottom_anchor(),
                        };
                        pixmap.fill_rect(
                            region.into(),
                            &Paint {
                                shader: LinearGradient::new(
                                    start.into(),
                                    end.into(),
                                    gradient.stops.clone(),
                                    gradient.mode,
                                    self.transform,
                                )
                                .expect("Failed to build LinearGradient shader"),
                                blend_mode: blend,
                                anti_alias: false,
                                force_hq_pipeline: false,
                            },
                            self.transform,
                            clip_mask,
                        );
                    }
                    Texture::Image(image) => {
                        let sx = background.rectangle.width / image.width();
                        let sy = background.rectangle.height / image.height();
                        pixmap.fill_rect(
                            region.into(),
                            &Paint {
                                shader: Pattern::new(
                                    image.pixmap(),
                                    SpreadMode::Repeat,
                                    FilterQuality::Bilinear,
                                    1.0,
                                    self.transform
                                        .post_translate(
                                            background.position.x,
                                            background.position.y,
                                        )
                                        .post_scale(sx, sy),
                                ),
                                blend_mode: blend,
                                anti_alias: false,
                                force_hq_pipeline: false,
                            },
                            self.transform,
                            clip_mask,
                        );
                    }
                    Texture::Transparent => {
                        pixmap.fill_rect(
                            region.into(),
                            &Paint {
                                shader: Shader::SolidColor(Color::TRANSPARENT),
                                blend_mode: BlendMode::Clear,
                                anti_alias: false,
                                force_hq_pipeline: false,
                            },
                            self.transform,
                            clip_mask,
                        );
                    }
                }
            }
            _ => {}
        }
    }
    pub fn damage_queue(&self) -> &[Region] {
        self.pending_damage.as_slice()
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
