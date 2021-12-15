pub mod context;
pub mod data;
pub mod font;
pub mod scene;
pub mod wayland;
pub mod widgets;

use context::*;
use scene::RenderNode;
pub use tiny_skia::*;
use widgets::button::{Button, Proxy};
use widgets::shapes::WidgetExt;
use widgets::WidgetBox;

pub const FG: u32 = 0xff_C8_BA_A4;

pub fn u32_to_source(color: u32) -> Color {
    let color = color.to_be_bytes();
    Color::from_rgba8(color[3], color[2], color[1], color[0])
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Orientation {
    Vertical,
    Horizontal,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub caps_lock: bool,
    pub logo: bool,
    pub num_lock: bool,
}

impl Modifiers {
    pub fn default() -> Self {
        Modifiers {
            ctrl: false,
            alt: false,
            shift: false,
            caps_lock: false,
            logo: false,
            num_lock: false,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Key<'k> {
    pub utf8: Option<&'k String>,
    pub value: &'k [u32],
    pub modifiers: Modifiers,
    pub pressed: bool,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Pointer {
    MouseClick {
        time: u32,
        button: MouseButton,
        pressed: bool,
    },
    Scroll {
        orientation: Orientation,
        value: f32,
    },
    Hover,
    Enter,
    Leave,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    Extra(u32),
}

impl MouseButton {
    fn new(button: u32) -> MouseButton {
        let button = button % 272;
        match button {
            0 => MouseButton::Left,
            1 => MouseButton::Right,
            2 => MouseButton::Middle,
            _ => MouseButton::Extra(button),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event<'d> {
    Commit,
    Prepare,
    // Your message object
    Message(data::Message<'d>),
    // Waiting for Wayland-rs 0.3.0 to implement it
    Keyboard(Key<'d>),
    Pointer(f32, f32, Pointer),
}

pub trait Container: Geometry {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn add(&mut self, widget: impl Widget + 'static);
}

pub trait Geometry {
    fn width(&self) -> f32;
    fn height(&self) -> f32;
    fn set_width(&mut self, _width: f32) -> Result<(), f32> {
        Err(self.width())
    }
    fn set_height(&mut self, _height: f32) -> Result<(), f32> {
        Err(self.height())
    }
    fn set_size(&mut self, width: f32, height: f32) -> Result<(), (f32, f32)> {
        let err_width = self.set_width(width);
        let err_height = self.set_height(height);

        if let Err(width) = err_width {
            if let Err(height) = err_height {
                Err((width, height))
            } else {
                Err((width, height))
            }
        } else {
            if let Err(height) = err_height {
                Err((width, height))
            } else {
                Ok(())
            }
        }
    }
}

pub trait Primitive: Geometry + std::fmt::Debug {
    fn draw(&self, x: f32, y: f32, ctx: &mut DrawContext) {
        self.draw_with_transform_clip(ctx, tiny_skia::Transform::from_translate(x, y), None);
    }
    fn draw_with_clip(
        &self,
        x: f32,
        y: f32,
        ctx: &mut DrawContext,
        clip: Option<&tiny_skia::ClipMask>,
    ) {
        self.draw_with_transform_clip(ctx, tiny_skia::Transform::from_translate(x, y), clip);
    }
    fn draw_with_tranform(
        &self,
        x: f32,
        y: f32,
        ctx: &mut DrawContext,
        tranform: tiny_skia::Transform,
    ) {
        self.draw_with_transform_clip(ctx, tranform.pre_translate(x, y), None);
    }
    fn draw_with_transform_clip(
        &self,
        ctx: &mut DrawContext,
        transform: tiny_skia::Transform,
        clip: Option<&tiny_skia::ClipMask>,
    );
    fn get_background(&self) -> scene::Background;
    fn apply_background(&self, background: scene::Background) -> scene::PrimitiveType;
    // Tell if the region can fit inside the Primitive
    // The coordinates will be relative to the Primitive
    fn contains(&self, region: &scene::Region) -> bool;
    // Basically Clone
    fn into_primitive(&self) -> scene::PrimitiveType;
}

pub trait Widget: Geometry {
    // Widgets are expected to compute their layout when
    // they're creating their render node.
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode;
    // Interface to communicate with the application and retained mode draw operation
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext, event: Event);
    fn create_canvas(&self, x: f32, y: f32) -> context::canvas::Canvas {
        context::canvas::Canvas::new(scene::Region::new(x, y, self.width(), self.height()))
    }
    fn contains(&self, x: f32, y: f32) -> bool {
        scene::Region::new(0., 0., self.width(), self.height()).contains(x, y)
    }
}

pub trait Wrapable: Widget + Sized {
    fn wrap(self) -> WidgetExt<Self>;
    fn into_box(self) -> WidgetBox<Self>;
    fn into_button(
        self,
        cb: impl for<'d> FnMut(&'d mut Proxy<Self>, &'d mut SyncContext, Pointer) + 'static,
    ) -> Button<Self>;
}

impl<W> Wrapable for W
where
    W: Widget,
{
    fn into_box(self) -> WidgetBox<Self> {
        WidgetBox::new(self)
    }
    fn wrap(self) -> WidgetExt<W> {
        WidgetExt::new(self)
    }
    fn into_button(
        self,
        cb: impl for<'d> FnMut(&'d mut Proxy<Self>, &'d mut SyncContext, Pointer) + 'static,
    ) -> Button<Self> {
        Button::new(self, cb)
    }
}
