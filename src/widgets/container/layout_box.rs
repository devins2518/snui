use crate::widgets::container::Child;
use crate::*;
use scene::{Coords, Region, RenderNode};

pub struct LayoutBox {
    widgets: Vec<Child>,
    orientation: Orientation,
}

impl Container for LayoutBox {
    fn len(&self) -> usize {
        self.widgets.len()
    }
    fn add(&mut self, widget: impl Widget + 'static) {
        self.widgets.push(Child::new(widget));
    }
}

impl Geometry for LayoutBox {
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

impl Widget for LayoutBox {
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        let sw = self.width();
        let sh = self.height();
        let (mut dx, mut dy) = (0., 0.);
        RenderNode::Container {
            region: Region::new(x, y, sw, sh),
            nodes: self
                .widgets
                .iter_mut()
                .map(|child| {
                    let node;
                    child.coords = Coords::new(dx, dy);
                    match self.orientation {
                        Orientation::Horizontal => {
                            let _ = child.set_height(sh);
                            let ww = child.width();
                            node = RenderNode::Extension {
                                background: scene::Instruction::empty(x + dx, y + dy, ww, sh),
                                border: None,
                                node: Box::new(child.widget.create_node(x + dx, y + dy)),
                            };
                            dx += child.width().round();
                        }
                        Orientation::Vertical => {
                            let _ = child.set_width(sw);
                            let wh = child.height();
                            node = RenderNode::Extension {
                                background: scene::Instruction::empty(x + dx, y + dy, sw, wh),
                                border: None,
                                node: Box::new(child.widget.create_node(x + dx, y + dy)),
                            };
                            dy += child.height().round();
                        }
                    }
                    node
                })
                .collect(),
        }
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext, event: Event) {
        for child in self.widgets.iter_mut() {
            child.sync(ctx, event);
        }
    }
}

impl LayoutBox {
    pub fn new() -> Self {
        Self {
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
