pub mod rectangle;
// pub mod button;

use crate::scene::*;
use crate::*;
use raqote::*;
pub use rectangle::Rectangle;
use std::sync::Arc;
// pub use button::Button;
use std::ops::{Deref, DerefMut};

pub trait Shape {
    fn radius(self, radius: f32) -> Self;
    fn background(self, color: u32) -> Self;
    fn border_width(self, width: f32) -> Self;
    fn border_color(self, color: u32) -> Self;
    fn border(self, color: u32, width: f32) -> Self;
}

#[derive(Clone)]
pub enum Style {
    Solid(SolidSource),
    Border(SolidSource, f32),
    LinearGradient(Arc<Gradient>, Spread),
    RadialGradient(Arc<Gradient>, Spread, f32),
}

impl PartialEq for Style {
    fn eq(&self, other: &Self) -> bool {
        match &self {
            Self::Solid(s) => {
                if let Self::Solid(o) = other {
                    return s == o;
                }
            }
            Self::Border(s, b) => {
                if let Self::Border(o, ob) = other {
                    return s == o && b == ob;
                }
            }
            Self::LinearGradient(sg, s) => {
                if let Self::LinearGradient(og, os) = other {
                    return match s {
                        Spread::Pad => {
                            if let Spread::Pad = os {
                                true
                            } else {
                                false
                            }
                        }
                        Spread::Reflect => {
                            if let Spread::Reflect = os {
                                true
                            } else {
                                false
                            }
                        }
                        Spread::Repeat => {
                            if let Spread::Repeat = os {
                                true
                            } else {
                                false
                            }
                        }
                    } && Arc::as_ptr(sg) == Arc::as_ptr(og);
                }
            }
            Self::RadialGradient(sg, s, r) => {
                if let Self::RadialGradient(og, os, or) = other {
                    return match s {
                        Spread::Pad => {
                            if let Spread::Pad = os {
                                true
                            } else {
                                false
                            }
                        }
                        Spread::Repeat => {
                            if let Spread::Repeat = os {
                                true
                            } else {
                                false
                            }
                        }
                        Spread::Reflect => {
                            if let Spread::Reflect = os {
                                true
                            } else {
                                false
                            }
                        }
                    } && r == or
                        && Arc::as_ptr(sg) == Arc::as_ptr(og);
                }
            }
        }
        false
    }
    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl std::fmt::Debug for Style {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Solid(s) => f.debug_tuple("Solid").field(s).finish(),
            Self::Border(s, b) => f.debug_tuple("Border").field(s).field(b).finish(),
            Self::LinearGradient(g, s) => f
                .debug_tuple("LinearGradient")
                .field(g)
                .field(&match s {
                    Spread::Pad => "Pad",
                    Spread::Reflect => "Reflect",
                    Spread::Repeat => "Repeat",
                })
                .finish(),
            Self::RadialGradient(g, s, r) => f
                .debug_tuple("RadialGradient")
                .field(g)
                .field(r)
                .field(&match s {
                    Spread::Pad => "Pad",
                    Spread::Reflect => "Reflect",
                    Spread::Repeat => "Repeat",
                })
                .finish(),
        }
    }
}

pub struct WidgetExt<W: Widget> {
    child: W,
    padding: [f32; 4],
    border: Option<Rectangle>,
    background: Option<Rectangle>,
}

impl<W: Widget> WidgetExt<W> {
    pub fn default(child: W) -> Self {
        WidgetExt {
            child,
            border: None,
            background: None,
            padding: [0.; 4],
        }
    }
    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = [padding.round(); 4];
        self
    }
    pub fn unwrap(self) -> W {
        self.child
    }
}

impl<W: Widget + Shape> WidgetExt<W> {
    pub fn radius(self, radius: f32) -> Self {
        let width = self.width();
        let height = self.height();
        let ratio = self.child.width() / self.width();
        let border_width = if let Some(rectangle) = &self.border {
            if let Style::Border(_, border) = rectangle.style {
                border
            } else {
                0.
            }
        } else {
            0.
        };
        let background = if let Some(mut rect) = self.background {
            rect.set_size(width, height).unwrap();
            Some(rect.radius(radius))
        } else {
            None
        };
        let border = if let Some(mut rect) = self.border {
            rect.set_size(width, height).unwrap();
            Some(rect.radius(radius))
        } else {
            None
        };
        Self {
            border,
            background,
            child: self.child.radius((radius * ratio).round() - border_width),
            padding: self.padding,
        }
    }
}

impl<W: Widget> Shape for WidgetExt<W> {
    fn radius(self, radius: f32) -> Self {
        let width = self.width();
        let height = self.height();
        let background = if let Some(mut rect) = self.background {
            rect.set_size(width, height).unwrap();
            Some(rect.radius(radius))
        } else {
            None
        };
        let border = if let Some(mut rect) = self.border {
            rect.set_size(width, height).unwrap();
            Some(rect.radius(radius))
        } else {
            None
        };
        Self {
            border,
            background,
            child: self.child,
            padding: self.padding,
        }
    }
    fn background(self, color: u32) -> Self {
        let width = self.width();
        let height = self.height();
        let bg = if let Some(mut rect) = self.background {
            rect.set_size(width, height).unwrap();
            rect.background(color)
        } else {
            Rectangle {
                width: self.width(),
                height: self.height(),
                style: Style::solid(color),
                radius: [0.; 4],
            }
        };
        Self {
            background: Some(bg),
            border: self.border,
            child: self.child,
            padding: self.padding,
        }
    }
    fn border(self, color: u32, size: f32) -> Self {
        let width = self.width();
        let height = self.height();
        let border = if let Some(mut rect) = self.border {
            rect.set_size(width, height).unwrap();
            rect.border(color, size)
        } else {
            Rectangle {
                width: self.width(),
                height: self.height(),
                style: Style::border(color, size),
                radius: [0.; 4],
            }
        };
        Self {
            border: Some(border),
            background: self.background,
            child: self.child,
            padding: self.padding,
        }
    }
    fn border_width(self, size: f32) -> Self {
        let width = self.width();
        let height = self.height();
        let border = if let Some(mut rect) = self.border {
            rect.set_size(width, height).unwrap();
            rect.border_width(size)
        } else {
            Rectangle {
                width: self.width(),
                height: self.height(),
                style: Style::border(FG, size),
                radius: [0.; 4],
            }
        };
        Self {
            border: Some(border),
            background: self.background,
            child: self.child,
            padding: self.padding,
        }
    }
    fn border_color(self, color: u32) -> Self {
        let width = self.width();
        let height = self.height();
        let border = if let Some(mut rect) = self.border {
            rect.set_size(width, height).unwrap();
            rect.border_color(color)
        } else {
            Rectangle {
                width: self.width(),
                height: self.height(),
                style: Style::border(color, 0.),
                radius: [0.; 4],
            }
        };
        Self {
            border: Some(border),
            background: self.background,
            child: self.child,
            padding: self.padding,
        }
    }
}

impl<W: Widget> Geometry for WidgetExt<W> {
    fn set_size(&mut self, width: f32, height: f32) -> Result<(), (f32, f32)> {
        if let Some(background) = self.background.as_mut() {
            background.set_size(width, height)?;
        }
        if let Some(border) = self.border.as_mut() {
            border.set_size(width, height)?;
        }
        self.child.set_size(
            width - self.padding[1] - self.padding[3],
            height - self.padding[0] - self.padding[2],
        )?;
        Ok(())
    }
    fn width(&self) -> f32 {
        let border = if let Some(rectangle) = &self.border {
            if let Style::Border(_, border) = rectangle.style {
                border
            } else {
                0.
            }
        } else {
            0.
        };
        self.child.width() + self.padding[1] + self.padding[3] + 2. * border
    }
    fn height(&self) -> f32 {
        let border = if let Some(rectangle) = &self.border {
            if let Style::Border(_, border) = rectangle.style {
                border
            } else {
                0.
            }
        } else {
            0.
        };
        self.child.height() + self.padding[0] + self.padding[2] + 2. * border
    }
}

impl<W: Widget> Widget for WidgetExt<W> {
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        let border = if let Some(border) = &self.border {
            if let Style::Border(_, size) = border.style {
                size
            } else {
                0.
            }
        } else {
            0.
        };
        if self.background.is_none() && self.border.is_none() {
            self.child
                .create_node(x + self.padding[3] + border, y + self.padding[0] + border)
        } else if self.border.is_none() {
            RenderNode::Extension {
                node: Box::new(
                    self.child
                        .create_node(x + self.padding[3] + border, y + self.padding[0] + border),
                ),
                background: {
                    let width = self.width();
                    let height = self.height();
                    self.background
                        .as_mut()
                        .unwrap()
                        .set_size(width, height)
                        .unwrap();
                    Instruction::new(0., 0., self.background.clone().unwrap())
                },
            }
        } else if self.background.is_none() {
            RenderNode::Container(vec![
                self.child
                    .create_node(x + self.padding[3] + border, y + self.padding[0] + border),
                RenderNode::Instruction({
                    let width = self.width();
                    let height = self.height();
                    self.border
                        .as_mut()
                        .unwrap()
                        .set_size(width, height)
                        .unwrap();
                    Instruction::new(x, y, self.border.clone().unwrap())
                }),
            ])
        } else {
            RenderNode::Extension {
                node: Box::new({
                    RenderNode::Container(vec![
                        self.child.create_node(
                            x + self.padding[3] + border,
                            y + self.padding[0] + border,
                        ),
                        RenderNode::Instruction({
                            let width = self.width();
                            let height = self.height();
                            self.border
                                .as_mut()
                                .unwrap()
                                .set_size(width, height)
                                .unwrap();
                            Instruction::new(x, y, self.border.clone().unwrap())
                        }),
                    ])
                }),
                background: {
                    let width = self.width();
                    let height = self.height();
                    self.background
                        .as_mut()
                        .unwrap()
                        .set_size(width, height)
                        .unwrap();
                    Instruction::new(x, y, self.background.clone().unwrap())
                },
            }
        }
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext, event: Event) {
        if let Event::Pointer(mut x, mut y, p) = event {
            let border = if let Some(border) = &self.border {
                if let Style::Border(_, size) = border.style {
                    size
                } else {
                    0.
                }
            } else {
                0.
            };
            x -= border + self.padding[0] + self.padding[3];
            y -= border + self.padding[1] + self.padding[2];
            self.child.sync(ctx, Event::Pointer(x, y, p))
        } else {
            self.child.sync(ctx, event);
        }
    }
}

impl<W: Widget> Deref for WidgetExt<W> {
    type Target = W;
    fn deref(&self) -> &Self::Target {
        &self.child
    }
}

impl<W: Widget> DerefMut for WidgetExt<W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.child
    }
}
