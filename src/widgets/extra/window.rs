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
    fn update<'s>(&'s mut self, ctx: &mut SyncContext<T>) -> Damage {
        Damage::None
    }
    fn event<'s>(&'s mut self, ctx: &mut SyncContext<T>, event: Event<'s>) -> Damage {
        if let Event::Pointer(MouseEvent {
            pointer,
            ref position,
        }) = event
        {
            if self.contains(position) && pointer.left_button_click().is_some() {
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
    fn update<'s>(&'s mut self, ctx: &mut SyncContext<T>) -> Damage {
        Damage::None
    }
    fn event<'s>(&'s mut self, ctx: &mut SyncContext<T>, event: Event<'s>) -> Damage {
        match event {
            Event::Pointer(MouseEvent {
                pointer,
                ref position,
            }) => {
                if self.contains(position) && pointer.left_button_click().is_some() {
                    ctx.window().maximize();
                }
            }
            Event::Configure => {
                self.maximized = ctx
                    .window()
                    .get_state()
                    .iter()
                    .any(|s| WindowState::Maximized.eq(s));
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
    fn update<'s>(&'s mut self, ctx: &mut SyncContext<T>) -> Damage {
        Damage::None
    }
    fn event<'s>(&'s mut self, ctx: &mut SyncContext<T>, event: Event<'s>) -> Damage {
        if let Event::Pointer(MouseEvent {
            pointer,
            ref position,
        }) = event
        {
            if self.contains(position) && pointer.left_button_click().is_some() {
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
    fn update<'s>(&'s mut self, ctx: &mut SyncContext<T>) -> Damage {
        self.widget.update(ctx)
    }
    fn event<'s>(&'s mut self, ctx: &mut SyncContext<T>, event: Event<'s>) -> Damage {
        match event {
            Event::Pointer(MouseEvent { pointer, position }) => {
                if self.contains(&position) {
                    if let Some(serial) = pointer.left_button_click() {
                        ctx.window()._move(serial);
                    } else if let Some(serial) = pointer.right_button_click() {
                        // Cursor position is relative.
                        // Only works because the Header is the first entered widget
                        ctx.window().show_menu(Menu::System { position, serial });
                    }
                }
            }
            _ => {}
        }
        self.widget.event(ctx, event)
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
    activated: bool,
    positioned: bool,
    /// Top window decoration
    header: Positioner<Header<H>>,
    /// The window's content
    window: Positioner<Proxy<WidgetStyle<T, W>>>,
    /// The background of the decorations
    decoration: Texture,
    border: BorderedRectangle,
    /// The radius of window borders
    radius: (f32, f32, f32, f32),
    /// Alternate background of the decorations
    alternate: Option<Texture>,
}

impl<T, H, W> Window<T, H, W>
where
    H: Widget<T> + Style,
    W: Widget<T>,
{
    pub fn new(header: H, widget: W) -> Self {
        Window {
            header: Positioner::new(Header {
                size: Size::default(),
                widget: header,
            }),
            activated: false,
            positioned: false,
            border: BorderedRectangle::default(),
            window: Positioner::new(Proxy::new(WidgetStyle::new(widget))),
            radius: (0., 0., 0., 0.),
            decoration: theme::BG2.into(),
            alternate: None,
        }
    }
    pub fn set_decoration(&mut self, texture: impl Into<Texture>, width: f32) {
        let texture = texture.into();
        self.border.set_border_width(width);
        self.border.set_texture(texture.clone());
        self.header.set_texture(texture.clone());
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
        if let Some(mut scene) = scene.apply_border(&self.border) {
            if let Some(scene) = scene.next(self.border.size()) {
                self.header.draw_scene(scene)
            }
            if let Some(scene) = scene.next(self.border.size()) {
                self.window.draw_scene(scene)
            }
        }
    }
    fn update<'s>(&'s mut self, ctx: &mut SyncContext<T>) -> Damage {
        let header = self.header.update(ctx);
        let window = self.window.update(ctx);
        header.max(window)
    }
    fn event<'s>(&'s mut self, ctx: &mut SyncContext<T>, event: Event<'s>) -> Damage {
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
                            if self.alternate.is_some() && !self.activated {
                                self.header.set_texture(self.decoration.clone());
                                self.border.set_texture(self.decoration.clone());
                            }
                        }
                        WindowState::TiledLeft
                        | WindowState::TiledRight
                        | WindowState::TiledBottom
                        | WindowState::TiledTop
                        | WindowState::Maximized
                        | WindowState::Fullscreen => {
                            positioned = true;
                            let radius = self.radius;
                            self.set_radius(0.);
                            self.radius = radius;
                        }
                        _ => {}
                    }
                }
                if !activated {
                    if let Some(ref texture) = self.alternate {
                        self.border.set_texture(texture.clone());
                        self.header.set_texture(texture.clone());
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
                    .event(ctx, event)
                    .max(self.window.event(ctx, event))
            }
            _ => {
                let header = self.header.event(ctx, event);
                let window = self.window.event(ctx, event);
                header.max(window)
            }
        }
    }
    fn layout<'l>(&mut self, ctx: &mut LayoutCtx, constraints: &BoxConstraints) -> Size {
        let (h_width, h_height) = self
            .header
            .layout(
                ctx,
                &constraints
                    .with_max(constraints.maximum_width(), 0.)
                    .crop(self.border.border_width * 2., 0.),
            )
            .into();
        let (w_width, w_height) = self
            .window
            .layout(
                ctx,
                &constraints.crop(
                    self.border.border_width * 2.,
                    h_height + self.border.border_width * 2.,
                ),
            )
            .into();
        self.header
            .set_coords(self.border.border_width, self.border.border_width);
        self.window.set_coords(
            self.border.border_width,
            h_height + self.border.border_width,
        );
        self.border
            .set_size(w_width.max(h_width), h_height + w_height);
        self.border.size()
    }
}

use std::f32::consts::FRAC_1_SQRT_2;

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
        self.header.set_top_left_radius(radius * FRAC_1_SQRT_2);
        self.border.set_top_left_radius(radius);
    }
    fn set_top_right_radius(&mut self, radius: f32) {
        self.radius.1 = radius;
        self.header.set_top_right_radius(radius * FRAC_1_SQRT_2);
        self.border.set_top_right_radius(radius);
    }
    fn set_bottom_right_radius(&mut self, radius: f32) {
        self.radius.2 = radius;
        self.window.set_bottom_right_radius(radius * FRAC_1_SQRT_2);
        self.border.set_bottom_right_radius(radius);
    }
    fn set_bottom_left_radius(&mut self, radius: f32) {
        self.radius.3 = radius;
        self.window.set_bottom_left_radius(radius * FRAC_1_SQRT_2);
        self.border.set_bottom_left_radius(radius);
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
        .with_child(Minimize::new().padding_left(5.).padding_right(5.))
        .with_child(Maximize::new().padding_left(5.).padding_right(5.))
        .with_child(Close::new().padding_left(5.))
}

fn headerbar<T: 'static>(widget: impl Widget<T> + 'static) -> impl Widget<T> {
    Flex::row()
        .with_child(widget.clamp().anchor(START, CENTER))
        .with_child(wm_button().clamp().anchor(END, CENTER))
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
        headerbar(header).background(theme::BG2).padding(10.),
        widget,
    )
}
