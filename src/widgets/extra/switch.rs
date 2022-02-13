use crate::mail::*;
use crate::widgets::extra::*;
use crate::widgets::shapes::{Rectangle, Style};
use crate::*;

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum SwitchState {
    Activated,
    Deactivated,
}

pub struct Switch<M> {
    start: bool,
    toggle: Positioner<Rectangle>,
    state: SwitchState,
    easer: Sinus,
    duration: u32,
    message: M,
}

impl<M> Geometry for Switch<M> {
    fn width(&self) -> f32 {
        self.toggle.width() * 2.
    }
    fn height(&self) -> f32 {
        self.toggle.height()
    }
}

impl<M> GeometryExt for Switch<M> {
    fn apply_width(&mut self, width: f32) {
        self.toggle.apply_width(width / 2.);
        self.easer.set_amplitude(self.toggle.width() / 2.);
    }
    fn apply_height(&mut self, height: f32) {
        self.toggle.apply_height(height);
    }
}

impl<M, D> Widget<D> for Switch<M>
where
    M: Clone + Copy,
    D: Mail<M, bool, bool>,
{
    fn create_node(&mut self, transform: Transform) -> RenderNode {
        Widget::<()>::create_node(&mut self.toggle, transform)
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<D>, event: Event<'d>) -> Damage {
        match event {
            Event::Pointer(x, y, p) => {
                if self.contains(x, y) {
                    match p {
                        Pointer::MouseClick {
                            serial: _,
                            button,
                            pressed,
                        } => {
                            if button.is_left() && pressed {
                                self.start = true;
                                let state = match self.state {
                                    SwitchState::Activated => {
                                        self.easer = Sinus::new(0.5, 1., self.toggle.width());
                                        SwitchState::Deactivated
                                    }
                                    SwitchState::Deactivated => {
                                        self.easer = Sinus::new(0., 0.5, self.toggle.width());
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
                        _ => {}
                    }
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

impl<M> Switch<M> {
    pub fn new(message: M) -> Self {
        Self {
            start: false,
            toggle: Positioner::new(Rectangle::new(20., 20.).background(theme::BG2)),
            easer: Sinus::new(0., 0.5, 20.),
            message,
            duration: 500,
            state: SwitchState::Deactivated,
        }
    }
    fn set_state(&mut self, state: SwitchState) {
        match state {
            SwitchState::Activated => {
                self.easer = Sinus::new(0.5, 1., self.toggle.width());
            }
            SwitchState::Deactivated => {
                self.easer = Sinus::new(0., 0.5, self.toggle.width());
            }
        };
        self.state = state;
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
    pub fn state(&self) -> SwitchState {
        self.state
    }
}

impl<M> Style for Switch<M> {
    fn set_background<B: Into<scene::Texture>>(&mut self, background: B) {
        self.toggle.set_background(background);
    }
    fn set_border_texture<T: Into<scene::Texture>>(&mut self, texture: T) {
        self.toggle.set_border_texture(texture);
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
    fn set_border_size(&mut self, size: f32) {
        self.toggle.set_border_size(size);
    }
}
