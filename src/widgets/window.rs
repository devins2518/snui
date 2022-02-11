use crate::*;
use crate::{
    scene::{Instruction, Texture},
    widgets::{
        layout::{dynamic::DynamicLayout, simple::SimpleLayout},
        shapes::*,
        *,
    },
};
use std::ops::{Deref, DerefMut};

struct Close {}

impl Geometry for Close {
    fn height(&self) -> f32 {
        15.
    }
    fn width(&self) -> f32 {
        15.
    }
}

impl<D> Widget<D> for Close {
    fn create_node(&mut self, transform: Transform) -> RenderNode {
        let mut canvas = self.create_canvas(transform);

        use std::f32::consts::FRAC_1_SQRT_2;

        let width = self.width() * FRAC_1_SQRT_2;
        let height = self.height() * FRAC_1_SQRT_2;

        let r = Rectangle::new(width, height).background(theme::RED);

        canvas.draw_at_angle(
            (self.width() - width) / 2.,
            (self.height() - height) / 2.,
            r,
            -45.,
        );

        canvas.finish()
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<D>, event: Event<'d>) -> Damage {
        if let Event::Pointer(x, y, p) = event {
            if self.contains(x, y) && p.left_button_click().is_some() {
                if let Some(w_handle) = ctx.handle() {
                    w_handle.close();
                }
            }
        }
        Damage::None
    }
    fn prepare_draw(&mut self) {}
    fn layout(&mut self, _: &mut LayoutCtx) -> (f32, f32) {
        (self.width(), self.height())
    }
}

// This is essentially the close button
struct Maximize {
    maximized: bool,
}

impl Geometry for Maximize {
    fn height(&self) -> f32 {
        15.
    }
    fn width(&self) -> f32 {
        15.
    }
}

impl<D> Widget<D> for Maximize {
    fn create_node(&mut self, transform: Transform) -> RenderNode {
        if self.maximized {
            Instruction {
                transform,
                primitive: Rectangle::new(self.width(), self.height())
                    .background(theme::BLU)
                    .into(),
            }
            .into()
        } else {
            let thickness = 2.;
            Instruction {
                transform,
                primitive: Rectangle::new(
                    self.width() - 2. * thickness,
                    self.height() - 2. * thickness,
                )
                .border(theme::BLU, thickness)
                .into(),
            }
            .into()
        }
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<D>, event: Event<'d>) -> Damage {
        match event {
            Event::Pointer(x, y, p) => {
                if self.contains(x, y) && p.left_button_click().is_some() {
                    if let Some(w_handle) = ctx.handle() {
                        w_handle.maximize();
                    }
                }
            }
            Event::Configure(state) => {
                self.maximized = state
                    .iter()
                    .find(|s| WindowState::Maximized.eq(s))
                    .is_some();
            }
            _ => {}
        }
        Damage::None
    }
    fn prepare_draw(&mut self) {}
    fn layout(&mut self, _: &mut LayoutCtx) -> (f32, f32) {
        (self.width(), self.height())
    }
}

struct Minimize {}

impl Geometry for Minimize {
    fn height(&self) -> f32 {
        15.
    }
    fn width(&self) -> f32 {
        15.
    }
}

impl<D> Widget<D> for Minimize {
    fn create_node(&mut self, transform: Transform) -> RenderNode {
        let r = Rectangle::new(self.width(), 3.).background(theme::YEL);

        Instruction::new(transform.pre_translate(0., 6.), r).into()
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<D>, event: Event<'d>) -> Damage {
        if let Event::Pointer(x, y, p) = event {
            if self.contains(x, y) && p.left_button_click().is_some() {
                if let Some(w_handle) = ctx.handle() {
                    w_handle.minimize();
                }
            }
        }
        Damage::None
    }
    fn prepare_draw(&mut self) {}
    fn layout(&mut self, _: &mut LayoutCtx) -> (f32, f32) {
        (self.width(), self.height())
    }
}

fn wm_button<D>() -> impl Widget<D>
where
    D: 'static,
{
    let mut l = SimpleLayout::default().spacing(15.);
    l.add(Minimize {});
    l.add(Maximize { maximized: false });
    l.add(Close {});
    l.justify(CENTER);
    l
}

fn headerbar<D: 'static>(widget: impl Widget<D> + 'static) -> impl Widget<D> {
    let mut l = DynamicLayout::new();
    l.add(widget.clamp().anchor(START, CENTER));
    l.add(wm_button().clamp().anchor(END, CENTER));
    l
}

pub struct Window<H, W> {
    activated: bool,
    positioned: bool,
    /// Top window decoration
    header: Header<H>,
    /// The window's content
    body: Positioner<Proxy<W>>,
    /// The background of the headerbar decoration
    background: Texture,
    /// The radius of window borders
    radius: (f32, f32, f32, f32),
    /// Alternative background of the decoration
    alternate: Option<Texture>,
}

impl<H, W> Window<H, W>
where
    H: Style,
    W: Style,
{
    pub fn set_alternate_background<B: Into<Texture>>(&mut self, background: B) {
        self.alternate = Some(background.into());
    }
    pub fn alternate_background<B: Into<Texture>>(mut self, background: B) -> Self {
        self.set_alternate_background(background);
        self
    }
}

impl<H, W> Geometry for Window<H, W>
where
    H: Geometry,
    W: Geometry,
{
    fn width(&self) -> f32 {
        self.header.width().max(self.body.width())
    }
    fn height(&self) -> f32 {
        self.body.height() + self.header.height()
    }
    fn set_width(&mut self, width: f32) {
        let c_width = width.min(self.maximum_width());
        self.body.set_width(c_width);
        self.header.set_width(c_width);
    }
    fn set_height(&mut self, height: f32) {
        self.body.set_height(height - self.header.height())
    }
    fn minimum_width(&self) -> f32 {
        self.header.minimum_width().max(self.body.minimum_width())
    }
    fn maximum_width(&self) -> f32 {
        self.header.maximum_width().min(self.body.maximum_width())
    }
    fn minimum_height(&self) -> f32 {
        self.header.minimum_height() + self.body.minimum_height()
    }
    fn maximum_height(&self) -> f32 {
        self.header.maximum_height() + self.body.maximum_height()
    }
}

impl<D, H, W> Widget<D> for Window<H, W>
where
    H: Widget<D> + Style,
    W: Widget<D> + Style,
{
    fn create_node(&mut self, transform: Transform) -> RenderNode {
        let header = self.header.create_node(transform);
        if !header.is_none() {
            self.body.set_coords(0., header.height());
        }
        let body = self.body.create_node(transform);
        if header.is_none() && body.is_none() {
            return RenderNode::None;
        }
        RenderNode::Container(vec![header, body])
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<D>, event: Event<'d>) -> Damage {
        match event {
            Event::Pointer(_, _, p) => {
                if let Pointer::Enter = p {
                    if let Some(w_handle) = ctx.handle() {
                        w_handle.set_cursor(Cursor::Arrow);
                    }
                }
                self.header.sync(ctx, event).max(self.body.sync(ctx, event))
            }
            Event::Configure(state) => {
                let mut activated = false;
                let mut positioned = false;
                for state in state.iter().rev() {
                    match state {
                        WindowState::Activated => {
                            activated = true;
                            if self.alternate.is_some() {
                                if !self.activated {
                                    self.header.set_background(self.background.clone());
                                    self.body.set_border_texture(self.background.clone());
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
                            self.body.set_bottom_right_radius(0.);
                            self.body.set_bottom_left_radius(0.);
                        }
                        _ => {}
                    }
                }
                if !activated {
                    if let Some(ref texture) = self.alternate {
                        self.header.set_background(texture.clone());
                        self.body.set_border_texture(texture.clone());
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
                self.header.sync(ctx, event).max(self.body.sync(ctx, event))
            }
            Event::Draw => {
                self.set_width(self.width());
                self.header.sync(ctx, event).max(self.body.sync(ctx, event))
            }
            _ => self.header.sync(ctx, event).max(self.body.sync(ctx, event)),
        }
    }
    fn prepare_draw(&mut self) {
        self.set_width(self.width());
        self.header.prepare_draw();
        self.body.prepare_draw();
    }
    fn layout(&mut self, ctx: &mut LayoutCtx) -> (f32, f32) {
        let (h_width, h_height) = self.header.layout(ctx);
        let (b_width, b_height) = self.body.layout(ctx);
        (h_width.max(b_width), h_height + b_height)
    }
}

impl<H, W> Style for Window<H, W>
where
    H: Style,
    W: Style,
{
    fn set_background<B: Into<scene::Texture>>(&mut self, background: B) {
        self.body.set_background(background.into());
    }
    fn set_border_texture<T: Into<Texture>>(&mut self, texture: T) {
        let texture = texture.into();
        self.background = texture.clone();
        self.header.set_background(texture.clone());
        self.body.set_border_texture(texture);
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
        self.body.set_bottom_right_radius(radius);
    }
    fn set_bottom_left_radius(&mut self, radius: f32) {
        self.radius.3 = radius;
        self.body.set_bottom_left_radius(radius);
    }
    fn set_border_size(&mut self, size: f32) {
        self.body.set_border_size(size);
    }
}

impl<H, W> Deref for Window<H, W> {
    type Target = W;
    fn deref(&self) -> &Self::Target {
        self.body.widget.deref()
    }
}

impl<H, W> DerefMut for Window<H, W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.body.widget.deref_mut()
    }
}

impl<H, W> Window<H, W>
where
    H: Geometry + Style,
    W: Geometry + Style,
{
    pub fn new(header: H, widget: W) -> Self {
        Window {
            header: Header { widget: header },
            activated: false,
            positioned: false,
            body: widget.child(),
            radius: (0., 0., 0., 0.),
            background: theme::BG2.into(),
            alternate: None,
        }
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

struct Header<W> {
    widget: W,
}

impl<W: Geometry> Geometry for Header<W> {
    fn width(&self) -> f32 {
        self.widget.width()
    }
    fn set_width(&mut self, width: f32) {
        self.widget.set_width(width)
    }
    fn height(&self) -> f32 {
        self.widget.height()
    }
    fn set_height(&mut self, height: f32) {
        self.widget.set_height(height)
    }
    fn minimum_width(&self) -> f32 {
        self.widget.minimum_width()
    }
    fn maximum_width(&self) -> f32 {
        self.widget.maximum_width()
    }
    fn minimum_height(&self) -> f32 {
        self.widget.minimum_height()
    }
    fn maximum_height(&self) -> f32 {
        self.widget.maximum_height()
    }
}

impl<D, W: Widget<D>> Widget<D> for Header<W> {
    fn create_node(&mut self, transform: Transform) -> RenderNode {
        self.widget.create_node(transform)
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<D>, event: Event<'d>) -> Damage {
        match event {
            Event::Pointer(x, y, p) => {
                if self.contains(x, y) {
                    if let Some(w_handle) = ctx.handle() {
                        if let Some(serial) = p.left_button_click() {
                            w_handle.drag(serial);
                        } else if let Some(serial) = p.right_button_click() {
                            w_handle.menu(x, y, serial);
                        }
                    }
                }
            }
            _ => {}
        }
        self.widget.sync(ctx, event)
    }
    fn prepare_draw(&mut self) {
        self.widget.prepare_draw()
    }
    fn layout(&mut self, ctx: &mut LayoutCtx) -> (f32, f32) {
        self.widget.layout(ctx)
    }
}

pub fn default_window<D, W>(
    header: impl Widget<D> + 'static,
    widget: W,
) -> Window<impl Widget<D> + Style, W>
where
    D: 'static,
    W: Widget<D> + Style,
{
    let header = headerbar(header)
        .style()
        .background(theme::BG2)
        .padding(10.);

    Window::new(header, widget)
}
