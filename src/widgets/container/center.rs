use crate::widgets::*;
use crate::*;
use scene::{Region};
use std::ops::{Deref, DerefMut};

pub struct Centerbox<F: Widget, S: Widget, L: Widget> {
    width: f32,
    height: f32,
    orientation: Orientation,
    childs: (WidgetBox<F>, WidgetBox<S>, WidgetBox<L>),
}

impl<F: Widget, S: Widget, L: Widget> Centerbox<F, S, L> {
    pub fn horizontal(a: F, b: S, c: L) -> Self {
        Self {
            width: a.width() + b.width() + c.width(),
            height: a.height().max(b.height()).max(c.height()),
            orientation: Orientation::Horizontal,
            childs: (a.into_box(), b.into_box(), c.into_box()),
        }
    }
    pub fn vertical(a: F, b: S, c: L) -> Self {
        Self {
            height: a.height() + b.height() + c.height(),
            width: a.width().max(b.width()).max(c.width()),
            orientation: Orientation::Vertical,
            childs: (a.into_box(), b.into_box(), c.into_box()),
        }
    }
    pub fn align(mut self) -> Self {
        match &self.orientation {
            Orientation::Horizontal => {
                self.childs
                    .0
                    .set_anchor(Alignment::Start, Alignment::Center);
                self.childs
                    .1
                    .set_anchor(Alignment::Center, Alignment::Center);
                self.childs.2.set_anchor(Alignment::End, Alignment::Center);
            }
            Orientation::Vertical => {
                self.childs
                    .0
                    .set_anchor(Alignment::Center, Alignment::Start);
                self.childs
                    .1
                    .set_anchor(Alignment::Center, Alignment::Center);
                self.childs.2.set_anchor(Alignment::Center, Alignment::End);
            }
        }
        self
    }
}

impl<F: Widget, S: Widget, L: Widget> Geometry for Centerbox<F, S, L> {
    fn height(&self) -> f32 {
        self.height
    }
    fn width(&self) -> f32 {
        self.width
    }
    fn set_width(&mut self, width: f32) -> Result<(), f32> {
        self.width = width;
        Ok(())
    }
    fn set_height(&mut self, height: f32) -> Result<(), f32> {
        self.height = height;
        Ok(())
    }
}

impl<F: Widget, S: Widget, L: Widget> Widget for Centerbox<F, S, L> {
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        let mut nodes = Vec::with_capacity(3);
        match self.orientation {
            Orientation::Horizontal => {
                let mut dx = 0.;
                let mut space = self.width;

                let (aw, bw, cw) = (
                    self.childs.0.deref().width().floor(),
                    self.childs.1.deref().width().floor(),
                    self.childs.2.deref().width().floor(),
                );
                let mut real_width = aw + bw + cw;

                let mut order = Vec::new();
                if real_width <= self.width {
                    order.push(aw as u32);
                    order.push(bw as u32);
                    order.push(cw as u32);
                } else if {
                    real_width -= cw;
                    real_width <= self.width
                } {
                    order.push(aw as u32);
                    order.push(bw as u32);
                } else if {
                    real_width -= bw;
                    real_width <= self.width
                } {
                    order.push(aw as u32);
                } else {
                    return RenderNode::None;
                }
                order.sort();

                let delta = (space / order.len() as f32).floor();

                for w in order.iter().rev() {
                    if space >= 0. {
                        if *w == aw as u32 {
                            self.childs
                                .0
                                .set_size(aw.max(delta).min(space), self.height)
                                .unwrap();
                            space -= self.childs.0.width();
                        } else if *w == bw as u32 {
                            self.childs
                                .1
                                .set_size(bw.max(delta).min(space), self.height)
                                .unwrap();
                            space -= self.childs.1.width();
                        } else if *w == cw as u32 {
                            self.childs
                                .2
                                .set_size(cw.max(delta).min(space), self.height)
                                .unwrap();
                            space -= self.childs.2.width();
                        } else {
                            break;
                        }
                    }
                }

                if order.len() > 0 {
                    nodes.push(self.childs.0.create_node(x + dx, y));
                    dx += self.childs.0.width();
                }
                if order.len() > 1 {
                    nodes.push(self.childs.1.create_node(x + dx, y));
                    dx += self.childs.1.width();
                }
                if order.len() > 2 {
                    nodes.push(self.childs.2.create_node(x + dx, y));
                }
            }
            Orientation::Vertical => {
                let mut dy = 0.;
                let mut space = self.height;

                let (ah, bh, ch) = (
                    self.childs.0.deref().height().floor(),
                    self.childs.1.deref().height().floor(),
                    self.childs.2.deref().height().floor(),
                );
                let mut real_height = ah + bh + ch;

                let mut order = Vec::new();
                if real_height <= self.height {
                    order.push(ah as u32);
                    order.push(bh as u32);
                    order.push(ch as u32);
                } else if {
                    real_height -= bh;
                    real_height <= self.height
                } {
                    order.push(ah as u32);
                    order.push(bh as u32);
                } else if {
                    real_height -= ch;
                    real_height <= self.height
                } {
                    order.push(ah as u32);
                } else {
                    return RenderNode::None;
                }
                order.sort();

                let delta = (space / order.len() as f32).floor();

                for h in order.iter().rev() {
                    if space >= 0. {
                        if *h == ah as u32 {
                            self.childs
                                .0
                                .set_size(self.width, ah.max(delta).min(space))
                                .unwrap();
                            space -= self.childs.0.height();
                        } else if *h == bh as u32 {
                            self.childs
                                .1
                                .set_size(self.width, bh.max(delta).min(space))
                                .unwrap();
                            space -= self.childs.1.height();
                        } else if *h == ch as u32 {
                            self.childs
                                .2
                                .set_size(self.width, ch.max(delta).min(space))
                                .unwrap();
                            space -= self.childs.2.height();
                        } else {
                            break;
                        }
                    }
                }

                if order.len() > 0 {
                    nodes.push(self.childs.0.create_node(x, y + dy));
                    dy += self.childs.0.height();
                }
                if order.len() > 1 {
                    nodes.push(self.childs.1.create_node(x, y + dy));
                    dy += self.childs.1.height();
                }
                if order.len() > 2 {
                    nodes.push(self.childs.2.create_node(x, y + dy));
                }
            }
        }
        RenderNode::Container {
            region: Region::new(x, y, self.width(), self.height()),
            nodes,
        }
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext, event: Event) {
        match event {
            Event::Pointer(mut x, mut y, p) => match &self.orientation {
                Orientation::Horizontal => {
                    self.childs.0.sync(ctx, Event::Pointer(x, y, p));
                    x -= self.childs.0.width();

                    self.childs.1.sync(ctx, Event::Pointer(x, y, p));
                    x -= self.childs.1.width();

                    self.childs.2.sync(ctx, Event::Pointer(x, y, p));
                }
                Orientation::Vertical => {
                    self.childs.0.sync(ctx, Event::Pointer(x, y, p));
                    y -= self.childs.0.height();

                    self.childs.1.sync(ctx, Event::Pointer(x, y, p));
                    y -= self.childs.1.height();

                    self.childs.2.sync(ctx, Event::Pointer(x, y, p));
                }
            },
            Event::Commit => {
                self.childs.0.sync(ctx, event);
                self.childs.1.sync(ctx, event);
                self.childs.2.sync(ctx, event);
                match &self.orientation {
                    Orientation::Horizontal => {
                        self.width = self.width.max(
                            self.childs
                                .0
                                .deref()
                                .width()
                                .max(self.childs.1.deref().width())
                                .max(self.childs.2.deref().width())
                                * 3.,
                        );
                        self.height = self
                            .childs
                            .0
                            .deref()
                            .height()
                            .max(self.childs.1.deref().height())
                            .max(self.childs.2.deref().height());
                    }
                    Orientation::Vertical => {
                        self.width = self
                            .childs
                            .0
                            .deref()
                            .width()
                            .max(self.childs.1.deref().width())
                            .max(self.childs.2.deref().width());
                        self.height = self.height.max(
                            self.childs
                                .0
                                .deref()
                                .height()
                                .max(self.childs.1.deref().height())
                                .max(self.childs.2.deref().height())
                                * 3.,
                        );
                    }
                }
            }
            _ => {
                self.childs.0.sync(ctx, event);
                self.childs.1.sync(ctx, event);
                self.childs.2.sync(ctx, event);
            }
        }
    }
}

impl<'w, F: Widget, S: Widget, L: Widget> Deref for Centerbox<F, S, L> {
    type Target = (WidgetBox<F>, WidgetBox<S>, WidgetBox<L>);
    fn deref(&self) -> &Self::Target {
        &self.childs
    }
}

impl<'w, F: Widget, S: Widget, L: Widget> DerefMut for Centerbox<F, S, L> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.childs
    }
}
