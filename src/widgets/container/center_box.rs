use crate::widgets::container::{Child, Container};
use crate::widgets::*;
use crate::*;
use scene::Region;

pub struct CenterBox {
    orientation: Orientation,
    widgets: [WidgetBox<Child>; 3],
}

impl FromIterator<Child> for CenterBox {
    fn from_iter<T: IntoIterator<Item = Child>>(iter: T) -> Self {
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

impl Container for CenterBox {
    fn len(&self) -> usize {
        self.widgets.len()
    }
    fn add(&mut self, widget: impl Widget + 'static) {
        for wbox in self.widgets.iter_mut() {
            if wbox.width() == 0. && wbox.height() == 0. {
                wbox.widget.widget = Box::new(widget);
                break;
            }
        }
    }
}

impl Geometry for CenterBox {
    fn set_width(&mut self, width: f32) -> Result<(), f32> {
        let mut local_width = 0.;
        let size = (width / self.widgets.len() as f32).ceil();
        for child in self.widgets.iter_mut() {
            match self.orientation {
                Orientation::Horizontal => {
                    if let Err(w) = child.set_width(size) {
                        local_width += w;
                    } else {
                        local_width += size;
                    }
                }
                Orientation::Vertical => {
                    if let Err(w) = child.set_width(width) {
                        local_width = local_width.max(w);
                    }
                }
            }
        }
        if local_width == width {
            return Ok(());
        }
        Err(local_width)
    }
    fn set_height(&mut self, height: f32) -> Result<(), f32> {
        let mut local_height = 0.;
        let size = (height / self.widgets.len() as f32).ceil();
        for child in self.widgets.iter_mut() {
            match self.orientation {
                Orientation::Vertical => {
                    if let Err(h) = child.set_height(size) {
                        local_height += h;
                    } else {
                        local_height += size;
                    }
                }
                Orientation::Horizontal => {
                    if let Err(w) = child.set_height(height) {
                        local_height = local_height.max(w);
                    }
                }
            }
        }
        if local_height == height {
            return Ok(());
        }
        Err(local_height)
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

impl Widget for CenterBox {
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        let sw = self.width();
        let sh = self.height();
        let (mut dx, mut dy) = (0., 0.);
        RenderNode::Container {
            region: Region::new(x, y, sw, sh),
            nodes: self
                .widgets
                .iter_mut()
                .map(|wbox| {
                    let node;
                    wbox.widget.coords = Coords::new(dx, dy);
                    match self.orientation {
                        Orientation::Horizontal => {
                            let _ = wbox.set_height(sh);
                            node = wbox.create_node(x, y);
                            dx += wbox.width().round();
                        }
                        Orientation::Vertical => {
                            let _ = wbox.set_width(sw);
                            node = wbox.create_node(x, y);
                            dy += wbox.height().round();
                        }
                    }
                    node
                })
                .collect(),
        }
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext, event: Event) -> Damage {
        let mut damage = Damage::None;
        for wbox in self.widgets.iter_mut() {
            damage.order(wbox.sync(ctx, event));
        }
        damage
    }
}

impl CenterBox {
    pub fn from(
        first: impl Widget + 'static,
        second: impl Widget + 'static,
        third: impl Widget + 'static,
    ) -> Self {
        Self {
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
