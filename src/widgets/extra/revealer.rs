use crate::mail::Mail;
use crate::scene::Region;
use crate::widgets::extra::*;
use crate::widgets::shapes::Style;
use crate::*;
use std::ops::{Deref, DerefMut};

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum RevealerAction {
    Reveal,
    Hide,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum RevealerState {
    Running,
    Hidden,
    Revealed,
}

pub struct Revealer<M, W> {
    widget: Positioner<W>,
    message: M,
    easer: Sinus,
    duration: u32,
    state: RevealerState,
    orientation: Orientation,
    action: Option<RevealerAction>,
}

impl<M, W: Geometry> Geometry for Revealer<M, W> {
    fn width(&self) -> f32 {
        self.widget.width() + self.widget.coords().x
    }
    fn height(&self) -> f32 {
        self.widget.height() + self.widget.coords().y
    }
    fn set_width(&mut self, width: f32) {
        match self.state {
            RevealerState::Revealed => self.widget.set_width(width),
            _ => {}
        }
    }
    fn set_height(&mut self, height: f32) {
        match self.state {
            RevealerState::Revealed => self.widget.set_height(height),
            _ => {}
        }
    }
    fn maximum_height(&self) -> f32 {
        match self.state {
            RevealerState::Running => self.width(),
            _ => self.widget.maximum_height(),
        }
    }
    fn minimum_height(&self) -> f32 {
        match self.state {
            RevealerState::Running => self.width(),
            _ => self.widget.minimum_height(),
        }
    }
    fn maximum_width(&self) -> f32 {
        match self.state {
            RevealerState::Running => self.width(),
            _ => self.widget.maximum_width(),
        }
    }
    fn minimum_width(&self) -> f32 {
        match self.state {
            RevealerState::Running => self.width(),
            _ => self.widget.minimum_width(),
        }
    }
}

impl<M, W: GeometryExt> GeometryExt for Revealer<M, W> {
    fn apply_width(&mut self, width: f32) {
        self.widget.apply_width(width)
    }
    fn apply_height(&mut self, height: f32) {
        self.widget.apply_height(height)
    }
}

impl<M, W, D> Widget<D> for Revealer<M, W>
where
    W: Widget<D>,
    M: Clone + Copy,
    D: Mail<M, RevealerState, RevealerAction>,
{
    fn create_node(&mut self, transform: Transform) -> RenderNode {
        if let Some(node) = self.widget.create_node(transform).as_option() {
            let region =
                Region::from_transform(transform, self.widget.width(), self.widget.height());
            RenderNode::Clip(region.into(), Box::new(node))
        } else {
            RenderNode::None
        }
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<D>, event: Event<'d>) -> Damage {
        match event {
            Event::Sync => {
                if self.state != RevealerState::Running {
                    if let Some(action) = ctx.send(self.message, self.state) {
                        match action {
                            RevealerAction::Reveal => {
                                if self.state == RevealerState::Hidden {
                                    self.state = RevealerState::Running;
                                    self.easer = match self.orientation {
                                        Orientation::Horizontal => {
                                            Sinus::new(0.5, 1., self.widget.width())
                                        }
                                        Orientation::Vertical => {
                                            Sinus::new(0.5, 1., self.widget.height())
                                        }
                                    };
                                    self.action = Some(action);
                                    return Damage::Frame;
                                }
                            }
                            RevealerAction::Hide => {
                                if self.state == RevealerState::Revealed {
                                    self.state = RevealerState::Running;
                                    self.easer = match self.orientation {
                                        Orientation::Horizontal => {
                                            Sinus::new(0., 0.50, self.widget.width())
                                        }
                                        Orientation::Vertical => {
                                            Sinus::new(0., 0.50, self.widget.height())
                                        }
                                    };
                                    self.action = Some(action);
                                    return Damage::Frame;
                                }
                            }
                        }
                    }
                }
            }
            Event::Callback(frame_time) => {
                if let RevealerState::Running = self.state {
                    let steps =
                        (frame_time * self.easer.steps() as u32) as usize / self.duration as usize;
                    for _ in 0..steps {
                        match self.easer.next() {
                            Some(position) => match self.orientation {
                                Orientation::Vertical => self.widget.set_coords(0., -position),
                                Orientation::Horizontal => self.widget.set_coords(-position, 0.),
                            },
                            None => {
                                self.state = match self.action.take().unwrap() {
                                    RevealerAction::Hide => RevealerState::Hidden,
                                    RevealerAction::Reveal => RevealerState::Revealed,
                                };
                                return Damage::Partial;
                            }
                        }
                    }
                    return Damage::Frame;
                }
            }
            _ => {}
        }
        self.widget.sync(ctx, event)
    }
    fn prepare_draw(&mut self) {
        if let RevealerState::Hidden = self.state {
            match self.orientation {
                Orientation::Horizontal => self.widget.set_coords(-self.widget.width(), 0.),
                Orientation::Vertical => self.widget.set_coords(0., -self.widget.height()),
            }
        }
        self.widget.prepare_draw()
    }
}

impl<M, W> Revealer<M, W> {
    pub fn new(widget: W, message: M) -> Self {
        Self {
            message,
            widget: Positioner::new(widget),
            duration: 500,
            action: None,
            easer: Sinus::new(0., 0.5, 0.),
            orientation: Orientation::Vertical,
            state: RevealerState::Hidden,
        }
    }
    /// Duration of the animation in ms
    pub fn duration(mut self, duration: u32) -> Self {
        self.duration = duration;
        self
    }
    pub fn message(mut self, message: M) -> Self {
        self.message = message;
        self
    }
    pub fn state(&self) -> RevealerState {
        self.state
    }
}

impl<M, W> Deref for Revealer<M, W> {
    type Target = W;
    fn deref(&self) -> &Self::Target {
        &self.widget.widget
    }
}

impl<M, W> DerefMut for Revealer<M, W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.widget.widget
    }
}

impl<M, W: Style> Style for Revealer<M, W> {
    fn set_background<B: Into<scene::Texture>>(&mut self, background: B) {
        self.widget.set_background(background);
    }
    fn set_border_texture<T: Into<scene::Texture>>(&mut self, texture: T) {
        self.widget.set_border_texture(texture);
    }
    fn set_top_left_radius(&mut self, radius: f32) {
        self.widget.set_top_left_radius(radius);
    }
    fn set_top_right_radius(&mut self, radius: f32) {
        self.widget.set_top_right_radius(radius);
    }
    fn set_bottom_right_radius(&mut self, radius: f32) {
        self.widget.set_bottom_right_radius(radius);
    }
    fn set_bottom_left_radius(&mut self, radius: f32) {
        self.widget.set_bottom_left_radius(radius);
    }
    fn set_border_size(&mut self, size: f32) {
        self.widget.set_border_size(size);
    }
}
