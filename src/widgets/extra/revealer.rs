use crate::scene::Region;
use crate::widgets::extra::*;
use crate::widgets::shapes::Style;
use crate::*;
use crate::{mail::Mail, scene::Coords};
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

enum Direction {
    Normal,
    Inverted,
}

pub enum Transition {
    SlideRight,
    SlideLeft,
    SlideTop,
    SlideBottom,
}

pub struct Revealer<M, E, W>
where
    E: Easer,
{
    widget: Positioner<W>,
    message: M,
    easer: E,
    duration: u32,
    direction: Direction,
    state: RevealerState,
    orientation: Orientation,
    action: Option<RevealerAction>,
}

impl<M, E, W: Geometry> Geometry for Revealer<M, E, W>
where
    E: Easer,
{
    fn width(&self) -> f32 {
        match self.direction {
            Direction::Normal => self.widget.width() - self.widget.coords().x,
            Direction::Inverted => self.widget.width() + self.widget.coords().x,
        }
    }
    fn height(&self) -> f32 {
        match self.direction {
            Direction::Normal => self.widget.height() - self.widget.coords().y,
            Direction::Inverted => self.widget.height() + self.widget.coords().y,
        }
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

impl<M, E, W> GeometryExt for Revealer<M, E, W>
where
    E: Easer,
    W: GeometryExt,
{
    fn apply_width(&mut self, width: f32) {
        self.widget.apply_width(width)
    }
    fn apply_height(&mut self, height: f32) {
        self.widget.apply_height(height)
    }
}

impl<M, E, W, D> Widget<D> for Revealer<M, E, W>
where
    E: Easer,
    W: Widget<D>,
    M: Clone + Copy,
    D: Mail<M, RevealerState, RevealerAction>,
{
    fn create_node(&mut self, transform: Transform) -> RenderNode {
        match self.state {
            RevealerState::Hidden => RenderNode::None,
            _ => {
                if let Some(node) = self.widget.create_node(transform).as_option() {
                    let region = Region::from_transform(
                        transform,
                        self.widget.width(),
                        self.widget.height(),
                    );
                    RenderNode::Clip(region.into(), Box::new(node))
                } else {
                    RenderNode::None
                }
            }
        }
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<D>, event: Event<'d>) -> Damage {
        let d = self.widget.sync(ctx, event);
        match event {
            Event::Sync => {
                if self.state != RevealerState::Running {
                    if let Some(action) = ctx.send(self.message, self.state) {
                        match action {
                            RevealerAction::Reveal => {
                                if self.state == RevealerState::Hidden {
                                    self.reveal();
                                    return d.max(Damage::Frame);
                                }
                            }
                            RevealerAction::Hide => {
                                if self.state == RevealerState::Revealed {
                                    self.hide();
                                    return d.max(Damage::Frame);
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
                                Orientation::Vertical => self.widget.set_coords(0., position),
                                Orientation::Horizontal => self.widget.set_coords(position, 0.),
                            },
                            None => {
                                self.state = match self.action.take().unwrap() {
                                    RevealerAction::Hide => RevealerState::Hidden,
                                    RevealerAction::Reveal => RevealerState::Revealed,
                                };
                                return d.max(Damage::Partial);
                            }
                        }
                    }
                    return d.max(Damage::Frame);
                }
            }
            _ => {}
        }
        d
    }
    fn prepare_draw(&mut self) {
        let direction = match self.direction {
            Direction::Normal => 1.,
            Direction::Inverted => -1.,
        };
        if let RevealerState::Hidden = self.state {
            match self.orientation {
                Orientation::Horizontal => {
                    self.widget.set_coords(direction * self.widget.width(), 0.)
                }
                Orientation::Vertical => {
                    self.widget.set_coords(0., direction * self.widget.height())
                }
            }
        }
        self.widget.prepare_draw()
    }
    fn layout(&mut self, ctx: &mut LayoutCtx) -> (f32, f32) {
        let (width, height) = self.widget.layout(ctx);
        let Coords { x, y } = self.widget.coords();
        match self.direction {
            Direction::Normal => (width - x, height - y),
            Direction::Inverted => (width + x, height + y),
        }
    }
}

impl<M, W> Revealer<M, Sinus, W> {
    pub fn sinus(widget: W, message: M) -> Self {
        Self {
            message,
            direction: Direction::Inverted,
            widget: Positioner::new(widget),
            duration: 500,
            action: None,
            easer: Sinus::new(0., 0.5, 0.),
            orientation: Orientation::Vertical,
            state: RevealerState::Hidden,
        }
    }
}

impl<M, E, W> Revealer<M, E, W>
where
    E: Easer,
    W: Geometry,
{
    pub fn new(widget: W, message: M) -> Self {
        Self {
            message,
            direction: Direction::Inverted,
            widget: Positioner::new(widget),
            duration: 500,
            action: None,
            easer: Easer::new(0., 0.5, 0.),
            orientation: Orientation::Vertical,
            state: RevealerState::Hidden,
        }
    }
    pub fn set_transition(&mut self, transition: Transition) {
        match transition {
            Transition::SlideRight => {
                self.direction = Direction::Normal;
                self.orientation = Orientation::Horizontal;
            }
            Transition::SlideLeft => {
                self.direction = Direction::Inverted;
                self.orientation = Orientation::Horizontal;
            }
            Transition::SlideBottom => {
                self.direction = Direction::Normal;
                self.orientation = Orientation::Vertical;
            }
            Transition::SlideTop => {
                self.direction = Direction::Inverted;
                self.orientation = Orientation::Vertical;
            }
        }
    }
    pub fn transition(mut self, transition: Transition) -> Self {
        self.set_transition(transition);
        self
    }
    fn reveal(&mut self) {
        let direction = match self.direction {
            Direction::Normal => 1.,
            Direction::Inverted => -1.,
        };
        self.state = RevealerState::Running;
        self.easer = match self.orientation {
            Orientation::Horizontal => Easer::new(0.5, 1., direction * self.widget.width()),
            Orientation::Vertical => Easer::new(0.5, 1., direction * self.widget.height()),
        };
        self.action = Some(RevealerAction::Reveal);
    }
    fn hide(&mut self) {
        let direction = match self.direction {
            Direction::Normal => 1.,
            Direction::Inverted => -1.,
        };
        self.state = RevealerState::Running;
        self.easer = match self.orientation {
            Orientation::Horizontal => Easer::new(0., 0.5, direction * self.widget.width()),
            Orientation::Vertical => Easer::new(0., 0.5, direction * self.widget.height()),
        };
        self.action = Some(RevealerAction::Hide);
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

impl<M, E: Easer, W> Deref for Revealer<M, E, W> {
    type Target = W;
    fn deref(&self) -> &Self::Target {
        &self.widget.widget
    }
}

impl<M, E: Easer, W> DerefMut for Revealer<M, E, W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.widget.widget
    }
}

impl<M, E: Easer, W: Style> Style for Revealer<M, E, W> {
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
