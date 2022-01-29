use crate::widgets::container::*;
use crate::widgets::shapes::Rectangle;
use crate::*;
use scene::Instruction;
use scene::RenderNode;

pub struct LayoutBox<W> {
    size: (f32, f32),
    orientation: Orientation,
    widgets: Vec<Positioner<Proxy<W>>>,
}

impl<W> FromIterator<W> for LayoutBox<W> {
    fn from_iter<T: IntoIterator<Item = W>>(iter: T) -> Self {
        let mut layoutbox = LayoutBox::new();
        for widget in iter {
            layoutbox.widgets.push(child(widget));
        }
        layoutbox
    }
}

impl<D, W> Container<D, W> for LayoutBox<W>
where
    W: Widget<D>,
{
    fn len(&self) -> usize {
        self.widgets.len()
    }
    fn add(&mut self, widget: W) {
        self.widgets.push(child(widget))
    }
    fn remove(&mut self, index: usize) -> W {
        self.widgets.remove(index).widget.inner
    }
    fn widgets(&mut self) -> Vec<&mut W> {
        self.widgets
            .iter_mut()
            .map(|inner| inner.widget.deref_mut())
            .collect()
    }
}

impl<W: Geometry> Geometry for LayoutBox<W> {
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

impl<D, W: Widget<D>> Widget<D> for LayoutBox<W> {
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
                    child.set_coords(dx, dy);
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
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<D>, event: Event<'d>) -> Damage {
        let mut damage = Damage::None;
        for child in self.widgets.iter_mut() {
            damage = damage.max(child.sync(ctx, event));
        }
        damage
    }
}

impl<D> Default for LayoutBox<Box<dyn Widget<D>>> {
    fn default() -> Self {
        Self {
            size: (0., 0.),
            widgets: Vec::new(),
            orientation: Orientation::Horizontal,
        }
    }
}

impl<D> LayoutBox<Box<dyn Widget<D>>> {
    pub fn add<W: Widget<D> + 'static>(&mut self, widget: W) {
        self.widgets.push(child(Box::new(widget)));
    }
}

impl<W> LayoutBox<W> {
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
    pub fn inner(&mut self) -> &mut [Positioner<Proxy<W>>] {
        self.widgets.as_mut_slice()
    }
}
