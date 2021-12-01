pub mod rectangle;
// pub mod button;

use crate::scene::*;
use crate::*;
pub use rectangle::Rectangle;
use std::ops::{Deref, DerefMut};
use tiny_skia::*;

pub trait Shape {
    fn radius(self, radius: f32) -> Self;
    fn background(self, background: Background) -> Self;
    fn border_width(self, width: f32) -> Self;
    fn border_color(self, color: u32) -> Self;
    fn border(self, color: u32, width: f32) -> Self;
    fn set_radius(&mut self, radius: f32);
    fn set_background(&mut self, background: Background);
    fn set_border_width(&mut self, width: f32);
    fn set_border_color(&mut self, color: u32);
    fn set_border(&mut self, color: u32, width: f32);
}

#[derive(Clone, PartialEq, Debug)]
pub enum Style {
    Background(Background),
    Border(Color, f32),
}

impl From<u32> for Style {
    fn from(color: u32) -> Self {
        Style::Background(color.into())
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
        let delta = (radius - 2_f32.sqrt().powi(-1) * radius).round();
        let width = self.width() + delta;
        let height = self.height() + delta;
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
        let padding = [
            self.padding[0] + delta,
            self.padding[1] + delta,
            self.padding[2] + delta,
            self.padding[3] + delta,
        ];
        Self {
            border,
            background,
            child: self
                .child
                .radius((radius * ratio).round() - border_width - self.padding[0] - delta),
            padding,
        }
    }
}

impl<W: Widget> Shape for WidgetExt<W> {
    fn set_radius(&mut self, radius: f32) {
        if let Some(rect) = self.background.as_mut() {
            rect.set_radius(radius)
        }
        if let Some(rect) = self.border.as_mut() {
            rect.set_radius(radius);
        }
    }
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
    fn set_background(&mut self, background: Background) {
        if let Some(rect) = self.background.as_mut() {
            rect.set_background(background);
        } else {
            self.background = Some(Rectangle {
                width: self.width(),
                height: self.height(),
                style: Style::Background(background),
                radius: (0., 0., 0., 0.),
            });
        }
    }
    fn background(self, background: Background) -> Self {
        let width = self.width();
        let height = self.height();
        let bg = if let Some(mut rect) = self.background {
            rect.set_size(width, height).unwrap();
            rect.background(background)
        } else {
            Rectangle {
                width: self.width(),
                height: self.height(),
                style: Style::Background(background),
                radius: (0., 0., 0., 0.),
            }
        };
        Self {
            background: Some(bg),
            border: self.border,
            child: self.child,
            padding: self.padding,
        }
    }
    fn set_border(&mut self, color: u32, width: f32) {
        if let Some(rect) = self.border.as_mut() {
            rect.set_border(color, width);
        } else {
            self.border = Some(Rectangle {
                width: self.width(),
                height: self.height(),
                style: Style::border(color, width),
                radius: if let Some(background) = &self.background {
                    background.radius
                } else {
                    (0., 0., 0., 0.)
                },
            });
        };
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
                radius: if let Some(background) = &self.background {
                    background.radius
                } else {
                    (0., 0., 0., 0.)
                },
            }
        };
        Self {
            border: Some(border),
            background: self.background,
            child: self.child,
            padding: self.padding,
        }
    }
    fn set_border_width(&mut self, width: f32) {
        if let Some(rect) = self.border.as_mut() {
            rect.set_border_width(width);
        } else {
            self.border = Some(Rectangle {
                width: self.width(),
                height: self.height(),
                style: Style::border(FG, width),
                radius: (0., 0., 0., 0.),
            });
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
                width,
                height,
                style: Style::border(FG, size),
                radius: (0., 0., 0., 0.),
            }
        };
        Self {
            border: Some(border),
            background: self.background,
            child: self.child,
            padding: self.padding,
        }
    }
    fn set_border_color(&mut self, color: u32) {
        if let Some(rect) = self.border.as_mut() {
            rect.set_border_color(color)
        } else {
            self.border = Some(Rectangle {
                width: self.width(),
                height: self.height(),
                style: Style::border(color, 0.),
                radius: (0., 0., 0., 0.),
            })
        };
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
                radius: (0., 0., 0., 0.),
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
    fn set_width(&mut self, width: f32) -> Result<(), f32> {
        if let Some(background) = self.background.as_mut() {
            background.set_width(width)?;
        }
        if let Some(border) = self.border.as_mut() {
            border.set_width(width)?;
        }
        self.child
            .set_width(width - self.padding[1] - self.padding[3])?;
        Ok(())
    }
    fn set_height(&mut self, height: f32) -> Result<(), f32> {
        if let Some(background) = self.background.as_mut() {
            background.set_height(height)?;
        }
        if let Some(border) = self.border.as_mut() {
            border.set_height(height)?;
        }
        self.child
            .set_height(height - self.padding[0] - self.padding[2])?;
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
                node: Box::new((
                    self.child
                        .create_node(x + self.padding[3] + border, y + self.padding[0] + border),
                    RenderNode::empty(x, y, self.width(), self.height()),
                )),
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
                node: Box::new((
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
                )),
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
            x -= border + self.padding[3];
            y -= border + self.padding[0];
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
