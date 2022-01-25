use crate::controller::TryFromArg;
use crate::controller::*;
use crate::scene::Instruction;
use crate::widgets::extra::*;
use crate::widgets::shapes::{Rectangle, Style};
use crate::*;

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum SwitchState {
    Activated,
    Deactivated,
}

pub struct Switch<M: TryFromArg<SwitchState>> {
    start: bool,
    toggle: Rectangle,
    orientation: Orientation,
    state: SwitchState,
    position: f32,
    easer: Sinus,
    duration: u32,
    message: Option<M>,
}

impl<M: TryFromArg<SwitchState>> Geometry for Switch<M> {
    fn width(&self) -> f32 {
        if let Orientation::Horizontal = self.orientation {
            self.toggle.width() * 2.
        } else {
            self.toggle.width()
        }
    }
    fn height(&self) -> f32 {
        if let Orientation::Vertical = self.orientation {
            self.toggle.height() * 2.
        } else {
            self.toggle.height()
        }
    }
    fn set_width(&mut self, width: f32) -> Result<(), f32> {
        self.toggle.set_width(width / 2.)?;
        if let Orientation::Horizontal = self.orientation {
            self.easer.set_amplitude(width - self.toggle.width())
        }
        Ok(())
    }
    fn set_height(&mut self, height: f32) -> Result<(), f32> {
        self.toggle.set_height(height / 2.)?;
        if let Orientation::Vertical = self.orientation {
            self.easer.set_amplitude(height - self.toggle.height())
        }
        Ok(())
    }
}

impl<M: TryFromArg<SwitchState>> Widget<M> for Switch<M> {
    fn create_node(&mut self, transform: Transform) -> RenderNode {
        let transform = match self.orientation {
            Orientation::Horizontal => {
                transform.pre_translate(self.position, 0.)
            }
            Orientation::Vertical => {
                transform.pre_translate(0., self.position)
            }
        };
        Widget::<()>::create_node(
            &mut self.toggle,
            transform
        )
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<M>, event: Event<'d, M>) -> Damage {
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
                                        self.easer =
                                            Sinus::new(0.5, 1., self.width() - self.toggle.width());
                                        SwitchState::Deactivated
                                    }
                                    SwitchState::Deactivated => {
                                        self.easer =
                                            Sinus::new(0., 0.5, self.width() - self.toggle.width());
                                        SwitchState::Activated
                                    }
                                };
                                if let Some(msg) = self.message.as_ref() {
                                    if let Ok(msg) = msg.try_from_arg(state) {
                                        if ctx.send(msg).is_ok() {
                                            self.state = state;
                                            return Damage::Frame;
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            Event::Callback(frame_time) => {
                if self.start {
                    let steps =
                        (frame_time * self.easer.steps() as u32) as usize / self.duration as usize;
                    for _ in 0..steps {
                        match self.easer.next() {
                            Some(position) => self.position = position,
                            None => {
                                self.start = false;
                                return Damage::None;
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
}

impl<M: TryFromArg<SwitchState>> Default for Switch<M> {
    fn default() -> Self {
        Self {
            start: false,
            position: 0.,
            toggle: Rectangle::empty(20., 20.).background(style::BG2),
            easer: Sinus::new(0., 0.5, 20.),
            orientation: Orientation::Horizontal,
            message: None,
            duration: 500,
            state: SwitchState::Deactivated,
        }
    }
}

impl<M: TryFromArg<SwitchState>> Switch<M> {
    // Time in ms
    pub fn duration(mut self, duration: u32) -> Self {
        self.duration = duration;
        self
    }
    pub fn message(mut self, message: M) -> Self {
        self.message = Some(message);
        self
    }
    pub fn state(&self) -> SwitchState {
        self.state
    }
}

impl<M: TryFromArg<SwitchState>> Style for Switch<M> {
    fn set_background<B: Into<scene::Texture>>(&mut self, background: B) {
        self.toggle.set_background(background);
    }
    fn set_border_texture<T: Into<scene::Texture>>(&mut self, texture: T) {
        self.toggle.set_border_texture(texture);
    }
    fn set_radius(&mut self, tl: f32, tr: f32, br: f32, bl: f32) {
        self.toggle.set_radius(tl, tr, br, bl);
    }
    fn set_border_size(&mut self, size: f32) {
        self.toggle.set_border_size(size);
    }
}
