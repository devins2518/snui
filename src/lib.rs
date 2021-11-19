pub mod context;
pub mod data;
pub mod font;
pub mod scene;
pub mod wayland;
pub mod widgets;

use context::*;
use scene::RenderNode;
use widgets::button::Button;
use widgets::shapes::WidgetExt;

pub const FG: u32 = 0xff_C8_BA_A4;

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
    Keyboard(Key<'d>),
    Message(&'d str),
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
    fn set_size(&mut self, width: f32, height: f32) -> Result<(), (f32, f32)>;
}

pub trait Primitive: Geometry + std::fmt::Debug {
    fn draw(&self, x: f32, y: f32, ctx: &mut DrawContext);
}

pub trait Widget: Geometry {
    // Widgets are expected to compute their layout when
    // they're creating their render node.
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode;
    // I don't think the user should have access to the context as is
    // because it exposes the Backend and I don't want a widget to have the ability
    // to draw so I should create another DrawContext, perhaps SyncDrawContext and rename the
    // previous DrawDrawContext. The SyncDrawContext would hold the Data.
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext, event: Event);
}

pub trait Wrapable: Widget + Sized {
    fn wrap(self) -> WidgetExt<Self>;
    fn into_button(
        self,
        cb: impl for<'d> FnMut(&'d mut Self, &'d mut SyncContext, Pointer) + 'static,
    ) -> Button<Self>;
}

impl<W> Wrapable for W
where
    W: Widget,
{
    fn wrap(self) -> WidgetExt<W> {
        WidgetExt::default(self)
    }
    fn into_button(
        self,
        cb: impl for<'d> FnMut(&'d mut Self, &'d mut SyncContext, Pointer) + 'static,
    ) -> Button<Self> {
        Button::new(self, cb)
    }
}
