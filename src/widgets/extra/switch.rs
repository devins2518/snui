use crate::*;
use crate::data::*;
use crate::widgets::extra::*;
use crate::data::TryIntoMessage;
use crate::scene::Instruction;
use crate::widgets::shapes::{Style, Rectangle};

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum SwitchState {
    Activated,
    Deactivated
}

pub struct Switch<M: TryIntoMessage<SwitchState>> {
    toggle: Rectangle,
    orientation: Orientation,
    state: SwitchState,
    easer: Easer,
    position: f32,
    duration: u32,
    message: Option<M>
}

impl<M: TryIntoMessage<SwitchState>> Geometry for Switch<M> {
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
            self.easer.set_max(width / self.toggle.width())
        }
        Ok(())
    }
    fn set_height(&mut self, height: f32) -> Result<(), f32> {
        self.toggle.set_height(height / 2.)?;
        if let Orientation::Vertical = self.orientation {
            self.easer.set_max(height / self.toggle.height())
        }
        Ok(())
    }
}

impl<M: TryIntoMessage<SwitchState>> Widget<M> for Switch<M> {
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        match self.orientation {
            Orientation::Horizontal => {
                Instruction::new(
                    x + self.position,
                    y,
                    self.toggle.clone()
                ).into()
            }
            Orientation::Vertical => {
                Instruction::new(
                    x,
                    y + self.position,
                    self.toggle.clone()
                ).into()
            }
        }
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<M>, event: &'d Event<'d, M>) -> Damage {
        match event {
            Event::Pointer(x, y, p) => {
                if self.contains(*x, *y) {
                    match *p {
                        Pointer::MouseClick {
                            time: _,
                            button,
                            pressed,
                        } => {
                            if button.is_left() && pressed {
                                let state = match self.state {
                                    SwitchState::Activated => SwitchState::Deactivated,
                                    SwitchState::Deactivated => SwitchState::Activated
                                };
                                if let Some(msg) = self.message.as_ref() {
                                    if let Ok(msg) = TryIntoMessage::try_into(msg, state) {
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
                self.easer.frame_time(*frame_time);
                match self.state {
                    SwitchState::Activated => match self.easer.trend() {
                        Trend::Neutral | Trend::Positive => if let Some(position) = self.easer.next() {
                            self.position = position;
                            return Damage::Frame;
                        }
                        _ => {}
                    }
                    SwitchState::Deactivated => match self.easer.trend() {
                        Trend::Negative => if let Some(position) = self.easer.next() {
                            self.position = position;
                            return Damage::Frame
                        }
                        Trend::Neutral => self.easer.reset(self.duration),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        Damage::None
    }
}

impl<M: TryIntoMessage<SwitchState>> Default for Switch<M> {
    fn default() -> Self {
        Self {
            toggle: Rectangle::empty(20., 20.)
            .background(style::BG2),
        	position: 0.,
        	easer: Easer::new(Start::Min, 20., 500, Curve::Sinus),
        	orientation: Orientation::Horizontal,
        	message: None,
        	duration: 500,
        	state: SwitchState::Deactivated
        }
    }
}

impl<M: TryIntoMessage<SwitchState>> Switch<M> {
    // Time in ms
    pub fn duration(mut self, duration: u32) -> Self {
        self.duration = duration / 2;
        self.easer.reset(self.duration);
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

impl<M: TryIntoMessage<SwitchState>> Style for Switch<M> {
    fn set_background<B: Into<scene::Background>>(&mut self, background: B) {
        self.toggle.set_background(background);
    }
    fn set_border(&mut self, color: u32, width: f32) {
        self.toggle.set_border(color, width);
    }
    fn set_border_color(&mut self, color: u32) {
        self.toggle.set_border_color(color);
    }
    fn set_radius(&mut self, tl: f32, tr: f32, br: f32, bl: f32) {
        self.toggle.set_radius(tl, tr, br, bl);
    }
    fn set_border_size(&mut self, size: f32) {
        self.toggle.set_border_size(size);
    }
    fn background<B: Into<scene::Background>>(mut self, background: B) -> Self {
        self.set_background(background);
        self
    }
    fn border(mut self, color: u32, size: f32) -> Self {
        self.set_border(color, size);
        self
    }
    fn border_color(mut self, color: u32) -> Self {
        self.set_border_color(color);
        self
    }
    fn border_size(mut self, size: f32) -> Self {
        self.set_border_size(size);
        self
    }
    fn radius(mut self, tl: f32, tr: f32, br: f32, bl: f32) -> Self {
        self.set_radius(tl, tr, br, bl);
        self
    }
}
