use crate::controller::*;
use crate::*;
use crate::{
    scene::{Coords, Instruction, Region},
    widgets::{container::*, shapes::*, text::Listener, *},
};

#[derive(Clone, PartialEq, Debug)]
pub enum WindowMessage {
    Move,
    Close,
    Maximize,
    Minimize,
    Title(String),
}

// This is essentially the close button
struct Close {}

impl Geometry for Close {
    fn height(&self) -> f32 {
        15.
    }
    fn width(&self) -> f32 {
        15.
    }
}

impl<M> Widget<M> for Close {
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        let mut canvas = self.create_canvas(x, y);

        use std::f32::consts::FRAC_1_SQRT_2;

        let width = self.width() * FRAC_1_SQRT_2;
        let height = self.height() * FRAC_1_SQRT_2;

        let r = Rectangle::empty(
            width, height
        ).background(style::RED);

        canvas.draw_at_angle((self.width() - width) / 2., (self.height() - height) / 2., r, -45.);

        canvas.finish()
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<M>, event: Event<'d, M>) -> Damage {
        if let Event::Pointer(x, y, p) = event {
            if let Pointer::MouseClick {
                time: _,
                button,
                pressed,
            } = p
            {
                if self.contains(x, y) {
                    if button.is_left() && pressed {
                        ctx.window_state(WindowMessage::Close);
                    }
                }
            }
        }
        Damage::None
    }
}

// This is essentially the close button
struct Maximize {}

impl Geometry for Maximize {
    fn height(&self) -> f32 {
        15.
    }
    fn width(&self) -> f32 {
        15.
    }
}

impl<M> Widget<M> for Maximize {
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        let thickness = 2.;
        Instruction::new(
            x,
            y,
            Rectangle::new(
                self.width() - 2. * thickness,
                self.height() - 2. * thickness,
                ShapeStyle::Border(u32_to_source(style::BLU), thickness),
            ),
        )
        .into()
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<M>, event: Event<'d, M>) -> Damage {
        if let Event::Pointer(x, y, p) = event {
            if let Pointer::MouseClick {
                time: _,
                button,
                pressed,
            } = p
            {
                if self.contains(x, y) {
                    if button.is_left() && pressed {
                        ctx.window_state(WindowMessage::Maximize);
                    }
                }
            }
        }
        Damage::None
    }
}

// This is essentially the close button
struct Minimize {}

impl Geometry for Minimize {
    fn height(&self) -> f32 {
        15.
    }
    fn width(&self) -> f32 {
        15.
    }
}

impl<M> Widget<M> for Minimize {
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        let thickness = 3.;
        Instruction::new(
            x,
            y + (self.height() - thickness) / 2.,
            Rectangle::empty(self.width(), thickness).background(style::YEL),
        )
        .into()
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<M>, event: Event<'d, M>) -> Damage {
        if let Event::Pointer(x, y, p) = event {
            if let Pointer::MouseClick {
                time: _,
                button,
                pressed,
            } = p
            {
                if self.contains(x, y) {
                    if button.is_left() && pressed {
                        ctx.window_state(WindowMessage::Minimize);
                    }
                }
            }
        }
        Damage::None
    }
}

fn wm_button<M>() -> impl Widget<M>
where
    M: 'static,
{
    let mut l = WidgetLayout::new(15.);
    l.add(Minimize {});
    l.add(Maximize {});
    l.add(Close {});
    l.justify(CENTER);
    l
}

fn headerbar<M>(listener: Listener<M>) -> impl Widget<M>
where
    M: TryInto<String> + 'static,
{
    let mut l = LayoutBox::new();
    l.add(listener.clamp().anchor(START, CENTER));
    l.add(wm_button().clamp().anchor(END, CENTER));
    l.button(|_, ctx, p| match p {
        Pointer::MouseClick {
            button, pressed, ..
        } => {
            if button.is_left() && pressed {
                ctx.window_state(WindowMessage::Move);
            }
        }
        _ => {}
    })
}

pub struct Window<M, H, W>
where
    M: TryInto<String>,
    H: Widget<M>,
    W: Widget<M>,
{
    header: WidgetExt<M, H>,
    // The position of the window
    coords: Coords,
    main: W,
}

impl<M, H, W> Geometry for Window<M, H, W>
where
    M: TryInto<String>,
    H: Widget<M>,
    W: Widget<M>,
{
    fn width(&self) -> f32 {
        self.header.width()
    }
    fn height(&self) -> f32 {
        self.main.height() + self.header.height()
    }
    fn set_width(&mut self, width: f32) -> Result<(), f32> {
        if let Err(width) = self.main.set_width(width) {
            self.header.set_width(width)
        } else {
            self.header.set_width(width)
        }
    }
    fn set_height(&mut self, height: f32) -> Result<(), f32> {
        self.main.set_height(height - self.header.height())
    }
}

impl<M, H, W> Widget<M> for Window<M, H, W>
where
    M: TryInto<String>,
    H: Widget<M>,
    W: Widget<M>,
{
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        self.set_width(self.width());
        let h = self.header.create_node(x, y);
        if !h.is_none() {
            self.coords.y = h.height();
        }
        let c = self.main.create_node(x, y + self.coords.y);
        if c.is_none() && h.is_none() {
            return c;
        }
        RenderNode::Container {
            region: Region::new(x, y, c.width().max(h.width()), self.coords.y + c.height()),
            nodes: vec![h, c],
        }
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<M>, event: Event<'d, M>) -> Damage {
        match event {
            Event::Pointer(x, y, p) => self
                .header
                .sync(ctx, event)
                .max(self.main.sync(ctx, Event::Pointer(x, y - self.coords.y, p))),
            _ => self.header.sync(ctx, event).max(self.main.sync(ctx, event)),
        }
    }
}

impl<M, H, W> Style for Window<M, H, W>
where
    M: TryInto<String>,
    H: Widget<M>,
    W: Widget<M> + Style,
{
    fn set_background<B: Into<scene::Background>>(&mut self, background: B) {
        self.header.set_background(background);
    }
    fn set_border(&mut self, color: u32, width: f32) {
        self.main.set_border(color, width);
    }
    fn set_border_color(&mut self, color: u32) {
        self.main.set_border_color(color);
    }
    fn set_border_size(&mut self, size: f32) {
        self.main.set_border_size(size);
    }
    fn set_even_radius(&mut self, radius: f32) {
        self.main.set_radius(0., 0., radius, radius);
        self.header.set_radius(radius, radius, 0., 0.);
    }
    fn set_radius(&mut self, tl: f32, tr: f32, br: f32, bl: f32) {
        self.main.set_radius(0., 0., br, bl);
        self.header.set_radius(tl, tr, 0., 0.);
    }
}

pub fn default_window<M, W>(listener: Listener<M>, widget: W) -> Window<M, impl Widget<M>, W>
where
    M: TryInto<String> + 'static,
    W: Widget<M>,
{
    let header = headerbar(listener)
        .ext()
        .background(style::BG0)
        .even_padding(10.);

    Window {
        header,
        coords: Coords::default(),
        main: widget,
    }
}
