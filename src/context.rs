use crate::cache::*;
use crate::*;
use scene::*;
use std::ops::{Deref, DerefMut};
use widgets::label::LabelRef;

// pub mod canvas {
//     use crate::scene::*;
//     use crate::widgets::shapes::*;
//     use crate::*;
//     use std::ops::{Deref, DerefMut};
//
//     // Helper to draw using the retained mode API
//     pub struct Canvas {
//         transform: Transform,
//         inner: InnerCanvas,
//     }
//
//     #[derive(Clone, PartialEq)]
//     pub struct InnerCanvas {
//         width: f32,
//         height: f32,
//         steps: Vec<Instruction>,
//     }
//
//     impl InnerCanvas {
//         pub fn new(width: f32, height: f32) -> Self {
//             Self {
//                 width,
//                 height,
//                 steps: Vec::new(),
//             }
//         }
//     }
//
//     impl Deref for Canvas {
//         type Target = InnerCanvas;
//         fn deref(&self) -> &Self::Target {
//             &self.inner
//         }
//     }
//
//     impl DerefMut for Canvas {
//         fn deref_mut(&mut self) -> &mut Self::Target {
//             &mut self.inner
//         }
//     }
//
//     impl Canvas {
//         pub fn new(transform: Transform, width: f32, height: f32) -> Self {
//             if transform.is_scale_translate() {
//                 Canvas {
//                     transform,
//                     inner: InnerCanvas::new(width, height),
//                 }
//             } else {
//                 panic!("Canvas' transformations can only be scale and translate")
//             }
//         }
//         pub fn draw<P: Into<Primitive>>(&mut self, transform: Transform, p: P) {
//             self.inner.steps.push(Instruction {
//                 transform,
//                 primitive: p.into(),
//             })
//         }
//         pub fn draw_at<P: Into<Primitive>>(&mut self, x: f32, y: f32, p: P) {
//             self.inner.steps.push(Instruction {
//                 transform: Transform::from_translate(x, y),
//                 primitive: p.into(),
//             })
//         }
//         pub fn draw_rectangle<B: Into<Texture>>(
//             &mut self,
//             transform: Transform,
//             width: f32,
//             height: f32,
//             texture: B,
//         ) {
//             let rect = Rectangle::new(width, height).background(texture);
//             self.inner.steps.push(Instruction {
//                 transform,
//                 primitive: rect.into(),
//             })
//         }
//         pub fn draw_at_angle<P: Into<Primitive> + Drawable>(
//             &mut self,
//             x: f32,
//             y: f32,
//             p: P,
//             angle: f32,
//         ) {
//             let w = p.width();
//             let h = p.height();
//             let transform = Transform::from_translate(x, y);
//             self.steps.push(Instruction {
//                 transform: transform.pre_concat(Transform::from_rotate_at(angle, w / 2., h / 2.)),
//                 primitive: p.into(),
//             })
//         }
//         pub fn finish(mut self) -> RenderNode {
//             match self.inner.steps.len() {
//                 0 => RenderNode::None,
//                 1 => {
//                     let mut i = self.inner.steps.remove(0);
//                     i.transform = i.transform.post_concat(self.transform);
//                     i.into()
//                 }
//                 _ => Instruction::other(self.transform, self.inner).into(),
//             }
//         }
//     }
//
//     impl std::fmt::Debug for InnerCanvas {
//         fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//             f.debug_struct("Canvas")
//                 .field("width", &self.width)
//                 .field("height", &self.height)
//                 .finish()
//         }
//     }
//
//     impl Geometry for InnerCanvas {
//         fn height(&self) -> f32 {
//             self.height
//         }
//         fn width(&self) -> f32 {
//             self.width
//         }
//     }
//
//     impl Drawable for InnerCanvas {
//         fn set_texture(&self, _: scene::Texture) -> scene::Primitive {
//             Primitive::Other(Box::new(self.clone()))
//         }
//         fn draw_with_transform_clip(
//             &self,
//             ctx: &mut DrawContext,
//             transform: tiny_skia::Transform,
//             clip: Option<&tiny_skia::ClipMask>,
//         ) {
//             for s in &self.steps {
//                 let t = s.transform.post_concat(transform);
//                 s.primitive.draw_with_transform_clip(ctx, t, clip);
//             }
//         }
//         fn contains(&self, region: &scene::Region) -> bool {
//             Region::new(0., 0., self.width, self.height).rfit(&region)
//         }
//         fn primitive(&self) -> scene::Primitive {
//             Primitive::Other(Box::new(self.clone()))
//         }
//         fn get_texture(&self) -> scene::Texture {
//             Texture::Transparent
//         }
//     }
// }

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

/// A context provided to the sync methods of widgets.
///
/// The context dereferences the Data so widgets can interact with it.
/// It also contains the cache which is used for text layouting and fetching images.
pub struct SyncContext<'c, D> {
    data: &'c mut D,
    pub(crate) cache: &'c mut Cache,
    pub(crate) handle: Option<&'c mut dyn WindowHandle>,
}

/// A context provided to primitives during draw.
///
/// It does all the text rendering along and gives access to the backend which primitive can use to draw.
pub struct DrawContext<'c> {
    transform: Transform,
    path_builder: Option<PathBuilder>,
    pub(crate) backend: Backend<'c>,
    pub(crate) clipmask: Option<&'c mut ClipMask>,
    pub(crate) cache: &'c mut Cache,
    pub(crate) pending_damage: Vec<Region>,
}

/// A context provided during layout.
///
/// Currently it only holds the Cache.
pub struct LayoutCtx<'c> {
    pub(crate) cache: &'c mut Cache,
    pub(crate) handle: Option<&'c mut dyn WindowHandle>,
}

impl<'c> AsMut<Cache> for LayoutCtx<'c> {
    fn as_mut(&mut self) -> &mut Cache {
        self.cache
    }
}

impl<'c> LayoutCtx<'c> {
    pub fn new(cache: &'c mut Cache) -> LayoutCtx {
        LayoutCtx {
            cache,
            handle: None,
        }
    }
    pub fn new_with_handle(cache: &'c mut Cache, handle: &'c mut impl WindowHandle) -> Self {
        LayoutCtx {
            cache,
            handle: Some(handle),
        }
    }
    /// Return a reference to the WindowHandle
    pub fn handle(&mut self) -> Option<&mut &'c mut dyn WindowHandle> {
        self.handle.as_mut()
    }
}

/// A handle to the window state.
pub trait WindowHandle {
    fn close(&mut self);
    fn minimize(&mut self);
    fn maximize(&mut self);
    /// Launch a system menu
    fn menu(&mut self, x: f32, y: f32, serial: u32);
    /// Move the application window.
    ///
    /// The serial is provided by events.
    fn drag(&mut self, serial: u32);
    fn set_title(&mut self, title: String);
    fn set_cursor(&mut self, cursor: Cursor);
    fn get_state(&self) -> &[WindowState];
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
            handle: None,
        }
    }
    /// Creates a SyncContext with a WindowHandle
    pub fn new_with_handle(
        data: &'c mut D,
        cache: &'c mut Cache,
        handle: &'c mut impl WindowHandle,
    ) -> Self {
        Self {
            data,
            cache,
            handle: Some(handle),
        }
    }
    /// Return a reference to the WindowHandle
    pub fn handle(&mut self) -> Option<&mut &'c mut dyn WindowHandle> {
        self.handle.as_mut()
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

use tiny_skia::Transform;

impl<'c> DrawContext<'c> {
    pub fn new(backend: Backend<'c>, cache: &'c mut Cache) -> Self {
        Self {
            cache,
            backend,
            clipmask: None,
            transform: Transform::identity(),
            pending_damage: Vec::new(),
            path_builder: Some(PathBuilder::new()),
        }
    }
    pub fn with_clipmask(mut self, clipmask: Option<&'c mut ClipMask>) -> Self {
        self.clipmask = clipmask;
        self
    }
    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }
    /// Returns the PathBuilder.
    pub fn path_builder(&mut self) -> PathBuilder {
        self.path_builder
            .take()
            .expect("Please reset the path_builder once you're finished.")
    }
    pub fn reset(&mut self, path_builder: PathBuilder) {
        self.path_builder = Some(path_builder);
    }
    pub fn transform(&self) -> Transform {
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
    pub fn reset_clip(&mut self, region: Region) {
        let width = self.width();
        let height = self.height();
        if let Some(clipmask) = &mut self.clipmask {
            if width == region.width && height == region.height {
                clipmask.clear();
            } else {
                let mut pb = self.path_builder.take().unwrap();
                pb.push_rect(region.x, region.y, region.width, region.height);
                let path = pb.finish().unwrap();
                clipmask.set_path(width as u32, height as u32, &path, FillRule::Winding, false);
                self.path_builder = Some(path.clear());
            }
        }
    }
    pub fn set_clip(&mut self, region: Region) {
        let width = self.width();
        let height = self.height();
        if let Some(clipmask) = &mut self.clipmask {
            let mut pb = self.path_builder.take().unwrap();
            pb.push_rect(region.x, region.y, region.width, region.height);
            let path = pb.finish().unwrap();
            if clipmask.is_empty() {
                clipmask.set_path(width as u32, height as u32, &path, FillRule::Winding, false);
            } else {
                clipmask.intersect_path(&path, FillRule::Winding, false);
            }
            self.path_builder = Some(path.clear());
        }
    }
    /// This method is usually called when you want to clean up an area to draw on it.
    pub fn damage_region(&mut self, background: &Background, region: Region) {
        let blend;
        if let Some(background) = background.previous {
            self.damage_region(background, region);
            blend = BlendMode::SourceOver;
        } else {
            if let Some(last) = self.pending_damage.last() {
                if last.contains(region.x, region.y) {
                    if last
                        .merge(&region)
                        .substract(*last)
                        .into_iter()
                        .filter_map(|region| {
                            if !region.is_empty() {
                                self.damage_region(background, region);
                                Some(())
                            } else {
                                None
                            }
                        })
                        .reduce(|_, _| ())
                        .is_some()
                    {
                        return;
                    }
                }
            }
            self.pending_damage.push(region);
            blend = BlendMode::Source;
        }
        let clip_mask = self
            .clipmask
            .as_ref()
            .map(|clipmask| {
                if !clipmask.is_empty() {
                    Some(&**clipmask)
                } else {
                    None
                }
            })
            .flatten();
        match background.texture() {
            Texture::Color(color) => match &mut self.backend {
                Backend::Pixmap(dt) => {
                    dt.fill_rect(
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
                _ => {}
            },
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
                        self.transform,
                        clip_mask,
                    );
                }
                _ => {}
            },
            _ => {}
        }
    }
    pub fn damage_queue(&self) -> &[Region] {
        self.pending_damage.as_slice()
    }
    pub fn draw_kit(&mut self) -> (&mut Backend<'c>, Option<&ClipMask>) {
        (
            &mut self.backend,
            self.clipmask
                .as_ref()
                .map(|clipmask| {
                    if !clipmask.is_empty() {
                        Some(&**clipmask)
                    } else {
                        None
                    }
                })
                .flatten(),
        )
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
