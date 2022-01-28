use crate::widgets::container::*;
use crate::widgets::shapes::Rectangle;
use crate::*;
use scene::Instruction;
use scene::{Coords, RenderNode};

pub struct LayoutBox<M> {
    size: (f32, f32),
    widgets: Vec<Child<M>>,
    orientation: Orientation,
}


impl<M, C> FromIterator<C> for LayoutBox<M>
where
    M: 'static,
    C: Widget<M> + 'static
{
    fn from_iter<T: IntoIterator<Item = C>>(iter: T) -> Self {
        let mut layoutbox = LayoutBox::new();
        for c in iter {
            layoutbox.widgets.push(c.child());
        }
        layoutbox
    }
}

impl<M: 'static, C> Container<M, Child<M>, C> for LayoutBox<M>
where
    M: 'static,
    C: Widget<M> + 'static
{
    fn len(&self) -> usize {
        self.widgets.len()
    }
    fn add(&mut self, widget: C) {
        self.widgets.push(Child::new(widget));
    }
    fn remove(&mut self, index: usize) -> Child<M> {
        self.widgets.remove(index)
    }
    fn widgets(&mut self) -> &mut [Child<M>] {
        &mut self.widgets
    }
}

impl<M> Geometry for LayoutBox<M> {
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
        if self.size.0 == width {
            Ok(())
        } else {
            Err(self.size.0)
        }
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
        if self.size.1 == height {
            Ok(())
        } else {
            Err(self.size.1)
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
        width.ceil()
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
        height.ceil()
    }
}

impl<M> Widget<M> for LayoutBox<M> {
    fn create_node(&mut self, transform: Transform) -> RenderNode {
        let sw = self.width();
        let sh = self.height();
        self.size = (sw, sh);
        let (mut dx, mut dy) = (0., 0.);
        RenderNode::Container(
            Instruction::new(transform, Rectangle::empty(sw, sh)),
            self.widgets
                .iter_mut()
                .map(|child| {
                    let node;
                    child.coords = Coords::new(dx, dy);
                    match self.orientation {
                        Orientation::Horizontal => {
                            let _ = child.set_height(sh);
                            node = child.create_node(transform);
                            dx += child.width();
                        }
                        Orientation::Vertical => {
                            let _ = child.set_width(sw);
                            node = child.create_node(transform);
                            dy += child.height();
                        }
                    }
                    node
                })
                .collect(),
        )
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<M>, event: Event<'d, M>) -> Damage {
        let mut damage = Damage::None;
        for child in self.widgets.iter_mut() {
            damage = damage.max(child.sync(ctx, event));
        }
        damage
    }
}

impl<M> LayoutBox<M> {
    pub fn new() -> Self {
        Self {
            size: (0., 0.),
            widgets: Vec::new(),
            orientation: Orientation::Horizontal,
        }
    }
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }
    pub fn clear(&mut self) {
        self.widgets.clear();
    }
}
