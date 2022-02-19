use crate::*;
use crate::{
    scene::Texture,
    widgets::{layout::flex::Flex, shapes::*, *},
};
use std::ops::{Deref, DerefMut};
use widgets::shapes::Style;

struct Close {
    rect: Rectangle,
}

impl Close {
    fn new() -> Self {
        Close {
            rect: Rectangle::new(15., 15.).texture(theme::RED),
        }
    }
}

impl Geometry for Close {
    fn height(&self) -> f32 {
        self.rect.width()
    }
    fn width(&self) -> f32 {
        self.rect.height()
    }
}

impl<T> Widget<T> for Close {
    fn draw_scene(&mut self, mut scene: Scene) {
        scene.insert_primitive(&self.rect)
    }
    fn sync<'s>(&'s mut self, ctx: &mut SyncContext<T>, event: Event<'s>) -> Damage {
        if let Event::Pointer(x, y, p) = event {
            if self.contains(x, y) && p.left_button_click().is_some() {
                ctx.window().close();
            }
        }
        Damage::None
    }
    fn layout(&mut self, _ctx: &mut LayoutCtx, _constraints: &BoxConstraints) -> Size {
        (self.width(), self.height()).into()
    }
}

struct Maximize {
    maximized: bool,
    rect: BorderedRectangle,
}

impl Maximize {
    fn new() -> Self {
        Maximize {
            maximized: false,
            rect: BorderedRectangle::new(11., 11.)
                .texture(theme::BLUE)
                .border_width(2.),
        }
    }
}

impl Geometry for Maximize {
    fn height(&self) -> f32 {
        self.rect.width()
    }
    fn width(&self) -> f32 {
        self.rect.height()
    }
}

impl<T> Widget<T> for Maximize {
    fn draw_scene(&mut self, mut scene: Scene) {
        scene.insert_primitive(&self.rect)
    }
    fn sync<'s>(&'s mut self, ctx: &mut SyncContext<T>, event: Event<'s>) -> Damage {
        match event {
            Event::Pointer(x, y, p) => {
                if self.contains(x, y) && p.left_button_click().is_some() {
                    ctx.window().maximize();
                }
            }
            Event::Draw => {
                self.maximized = ctx
                    .window()
                    .get_state()
                    .iter()
                    .find(|s| WindowState::Maximized.eq(s))
                    .is_some();
            }
            _ => {}
        }
        Damage::None
    }
    fn layout(&mut self, _ctx: &mut LayoutCtx, _constraints: &BoxConstraints) -> Size {
        (self.width(), self.height()).into()
    }
}

struct Minimize {
    rect: Rectangle,
}

impl Minimize {
    fn new() -> Minimize {
        Minimize {
            rect: Rectangle::new(15., 4.).texture(theme::YELLOW),
        }
    }
}

impl Geometry for Minimize {
    fn height(&self) -> f32 {
        self.rect.width()
    }
    fn width(&self) -> f32 {
        self.rect.width()
    }
}

impl<T> Widget<T> for Minimize {
    fn draw_scene(&mut self, scene: Scene) {
        let width = 3.;
        scene
            .translate(0., (self.height() - width) / 2.)
            .insert_primitive(&self.rect)
    }
    fn sync<'s>(&'s mut self, ctx: &mut SyncContext<T>, event: Event<'s>) -> Damage {
        if let Event::Pointer(x, y, p) = event {
            if self.contains(x, y) && p.left_button_click().is_some() {
                ctx.window().minimize();
            }
        }
        Damage::None
    }
    fn layout(&mut self, _ctx: &mut LayoutCtx, _constraints: &BoxConstraints) -> Size {
        (self.width(), self.height()).into()
    }
}

struct Header<W> {
    widget: W,
    size: Size,
}

impl<W> Geometry for Header<W> {
    fn width(&self) -> f32 {
        self.size.width
    }
    fn height(&self) -> f32 {
        self.size.height
    }
}

impl<T, W: Widget<T>> Widget<T> for Header<W> {
    fn draw_scene(&mut self, scene: Scene) {
        self.widget.draw_scene(scene)
    }
    fn sync<'s>(&'s mut self, ctx: &mut SyncContext<T>, event: Event<'s>) -> Damage {
        match event {
            Event::Pointer(x, y, p) => {
                if self.contains(x, y) {
                    if let Some(serial) = p.left_button_click() {
                        ctx.window()._move(serial);
                    } else if let Some(serial) = p.right_button_click() {
                        // Cursor position is relative.
                        // Only works because the Header is the first entered widget
                        ctx.window().menu(x, y, serial);
                    }
                }
            }
            _ => {}
        }
        self.widget.sync(ctx, event)
    }
    fn layout(&mut self, ctx: &mut LayoutCtx, constraints: &BoxConstraints) -> Size {
        self.size = self.widget.layout(ctx, constraints);
        self.size
    }
}

impl<W> Deref for Header<W> {
    type Target = W;
    fn deref(&self) -> &Self::Target {
        &self.widget
    }
}

impl<W> DerefMut for Header<W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.widget
    }
}

pub struct Window<T, H, W>
where
    H: Widget<T>,
    W: Widget<T>,
{
    size: Size,
    activated: bool,
    positioned: bool,
    /// Top window decoration
    header: Header<H>,
    /// The window's content
    window: Positioner<Proxy<WidgetStyle<T, W>>>,
    /// The background of the decorations
    decoration: Texture,
    /// The radius of window borders
    radius: (f32, f32, f32, f32),
    /// Alternate background of the decorations
    alternate: Option<Texture>,
}

impl<T, H, W> Window<T, H, W>
where
    H: Widget<T>,
    W: Widget<T>,
{
    pub fn new(header: H, widget: W) -> Self {
        Window {
            size: Size::default(),
            header: Header {
                size: Size::default(),
                widget: header,
            },
            activated: false,
            positioned: false,
            window: Positioner::new(Proxy::new(WidgetStyle::new(widget))),
            radius: (0., 0., 0., 0.),
            decoration: theme::BG2.into(),
            alternate: None,
        }
    }
    pub fn set_decoration(&mut self, texture: impl Into<Texture>, width: f32) {
        let texture = texture.into();
        self.window.set_border(texture.clone(), width);
        self.decoration = texture;
    }
    pub fn decoration(mut self, texture: impl Into<Texture>, width: f32) -> Self {
        self.set_decoration(texture, width);
        self
    }
    pub fn set_alternate_decoration(&mut self, texture: impl Into<Texture>) {
        self.alternate = Some(texture.into());
    }
    pub fn alternate_decoration(mut self, texture: impl Into<Texture>) -> Self {
        self.set_alternate_decoration(texture);
        self
    }
}

impl<T, H, W> Widget<T> for Window<T, H, W>
where
    H: Widget<T> + Style,
    W: Widget<T>,
{
    fn draw_scene(&mut self, mut scene: Scene) {
        if let Some(scene) = scene.auto_next(self.size) {
            self.header.draw_scene(scene)
        }
        if let Some(scene) = scene.auto_next(self.size) {
            self.window.draw_scene(scene)
        }
    }
    fn sync<'s>(&'s mut self, ctx: &mut SyncContext<T>, event: Event<'s>) -> Damage {
        match event {
            Event::Configure => {
                let window = ctx.window();
                let state = window.get_state();
                let mut activated = false;
                let mut positioned = false;
                for state in state.iter().rev() {
                    match state {
                        WindowState::Activated => {
                            activated = true;
                            if self.alternate.is_some() {
                                if !self.activated {
                                    self.header.set_texture(self.decoration.clone());
                                    self.window.set_border_texture(self.decoration.clone());
                                }
                            }
                        }
                        WindowState::TiledLeft
                        | WindowState::TiledRight
                        | WindowState::TiledBottom
                        | WindowState::TiledTop
                        | WindowState::Maximized
                        | WindowState::Fullscreen => {
                            positioned = true;
                            self.header.set_top_left_radius(0.);
                            self.header.set_top_right_radius(0.);
                            self.window.set_bottom_right_radius(0.);
                            self.window.set_bottom_left_radius(0.);
                        }
                        _ => {}
                    }
                }
                if !activated {
                    if let Some(ref texture) = self.alternate {
                        self.header.set_texture(texture.clone());
                        self.window.set_border_texture(texture.clone());
                    }
                }
                if !self.activated && !activated {
                    self.set_top_left_radius(self.radius.0);
                    self.set_top_right_radius(self.radius.1);
                    self.set_bottom_right_radius(self.radius.2);
                    self.set_bottom_left_radius(self.radius.3);
                }
                if !positioned && self.positioned {
                    self.positioned = false;
                    self.set_top_left_radius(self.radius.0);
                    self.set_top_right_radius(self.radius.1);
                    self.set_bottom_right_radius(self.radius.2);
                    self.set_bottom_left_radius(self.radius.3);
                }
                self.activated = activated;
                self.positioned = positioned;
                self.header
                    .sync(ctx, event)
                    .max(self.window.sync(ctx, event))
            }
            _ => {
                let header = self.header.sync(ctx, event);
                let window = self.window.sync(ctx, event);
                header.max(window)
            }
        }
    }
    fn layout(&mut self, ctx: &mut LayoutCtx, constraints: &BoxConstraints) -> Size {
        let (h_width, h_height) = self
            .header
            .layout(
                ctx,
                &constraints
                    .with_min(constraints.minimum_width(), 0.)
                    .with_max(constraints.maximum_width(), 0.),
            )
            .into();
        let (b_width, b_height) = self
            .window
            .layout(ctx, &constraints.crop(0., h_height))
            .into();
        self.window.set_coords(0., h_height);
        self.size = (b_width.max(h_width), h_height + b_height).into();
        self.size
    }
}

impl<T, H, W> Style for Window<T, H, W>
where
    H: Widget<T> + Style,
    W: Widget<T>,
{
    fn set_texture<B: Into<Texture>>(&mut self, texture: B) {
        self.window.set_texture(texture);
    }
    fn set_top_left_radius(&mut self, radius: f32) {
        self.radius.0 = radius;
        self.header.set_top_left_radius(radius);
    }
    fn set_top_right_radius(&mut self, radius: f32) {
        self.radius.1 = radius;
        self.header.set_top_right_radius(radius);
    }
    fn set_bottom_right_radius(&mut self, radius: f32) {
        self.radius.2 = radius;
        self.window.set_bottom_right_radius(radius);
    }
    fn set_bottom_left_radius(&mut self, radius: f32) {
        self.radius.3 = radius;
        self.window.set_bottom_left_radius(radius);
    }
}

impl<T, H, W> Deref for Window<T, H, W>
where
    H: Widget<T>,
    W: Widget<T>,
{
    type Target = W;
    fn deref(&self) -> &Self::Target {
        self.window.widget.deref()
    }
}

impl<T, H, W> DerefMut for Window<T, H, W>
where
    H: Widget<T>,
    W: Widget<T>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.window.widget.deref_mut()
    }
}

fn wm_button<T>() -> impl Widget<T>
where
    T: 'static,
{
    Flex::row()
        .with(Minimize::new().padding_left(5.).padding_right(5.))
        .with(Maximize::new().padding_left(5.).padding_right(5.))
        .with(Close::new().padding_left(5.))
}

fn headerbar<T: 'static>(widget: impl Widget<T> + 'static) -> impl Widget<T> {
    Flex::row()
        .with(widget)
        .with(wm_button().clamp().anchor(END, CENTER))
}

pub fn default_window<T, W>(
    header: impl Widget<T> + 'static,
    widget: W,
) -> Window<T, impl Widget<T> + Style, W>
where
    T: 'static,
    W: Widget<T>,
{
    Window::new(
        headerbar(header)
        	.background(theme::BG2)
        	.padding(10.),
        widget)
}
