use crate::widgets::layout::*;
use crate::widgets::shapes::Rectangle;
use crate::widgets::*;
use crate::*;
use scene::Instruction;

pub struct CenterBox<W> {
    orientation: Orientation,
    widgets: [WidgetBox<Positioner<Proxy<W>>>; 3],
}

impl<W: Geometry> Geometry for CenterBox<W> {
    fn set_width(&mut self, width: f32) -> Result<(), f32> {
        let size = (width / self.widgets.len() as f32).ceil();
        match self.orientation {
            Orientation::Horizontal => {
                let mut fixed = Vec::new();
                for i in 0..self.widgets.len() {
                    apply_width(&mut self.widgets, &mut fixed, i, size);
                }
                if fixed.len() == self.widgets.len() {
                    Err(self.width())
                } else {
                    Ok(())
                }
            }
            Orientation::Vertical => return Err(self.width()),
        }
    }
    fn set_height(&mut self, height: f32) -> Result<(), f32> {
        let size = (height / self.widgets.len() as f32).ceil();
        match self.orientation {
            Orientation::Horizontal => {
                let mut fixed = Vec::new();
                for i in 0..self.widgets.len() {
                    apply_height(&mut self.widgets, &mut fixed, i, size);
                }
                if fixed.len() == self.widgets.len() {
                    Err(self.height())
                } else {
                    Ok(())
                }
            }
            Orientation::Vertical => return Err(self.height()),
        }
    }
    fn width(&self) -> f32 {
        let mut width = 0.;
        match self.orientation {
            Orientation::Horizontal => {
                for w in &self.widgets {
                    width += w.width();
                }
            }
            Orientation::Vertical => {
                for w in &self.widgets {
                    width = width.max(w.width());
                }
            }
        }
        width
    }
    fn height(&self) -> f32 {
        let mut height = 0.;
        match self.orientation {
            Orientation::Vertical => {
                for w in &self.widgets {
                    height += w.height();
                }
            }
            Orientation::Horizontal => {
                for w in &self.widgets {
                    height = height.max(w.height());
                }
            }
        }
        height
    }
}

impl<D, W: Widget<D>> Widget<D> for CenterBox<W> {
    fn create_node(&mut self, transform: Transform) -> RenderNode {
        let sw = self.width();
        let sh = self.height();
        let (mut dx, mut dy) = (0., 0.);
        RenderNode::Container(
            Instruction::new(transform, Rectangle::empty(sw, sh)),
            self.widgets
                .iter_mut()
                .map(|widget| {
                    let node;
                    widget.set_coords(dx, dy);
                    match self.orientation {
                        Orientation::Horizontal => {
                            let _ = widget.set_height(sh);
                            node = widget.create_node(transform);
                            dx += widget.width();
                        }
                        Orientation::Vertical => {
                            let _ = widget.set_width(sw);
                            node = widget.create_node(transform);
                            dy += widget.height();
                        }
                    }
                    node
                })
                .collect(),
        )
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<D>, event: Event) -> Damage {
        self.widgets
            .iter_mut()
            .map(|widget| widget.sync(ctx, event))
            .max()
            .unwrap_or_default()
    }
}

impl<W> CenterBox<W> {
    pub fn from(first: W, second: W, third: W) -> Self {
        Self {
            widgets: [
                WidgetBox::new(Positioner::new(Proxy::new(first))).anchor(START, CENTER),
                WidgetBox::new(Positioner::new(Proxy::new(second))).anchor(CENTER, CENTER),
                WidgetBox::new(Positioner::new(Proxy::new(third))).anchor(END, CENTER),
            ],
            orientation: Orientation::Horizontal,
        }
    }
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        match orientation {
            Orientation::Vertical => {
                for (i, anchor) in [START, CENTER, END].iter().enumerate() {
                    self.widgets[i].set_anchor(CENTER, *anchor);
                }
            }
            Orientation::Horizontal => {
                for (i, anchor) in [START, CENTER, END].iter().enumerate() {
                    self.widgets[i].set_anchor(*anchor, CENTER);
                }
            }
        }
        self.orientation = orientation;
        self
    }
}
