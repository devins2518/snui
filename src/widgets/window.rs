use crate::*;
use crate::{
    scene::{Instruction, Texture},
    widgets::{container::*, shapes::*, *},
};
use std::ops::{Deref, DerefMut};

#[derive(Clone, PartialEq, Debug)]
pub enum WindowRequest {
    Move(u32),
    Close,
    Maximize,
    Minimize,
    Menu(f32, f32, u32),
    Title(String),
}

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

        let r = Rectangle::empty(width, height).background(style::RED);

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
            if let Pointer::MouseClick {
                serial: _,
                button,
                pressed,
            } = p
            {
                if self.contains(x, y) {
                    if button.is_left() && pressed {
                        ctx.window_request(WindowRequest::Close);
                    }
                }
            }
        }
        Damage::None
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
                primitive: Rectangle::new(
                    self.width(),
                    self.height(),
                    ShapeStyle::solid(style::BLU),
                )
                .into(),
            }
            .into()
        } else {
            let thickness = 2.;
            Instruction {
                transform,
                primitive: Rectangle::empty(
                    self.width() - 2. * thickness,
                    self.height() - 2. * thickness,
                )
                .border(style::BLU, thickness)
                .into(),
            }
            .into()
        }
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<D>, event: Event<'d>) -> Damage {
        match event {
            Event::Pointer(x, y, p) => {
                if let Pointer::MouseClick {
                    serial: _,
                    button,
                    pressed,
                } = p
                {
                    if self.contains(x, y) {
                        if button.is_left() && pressed {
                            ctx.window_request(WindowRequest::Maximize);
                        }
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
        let mut canvas = self.create_canvas(transform);

        use std::f32::consts::FRAC_1_SQRT_2;

        let width = self.width() * FRAC_1_SQRT_2;
        let height = self.height() * FRAC_1_SQRT_2;

        let r = Rectangle::empty(width, height).background(style::YEL);

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
            if let Pointer::MouseClick {
                serial: _,
                button,
                pressed,
            } = p
            {
                if self.contains(x, y) {
                    if button.is_left() && pressed {
                        ctx.window_request(WindowRequest::Minimize);
                    }
                }
            }
        }
        Damage::None
    }
}

fn wm_button<D>() -> impl Widget<D>
where
    D: 'static,
{
    let mut l = WidgetLayout::default().spacing(15.);
    l.add(Minimize {});
    l.add(Maximize { maximized: false });
    l.add(Close {});
    l.justify(CENTER);
    l
}

fn headerbar<D: 'static>(widget: impl Widget<D> + 'static) -> impl Widget<D> {
    let mut l = LayoutBox::new();
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
        self.header.width()
    }
    fn height(&self) -> f32 {
        self.body.height() + self.header.height()
    }
    fn set_width(&mut self, width: f32) -> Result<(), f32> {
        if let Err(width) = self.body.set_width(width) {
            if let Err(width) = self.header.set_width(width) {
                self.body.set_width(width)
            } else {
                Ok(())
            }
        } else {
            if let Err(width) = self.header.set_width(width) {
                self.body.set_width(width)
            } else {
                Ok(())
            }
        }
    }
    fn set_height(&mut self, height: f32) -> Result<(), f32> {
        self.body.set_height(height - self.header.height())
    }
}

impl<D, H, W> Widget<D> for Window<H, W>
where
    H: Widget<D> + Style,
    W: Widget<D> + Style,
{
    fn create_node(&mut self, transform: Transform) -> RenderNode {
        let h = self.header.create_node(transform);
        if !h.is_none() {
            self.body.set_coords(0., h.height());
        }
        let c = self
            .body
            .create_node(transform);
        if c.is_none() && h.is_none() {
            return c;
        }
        RenderNode::Container(
            scene::Instruction::new(
                transform,
                Rectangle::empty(c.width().max(h.width()), h.height() + c.height()),
            ),
            vec![h, c],
        )
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<D>, event: Event<'d>) -> Damage {
        match event {
            Event::Configure(state) => {
                let mut activated = false;
                let mut positioned = false;
                for state in state.iter().rev() {
                    match state {
                        WindowState::Activated => {
                            if self.alternate.is_some() {
                                activated = true;
                                if !self.activated {
                                    self.body.set_border_texture(self.background.clone());
                                    self.header.set_background(self.background.clone());
                                    self.header.set_border_texture(self.background.clone());
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
                            self.body.set_even_radius(0.);
                            self.header.set_even_radius(0.);
                        }
                        _ => {}
                    }
                }
                if !activated {
                    if let Some(ref texture) = self.alternate {
                        self.body.set_border_texture(texture.clone());
                        self.header.set_border_texture(texture.clone());
                        self.header.set_background(texture.clone());
                    }
                }
                if !positioned && self.positioned {
                    self.positioned = false;
                    self.set_radius(self.radius.0, self.radius.1, self.radius.2, self.radius.3);
                }
                self.activated = activated;
                self.positioned = positioned;
                self.header.sync(ctx, event).max(self.body.sync(ctx, event))
            }
            Event::Prepare => {
                if self.set_width(self.width()).is_ok() {
                    self.header.sync(ctx, event).max(self.body.sync(ctx, event))
                } else {
                    Damage::None
                }
            }
            _ => self.header.sync(ctx, event).max(self.body.sync(ctx, event)),
        }
    }
}

impl<H, W> Style for Window<H, W>
where
    H: Style,
    W: Style,
{
    fn set_background<B: Into<scene::Texture>>(&mut self, background: B) {
        self.background = background.into();
        self.header.set_background(self.background.clone());
    }
    fn set_border_texture<T: Into<Texture>>(&mut self, texture: T) {
        let texture = texture.into();
        self.header.set_border_texture(texture.clone());
        self.body.set_border_texture(texture);
    }
    fn set_border_size(&mut self, size: f32) {
        self.body.set_border_size(size);
    }
    fn set_even_radius(&mut self, radius: f32) {
        self.radius = (radius, radius, radius, radius);
        self.body.set_radius(0., 0., radius, radius);
        self.header.set_radius(radius, radius, 0., 0.);
    }
    fn set_radius(&mut self, tl: f32, tr: f32, br: f32, bl: f32) {
        self.radius = (tl, tr, br, bl);
        self.body.set_radius(0., 0., br, bl);
        self.header.set_radius(tl, tr, 0., 0.);
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
    fn set_width(&mut self, width: f32) -> Result<(), f32> {
        self.widget.set_width(width)
    }
    fn height(&self) -> f32 {
        self.widget.height()
    }
    fn set_height(&mut self, height: f32) -> Result<(), f32> {
        self.widget.set_height(height)
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
                    match p {
                        Pointer::MouseClick {
                            button,
                            pressed,
                            serial,
                        } => {
                            if button.is_left() && pressed {
                                ctx.window_request(WindowRequest::Move(serial));
                            } else if button.is_right() && pressed {
                                ctx.window_request(WindowRequest::Menu(x, y, serial));
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        self.widget.sync(ctx, event)
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
    let header = Header {
        widget: headerbar(header)
            .ext()
            .background(style::BG2)
            .even_padding(10.),
    };

    Window {
        header,
        activated: false,
        positioned: false,
        body: widget.child(),
        radius: (0., 0., 0., 0.),
        background: style::BG2.into(),
        alternate: None,
    }
}
