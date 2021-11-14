pub mod rectangle;
// pub mod button;

use crate::scene::*;
use crate::*;
use raqote::*;
pub use rectangle::Rectangle;
// pub use button::Button;
use std::ops::{Deref, DerefMut};

pub trait Shape {
    fn radius(self, radius: f32) -> Self;
    fn background(self, color: u32) -> Self;
    fn border_width(self, width: f32) -> Self;
    fn border_color(self, color: u32) -> Self;
    fn border(self, color: u32, width: f32) -> Self;
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Style {
    Fill(SolidSource),
    Border(SolidSource, f32),
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

impl <W: Widget + Shape> WidgetExt<W> {
    pub fn radius(self, radius: f32) -> Self {
        let background = if let Some(mut rect) = self.background {
            rect.set_size(self.width(), self.height()).unwrap();
            Some(rect.radius(radius))
        } else {
            None
        };
        let border = if let Some(mut rect) = self.border {
            rect.set_size(self.width(), self.height()).unwrap();
            Some(rect.radius(radius))
        } else {
            None
        };
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
        let background = if let Some(mut rect) = self.background {
            rect.set_size(self.width(), self.height()).unwrap();
            Some(rect.radius(radius))
        } else {
            None
        };
        let border = if let Some(mut rect) = self.border {
            rect.set_size(self.width(), self.height()).unwrap();
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
        let bg = if let Some(mut rect) = self.background {
            rect.set_size(self.width(), self.height()).unwrap();
            rect.background(color)
        } else {
            Rectangle {
                width: self.width(),
                height: self.height(),
                style: Style::fill(color),
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
    fn border(self, color: u32, width: f32) -> Self {
        let border = if let Some(mut rect) = self.border {
            rect.set_size(self.width(), self.height()).unwrap();
            rect.border(color, width)
        } else {
            Rectangle {
                width: self.width(),
                height: self.height(),
                style: Style::border(color, width),
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
    fn border_width(self, width: f32) -> Self {
        let border = if let Some(mut rect) = self.border {
            rect.set_size(self.width(), self.height()).unwrap();
            rect.border_width(width)
        } else {
            Rectangle {
                width: self.width(),
                height: self.height(),
                style: Style::border(FG, width),
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
        let border = if let Some(mut rect) = self.border {
            rect.set_size(self.width(), self.height()).unwrap();
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
                    Instruction::new(0., 0., self.background.unwrap())
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
                    Instruction::new(x, y, self.border.unwrap())
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
                            Instruction::new(0., 0., self.border.unwrap())
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
                    Instruction::new(x, y, self.background.unwrap())
                },
            }
        }
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext, event: Event) {
        self.child.sync(ctx, event);
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
