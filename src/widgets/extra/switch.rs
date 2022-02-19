use crate::mail::*;
use crate::widgets::extra::*;
use crate::widgets::shapes::{Rectangle, Style};
use crate::*;

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum SwitchState {
    Activated,
    Deactivated,
}

pub struct Switch<M, E: Easer> {
    start: bool,
    toggle: Positioner<Rectangle>,
    state: SwitchState,
    easer: E,
    duration: u32,
    message: M,
}

impl<M> Switch<M, Quadratic> {
    pub fn default(message: M) -> Self {
        Self {
            start: false,
            toggle: Positioner::new(Rectangle::new(20., 20.).texture(theme::BG2)),
            easer: Easer::new(0., 0.5, 20.),
            message,
            duration: 500,
            state: SwitchState::Deactivated,
        }
    }
}

impl<M, E: Easer> Switch<M, E> {
    pub fn new(message: M) -> Self {
        Self {
            start: false,
            toggle: Positioner::new(Rectangle::new(20., 20.).texture(theme::BG2)),
            easer: Easer::new(0., 0.5, 20.),
            message,
            duration: 500,
            state: SwitchState::Deactivated,
        }
    }
    fn set_state(&mut self, state: SwitchState) {
        match state {
            SwitchState::Activated => {
                self.easer = Easer::new(0.5, 1., self.toggle.width());
            }
            SwitchState::Deactivated => {
                self.easer = Easer::new(0., 0.5, self.toggle.width());
            }
        };
        self.state = state;
    }
    /// Duration of the animation in ms
    pub fn duration(mut self, duration: u32) -> Self {
        self.duration = duration;
        self
    }
    pub fn state(&self) -> SwitchState {
        self.state
    }
}

impl<M, E: Easer> Geometry for Switch<M, E> {
    fn width(&self) -> f32 {
        self.toggle.width() * 2.
    }
    fn height(&self) -> f32 {
        self.toggle.height()
    }
}

impl<M, E: Easer> GeometryExt for Switch<M, E> {
    fn set_width(&mut self, width: f32) {
        self.toggle.set_width(width / 2.);
        self.easer.set_amplitude(self.toggle.width() / 2.);
    }
    fn set_height(&mut self, height: f32) {
        self.toggle.set_height(height);
    }
}

impl<M, E, T> Widget<T> for Switch<M, E>
where
    M: Clone + Copy,
    E: Easer,
    T: Mail<M, bool, bool>,
{
    fn draw_scene(&mut self, scene: Scene) {
        Widget::<()>::draw_scene(&mut self.toggle, scene)
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<T>, event: Event<'d>) -> Damage {
        match event {
            Event::Pointer(x, y, p) => {
                if self.contains(x, y) && p.left_button_click().is_some() {
                    self.start = true;
                    let state = match self.state {
                        SwitchState::Activated => {
                            self.easer = Easer::new(0.5, 1., self.toggle.width());
                            SwitchState::Deactivated
                        }
                        SwitchState::Deactivated => {
                            self.easer = Easer::new(0., 0.5, self.toggle.width());
                            SwitchState::Activated
                        }
                    };
                    self.state = state;
                    match self.state {
                        SwitchState::Activated => ctx.send(self.message, true),
                        SwitchState::Deactivated => ctx.send(self.message, false),
                    };
                    return Damage::Frame;
                }
            }
            Event::Sync => {
                if let Some(state) = ctx.get(self.message) {
                    let state = if state {
                        SwitchState::Activated
                    } else {
                        SwitchState::Deactivated
                    };
                    if state != self.state {
                        self.set_state(state);
                        return Damage::Frame;
                    }
                }
            }
            Event::Callback(frame_time) => {
                if self.start {
                    let steps =
                        (frame_time * self.easer.steps() as u32) as usize / self.duration as usize;
                    for _ in 0..steps {
                        match self.easer.next() {
                            Some(position) => self.toggle.set_coords(position, 0.),
                            None => {
                                self.start = false;
                                return Damage::Frame;
                            }
                        }
                    }
                    return Damage::Frame;
                }
            }
            _ => {}
        }
        Damage::None
    }
    fn layout(&mut self, _ctx: &mut LayoutCtx, _constraints: &BoxConstraints) -> Size {
        (self.width(), self.height()).into()
    }
}

impl<M, E: Easer> Style for Switch<M, E> {
    fn set_texture<B: Into<scene::Texture>>(&mut self, texture: B) {
        self.toggle.set_texture(texture);
    }
    fn set_top_left_radius(&mut self, radius: f32) {
        self.toggle.set_top_left_radius(radius);
    }
    fn set_top_right_radius(&mut self, radius: f32) {
        self.toggle.set_top_right_radius(radius);
    }
    fn set_bottom_right_radius(&mut self, radius: f32) {
        self.toggle.set_bottom_right_radius(radius);
    }
    fn set_bottom_left_radius(&mut self, radius: f32) {
        self.toggle.set_bottom_left_radius(radius);
    }
}
