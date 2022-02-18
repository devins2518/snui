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

/// A widget that displays its child using an animated transition
pub struct Revealer<M, E, W>
where
    E: Easer,
{
    widget: Positioner<W>,
    message: M,
    easer: E,
    duration: u32,
    size: Size,
    direction: Direction,
    state: RevealerState,
    orientation: Orientation,
    action: Option<RevealerAction>,
}

impl<M, E, W> Geometry for Revealer<M, E, W>
where
    E: Easer,
{
    fn width(&self) -> f32 {
        match self.direction {
            Direction::Normal => self.size.width - self.widget.coords().x,
            Direction::Inverted => self.size.width + self.widget.coords().x,
        }
    }
    fn height(&self) -> f32 {
        match self.direction {
            Direction::Normal => self.size.height - self.widget.coords().y,
            Direction::Inverted => self.size.height + self.widget.coords().y,
        }
    }
}

impl<M, E, W, T> Widget<T> for Revealer<M, E, W>
where
    E: Easer,
    W: Widget<T>,
    M: Clone + Copy,
    T: Mail<M, RevealerState, RevealerAction>,
{
    fn draw_scene(&mut self, mut scene: Scene) {
        match self.state {
            RevealerState::Hidden => {}
            _ => {
                if let Some(scene) = scene.apply_clip(self.size) {
                    self.widget.draw_scene(scene)
                }
            }
        }
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<T>, event: Event<'d>) -> Damage {
        match event {
            Event::Sync => {
                if self.state != RevealerState::Running {
                    if let Some(action) = ctx.send(self.message, self.state) {
                        match action {
                            RevealerAction::Reveal => {
                                if self.state == RevealerState::Hidden {
                                    self.reveal();
                                    return self.widget.sync(ctx, event).max(Damage::Frame);
                                }
                            }
                            RevealerAction::Hide => {
                                if self.state == RevealerState::Revealed {
                                    self.hide();
                                    return self.widget.sync(ctx, event).max(Damage::Frame);
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
                                return self
                                    .widget
                                    .sync(ctx, event)
                                    .max(self.widget.sync(ctx, Event::Draw));
                            }
                        }
                    }
                    return self
                        .widget
                        .sync(ctx, Event::Draw)
                        .max(self.widget.sync(ctx, event))
                        .max(Damage::Frame);
                }
            }
            Event::Draw => {
                if self.state == RevealerState::Hidden {
                    return Damage::None;
                }
            }
            _ => {}
        }
        match self.state {
            RevealerState::Hidden => Damage::None,
            _ => self.widget.sync(ctx, event),
        }
    }
    fn layout(&mut self, ctx: &mut LayoutCtx, constraints: &BoxConstraints) -> Size {
        self.size = self.widget.layout(ctx, constraints);
        let Coords { x, y } = self.widget.coords();
        let (width, height) = self.size.into();
        match self.orientation {
            Orientation::Vertical => match self.direction {
                Direction::Normal => (width - x, height),
                Direction::Inverted => (width + x, height),
            },
            Orientation::Horizontal => match self.direction {
                Direction::Normal => (width, height - y),
                Direction::Inverted => (width, height + y),
            },
        }
        .into()
    }
}

impl<M, W> Revealer<M, Sinus, W> {
    pub fn sinus(widget: W, message: M) -> Self {
        Self {
            message,
            size: Size::default(),
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
{
    pub fn new(widget: W, message: M) -> Self {
        Self {
            message,
            size: Default::default(),
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
            Orientation::Horizontal => {
                self.widget.set_coords(direction * self.size.width, 0.);
                Easer::new(0.5, 1., direction * self.size.width)
            }
            Orientation::Vertical => {
                self.widget.set_coords(0., direction * self.size.height);
                Easer::new(0.5, 1., direction * self.size.height)
            }
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
            Orientation::Horizontal => Easer::new(0., 0.5, direction * self.size.width),
            Orientation::Vertical => Easer::new(0., 0.5, direction * self.size.height),
        };
        self.action = Some(RevealerAction::Hide);
    }
    /// Duration of the animation in ms
    pub fn duration(mut self, duration: u32) -> Self {
        self.duration = duration;
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
    fn set_texture<B: Into<scene::Texture>>(&mut self, texture: B) {
        self.widget.set_texture(texture);
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
}
