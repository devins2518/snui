use crate::controller::*;
use crate::*;
use crate::{
    scene::{Coords, Instruction, Region},
    widgets::{container::*, shapes::*, text::Listener, *},
    *,
};

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
        24.
    }
    fn width(&self) -> f32 {
        24.
    }
}

impl<M: TryFrom<WindowMessage>> Widget<M> for Close {
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        let mut canvas = self.create_canvas(x, y);

        let (w, h) = (3., self.width());

        let r = Rectangle::empty(w, h).background(style::RED);

        let (x, y) = ((self.width() - w) / 2., (self.height() - h) / 2.);

        canvas.draw_at_angle(x, y, r.clone(), 45.);
        canvas.draw_at_angle(x, y, r, -45.);

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
                        if let Ok(msg) = WindowMessage::Close.try_into() {
                            let _ = ctx.send(msg);
                        }
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
        24.
    }
    fn width(&self) -> f32 {
        24.
    }
}

impl<M: TryFrom<WindowMessage>> Widget<M> for Maximize {
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        Instruction::new(
            x,
            y,
            Rectangle::new(
                self.width(),
                self.height(),
                ShapeStyle::Border(u32_to_source(style::BLU), 3.),
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
                        if let Ok(msg) = WindowMessage::Maximize.try_into() {
                            let _ = ctx.send(msg);
                        }
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
        24.
    }
    fn width(&self) -> f32 {
        24.
    }
}

impl<M: TryFrom<WindowMessage>> Widget<M> for Minimize {
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        Instruction::new(
            x,
            y,
            Rectangle::empty(self.width(), 3.).background(style::YEL),
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
                        if let Ok(msg) = WindowMessage::Minimize.try_into() {
                            let _ = ctx.send(msg);
                        }
                    }
                }
            }
        }
        Damage::None
    }
}

fn wm_button<M>() -> impl Widget<M>
where
    M: TryFrom<WindowMessage> + 'static,
{
    let mut l = WidgetLayout::new(5.);
    l.add(Minimize {});
    l.add(Maximize {});
    l.add(Close {});
    l.justify(CENTER);
    l
}

fn headerbar<M>(listener: Listener<M>) -> impl Widget<M>
where
    M: TryFrom<WindowMessage> + TryInto<String> + PartialEq + 'static,
{
    let mut l = LayoutBox::new();
    l.add(listener.clamp());
    l.add(wm_button());
    l.button(|_, ctx, p| match p {
        Pointer::MouseClick {
            button, pressed, ..
        } => {
            if button.is_left() && pressed {
                if let Ok(msg) = WindowMessage::Move.try_into() {
                    // Move is sent first and if a button sends another message,
                    // the controller shall overwrite this message
                    let _ = ctx.send(msg);
                }
            }
        }
        _ => {}
    })
}

pub struct Window<M, H, W>
where
    M: TryFrom<WindowMessage> + TryInto<String> + PartialEq,
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
    M: TryFrom<WindowMessage> + TryInto<String> + PartialEq,
    H: Widget<M>,
    W: Widget<M>,
{
    fn width(&self) -> f32 {
        self.main.width()
    }
    fn height(&self) -> f32 {
        self.main.height()
    }
    fn set_width(&mut self, width: f32) -> Result<(), f32> {
        if let Err(width) = self.header.set_width(width) {
            self.main.set_width(width)
        } else {
            self.main.set_width(width)
        }
    }
}

impl<M, H, W> Widget<M> for Window<M, H, W>
where
    M: TryFrom<WindowMessage> + TryInto<String> + PartialEq,
    H: Widget<M>,
    W: Widget<M>,
{
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        let h = self.header.create_node(x, y);
        self.coords.y = h.height();
        let c = self.main.create_node(x, y + self.coords.y);
        RenderNode::Container {
            region: Region::new(x, y, c.width(), h.height() + c.height()),
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

pub fn default_window<M, W>(listener: Listener<M>, widget: W) -> impl Widget<M>
where
    M: TryFrom<WindowMessage> + TryInto<String> + PartialEq + 'static,
    W: Widget<M>,
{
    let header = headerbar(listener)
        .ext()
        .background(style::BG0)
        .even_padding(4.)
        .radius(4., 4., 0., 0.);

    Window {
        header,
        coords: Coords::default(),
        main: widget,
    }
}
