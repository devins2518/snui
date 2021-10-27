use crate::context::DamageType;
use crate::widgets::primitives::WidgetShell;
use crate::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Anchor {
    Left,
    Right,
    Top,
    Bottom,
    Center,
    TopLeft,
    TopRight,
    BottomRight,
    BottomLeft,
}

pub struct Wbox {
    width: f32,
    height: f32,
    pub widgets: Vec<Inner>,
}

pub struct Inner {
    x: f32,
    y: f32,
    mapped: bool,
    anchor: Anchor,
    widget: Box<dyn Widget>,
}

impl Geometry for Inner {
    fn width(&self) -> f32 {
        self.widget.as_ref().width()
    }
    fn height(&self) -> f32 {
        self.widget.as_ref().height()
    }
}

impl Drawable for Inner {
    fn set_color(&mut self, color: u32) {
        self.widget.set_color(color)
    }
    fn draw(&self, ctx: &mut Context, x: f32, y: f32) {
        self.widget.draw(ctx, x, y);
    }
}

impl Widget for Inner {
    fn roundtrip<'d>(&'d mut self, wx: f32, wy: f32, ctx: &mut Context, dispatch: &Dispatch) {
        self.widget.roundtrip(wx, wy, ctx, dispatch)
    }
}

impl Inner {
    pub fn new(widget: impl Widget + 'static) -> Inner {
        Inner {
            x: 0.,
            y: 0.,
            mapped: true,
            anchor: Anchor::TopLeft,
            widget: Box::new(widget),
        }
    }
    pub fn new_at(widget: impl Widget + 'static, anchor: Anchor, x: f32, y: f32) -> Inner {
        Inner {
            x,
            y,
            anchor,
            mapped: true,
            widget: Box::new(widget),
        }
    }
    pub fn anchor(&self) -> Anchor {
        self.anchor
    }
    pub fn is_mapped(&self) -> bool {
        self.mapped
    }
    pub fn map(&mut self) {
        self.mapped = true;
    }
    pub fn unmap(&mut self) {
        self.mapped = false;
    }
    pub fn coords(&self) -> (f32, f32) {
        (self.x, self.y)
    }
    pub fn child(&self) -> &dyn Widget {
        self.widget.as_ref()
    }
    pub fn location(&self, width: f32, height: f32) -> Result<(f32, f32), Error> {
        let widwidth = self.width();
        let widheight = self.height();
        match self.anchor {
            Anchor::Left => {
                if height >= widheight {
                    return Ok((self.x, (height - widheight + self.y) / 2.));
                }
            }
            Anchor::Right => {
                if height >= widheight && width >= widheight {
                    return Ok((
                        width - widwidth - self.x,
                        (height - widheight + self.y) / 2.,
                    ));
                }
            }
            Anchor::Top => {
                if width >= widwidth {
                    return Ok(((width - widwidth + self.x) / 2., self.y));
                }
            }
            Anchor::Bottom => {
                if height > self.y + widheight {
                    return Ok((
                        (width - widwidth + self.x) / 2.,
                        height - self.y - widheight,
                    ));
                }
            }
            Anchor::Center => {
                return Ok((
                    if width >= widwidth {
                        (width - widwidth + self.x) / 2.
                    } else {
                        0.
                    },
                    if height >= widheight {
                        (height - widheight + self.y) / 2.
                    } else {
                        0.
                    },
                ))
            }
            Anchor::TopRight => {
                if width > self.x + widwidth {
                    return Ok((width - self.x - widwidth, self.y));
                }
            }
            Anchor::TopLeft => return Ok((self.x, self.y)),
            Anchor::BottomRight => {
                if width > self.x + widwidth && height > self.y + widheight {
                    return Ok((width - self.x - widwidth, height - self.y - widheight));
                }
            }
            Anchor::BottomLeft => {
                if height > self.y + widheight {
                    return Ok((self.x, height - self.y - widheight));
                }
            }
        }
        Err(Error::Dimension("wbox", widwidth, widheight))
    }
    pub fn set_anchor(&mut self, anchor: Anchor) {
        self.anchor = anchor;
    }
    pub fn set_location(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }
    pub fn translate(&mut self, x: f32, y: f32) {
        self.x += x;
        self.y += y;
    }
}

impl Geometry for Wbox {
    fn width(&self) -> f32 {
        self.width
    }
    fn height(&self) -> f32 {
        self.height
    }
}

impl Drawable for Wbox {
    fn set_color(&mut self, _color: u32) {}
    fn draw(&self, ctx: &mut Context, x: f32, y: f32) {
        let sw = self.width();
        let sh = self.height();
        for w in &self.widgets {
            match w.location(sw, sh) {
                Ok((dx, dy)) => {
                    if w.is_mapped() && dx <= sw && dy <= sh {
                        w.draw(ctx, x + dx, y + dy)
                    }
                }
                Err(e) => e.debug(),
            }
        }
    }
}

impl Container for Wbox {
    fn len(&self) -> usize {
        self.widgets.len()
    }
    fn add(&mut self, _widget: impl Widget + 'static) -> Result<(), Error> {
        Err(Error::Message("add is not valid on \"wbox\""))
    }
}

impl Wbox {
    pub fn new(width: u32, height: u32) -> WidgetShell<Self> {
        WidgetShell::default(Wbox {
            width: width as f32,
            height: height as f32,
            widgets: Vec::new(),
        })
    }

    pub fn anchor(&mut self, widget: impl Widget + 'static, anchor: Anchor, x: u32, y: u32) {
        self.widgets
            .push(Inner::new_at(widget, anchor, x as f32, y as f32));
    }

    pub fn unmap(&mut self, i: usize) {
        if i < self.widgets.len() {
            self.widgets[i].unmap();
        }
    }
    pub fn unmap_all(&mut self) {
        for w in &mut self.widgets {
            w.unmap();
        }
    }
    pub fn map(&mut self, i: usize) {
        if i < self.widgets.len() {
            self.widgets[i].map();
        }
    }
    pub fn map_all(&mut self) {
        for w in &mut self.widgets {
            w.map();
        }
    }
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width as f32;
        self.height = height as f32;
    }
}

impl Widget for Wbox {
    fn roundtrip<'d>(&'d mut self, wx: f32, wy: f32, ctx: &mut Context, dispatch: &Dispatch) {
        let width = self.width();
        let height = self.height();
        let mut draw = false;
        for l in &mut self.widgets {
            if let Ok((dx, dy)) = l.location(width, height) {
                l.roundtrip(wx + dx, wy + dy, ctx, dispatch);
                if let DamageType::Resize = ctx.damage_type() {
                    draw = true;
                }
            }
        }
        if draw {
            self.draw(ctx, wx, wy);
            ctx.partial_damage();
        }
    }
}
