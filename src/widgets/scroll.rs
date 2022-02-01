use crate::*;
use scene::Region;
use widgets::layout::child;
use widgets::*;

/// For widgets that move linearly within in a region.
/// If the step is None, the Scrollable is free to determine it.
/// If the step is Some, the Scrollable has to be shifted by that value.
pub trait Scrollable {
    fn forward(&mut self, step: Option<f32>);
    fn backward(&mut self, step: Option<f32>);
    fn orientation(&self) -> Orientation;
    fn inner_width(&self) -> f32;
    fn inner_height(&self) -> f32;
    fn position(&self) -> f32;
}

pub struct ScrollBox<W> {
    size: f32,
    orientation: Orientation,
    widget: Positioner<Proxy<W>>,
}

impl<W> ScrollBox<W> {
    pub fn new(widget: W) -> Self {
        Self {
            widget: child(widget),
            orientation: Orientation::Vertical,
            size: 100.,
        }
    }
    pub fn set_orientation(&mut self, orientation: Orientation) {
        self.orientation = orientation;
    }
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.set_orientation(orientation);
        self
    }
}

impl<W: Geometry> Scrollable for ScrollBox<W> {
    fn forward(&mut self, step: Option<f32>) {
        let mut coords = self.widget.coords();
        match self.orientation {
            Orientation::Horizontal => {
                match step {
                    Some(delta) => coords.x -= delta,
                    None => coords.x -= 10.,
                }
                if coords.x.abs() < self.size {
                    self.widget.swap(coords);
                }
            }
            Orientation::Vertical => {
                match step {
                    Some(delta) => coords.y += delta,
                    None => coords.y += 10.,
                }
                if coords.y <= 0. {
                    self.widget.swap(coords);
                }
            }
        }
    }
    fn backward(&mut self, step: Option<f32>) {
        let mut coords = self.widget.coords();
        match self.orientation {
            Orientation::Horizontal => {
                match step {
                    Some(delta) => coords.x -= delta,
                    None => coords.x -= 10.,
                }
                if coords.x.abs() < self.size {
                    self.widget.swap(coords);
                }
            }
            Orientation::Vertical => {
                match step {
                    Some(delta) => coords.y -= delta,
                    None => coords.y -= 10.,
                }
                if coords.y.abs() < self.size {
                    self.widget.swap(coords);
                }
            }
        }
    }
    fn position(&self) -> f32 {
        match self.orientation {
            Orientation::Horizontal => self.widget.coords().x,
            Orientation::Vertical => self.widget.coords().y,
        }
    }
    fn inner_width(&self) -> f32 {
        self.widget.width()
    }
    fn inner_height(&self) -> f32 {
        self.widget.height()
    }
    fn orientation(&self) -> Orientation {
        self.orientation
    }
}

impl<W: Geometry> Geometry for ScrollBox<W> {
    fn width(&self) -> f32 {
        match self.orientation {
            Orientation::Horizontal => self.size,
            _ => self.widget.width(),
        }
    }
    fn set_width(&mut self, width: f32) -> Result<(), f32> {
        match self.orientation {
            Orientation::Horizontal => {
                if width.is_sign_positive() {
                    self.size = width;
                    Ok(())
                } else {
                    Err(self.size)
                }
            }
            _ => self.widget.set_width(width),
        }
    }
    fn height(&self) -> f32 {
        match self.orientation {
            Orientation::Vertical => self.size,
            _ => self.widget.height(),
        }
    }
    fn set_height(&mut self, height: f32) -> Result<(), f32> {
        match self.orientation {
            Orientation::Vertical => {
                if height.is_sign_positive() {
                    self.size = height;
                    Ok(())
                } else {
                    Err(self.size)
                }
            }
            _ => self.widget.set_height(height),
        }
    }
}

impl<D, W> Widget<D> for ScrollBox<W>
where
    W: Widget<D>,
{
    fn create_node(&mut self, transform: Transform) -> RenderNode {
        let region = Region::transform(transform, self.width(), self.height());
        RenderNode::Clip(region.into(), Box::new(self.widget.create_node(transform)))
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<D>, event: Event<'d>) -> Damage {
        match event {
            Event::Pointer(_, _, p) => match p {
                Pointer::Scroll { orientation, value } => {
                    let coords = self.widget.coords();
                    let damage = self.widget.sync(ctx, event);
                    if damage.is_none() && orientation == self.orientation {
                        match value {
                            Move::Step(i) => {
                                if i.is_positive() {
                                    for _ in 0..i {
                                        self.forward(None);
                                    }
                                } else {
                                    for _ in i..0 {
                                        self.backward(None);
                                    }
                                }
                            }
                            Move::Value(value) => {
                                if value.is_sign_positive() {
                                    self.forward(Some(value.abs()));
                                } else {
                                    self.backward(Some(value.abs()));
                                }
                            }
                        }
                        if coords != self.widget.coords() {
                            return self.widget.sync(ctx, Event::Prepare).max(damage);
                        }
                    }
                    damage
                }
                _ => self.widget.sync(ctx, event),
            },
            _ => self.widget.sync(ctx, event),
        }
    }
}
