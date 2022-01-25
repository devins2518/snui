use crate::widgets::container::*;
use crate::widgets::shapes::Rectangle;
use crate::widgets::*;
use crate::*;
use scene::Instruction;

pub struct CenterBox<M> {
    size: (f32, f32),
    orientation: Orientation,
    widgets: [WidgetBox<M, Child<M>>; 3],
}

impl<M: 'static> FromIterator<Child<M>> for CenterBox<M> {
    fn from_iter<T: IntoIterator<Item = Child<M>>>(iter: T) -> Self {
        let mut centerbox = CenterBox::new();
        let mut i = 0;
        for c in iter {
            if i < 3 {
                centerbox.widgets[i] = c.clamp();
            } else {
                break;
            }
            i += 1;
        }
        centerbox
    }
}

impl<M: 'static> Container<M> for CenterBox<M> {
    fn len(&self) -> usize {
        self.widgets.len()
    }
    fn add(&mut self, widget: impl Widget<M> + 'static) {
        for wbox in self.widgets.iter_mut() {
            if wbox.width() == 0. && wbox.height() == 0. {
                wbox.widget.widget = Box::new(widget);
                break;
            }
        }
    }
    fn remove(&mut self, index: usize) -> Child<M> {
        std::mem::replace(&mut self.widgets[index], Child::new(()).clamp()).widget
    }
}

impl<M> Geometry for CenterBox<M> {
    fn set_width(&mut self, width: f32) -> Result<(), f32> {
        if width != self.size.0 {
            let size = (width / self.widgets.len() as f32).ceil();
            match self.orientation {
                Orientation::Horizontal => {
                    let mut fixed = Vec::new();
                    for i in 0..self.widgets.len() {
                        apply_width(&mut self.widgets, &mut fixed, i, size);
                    }
                }
                Orientation::Vertical => return Err(self.width()),
            }
            self.size.0 = self.width();
        }
        Err(self.size.0)
    }
    fn set_height(&mut self, height: f32) -> Result<(), f32> {
        if height != self.size.1 {
            let size = (height / self.widgets.len() as f32).ceil();
            match self.orientation {
                Orientation::Horizontal => {
                    let mut fixed = Vec::new();
                    for i in 0..self.widgets.len() {
                        apply_height(&mut self.widgets, &mut fixed, i, size);
                    }
                }
                Orientation::Vertical => return Err(self.height()),
            }
            self.size.1 = self.height();
        }
        Err(self.size.1)
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

impl<M> Widget<M> for CenterBox<M> {
    fn create_node(&mut self, transform: Transform) -> RenderNode {
        let sw = self.width();
        let sh = self.height();
        self.size = (sw, sh);
        let (mut dx, mut dy) = (0., 0.);
        RenderNode::Container {
            region: Instruction::new(transform, Rectangle::empty(sw, sh)),
            nodes: self
                .widgets
                .iter_mut()
                .map(|wbox| {
                    let node;
                    wbox.widget.coords = Coords::new(dx, dy);
                    match self.orientation {
                        Orientation::Horizontal => {
                            let _ = wbox.set_height(sh);
                            node = wbox.create_node(transform);
                            dx += wbox.width();
                        }
                        Orientation::Vertical => {
                            let _ = wbox.set_width(sw);
                            node = wbox.create_node(transform);
                            dy += wbox.height();
                        }
                    }
                    node
                })
                .collect(),
        }
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<M>, event: Event<'d, M>) -> Damage {
        let mut damage = Damage::None;
        for wbox in self.widgets.iter_mut() {
            damage = damage.max(wbox.sync(ctx, event));
        }
        damage
    }
}

impl<M: 'static> CenterBox<M> {
    pub fn from(
        first: impl Widget<M> + 'static,
        second: impl Widget<M> + 'static,
        third: impl Widget<M> + 'static,
    ) -> Self {
        Self {
            size: (0., 0.),
            widgets: [
                Child::new(first).clamp().anchor(START, CENTER),
                Child::new(second).clamp().anchor(CENTER, CENTER),
                Child::new(third).clamp().anchor(END, CENTER),
            ],
            orientation: Orientation::Horizontal,
        }
    }
    pub fn new() -> Self {
        Self {
            size: (0., 0.),
            widgets: [
                Child::new(Spacer::default()).clamp().anchor(START, CENTER),
                Child::new(Spacer::default()).clamp().anchor(CENTER, CENTER),
                Child::new(Spacer::default()).clamp().anchor(END, CENTER),
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
