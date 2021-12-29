use scene::Instruction;
use snui::data::*;
use snui::wayland::shell::*;
use snui::widgets::container::*;
use snui::widgets::shapes::*;
use snui::{
    widgets::{text::*, *},
    *,
};
use std::f32::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq)]
enum AnimationState {
    Stop,
    Start,
    Pause,
}

#[derive(Debug, Clone, Copy)]
struct EaserCtl {
    block: bool,
    state: AnimationState,
}

impl Default for EaserCtl {
    fn default() -> Self {
        EaserCtl {
            block: false,
            state: AnimationState::Stop
        }
    }
}

impl Controller<AnimationState> for EaserCtl {
    fn serialize(&mut self, _msg: Message<AnimationState>) -> Result<u32, ControllerError> {
        Err(ControllerError::NonBlocking)
    }
    fn deserialize(&mut self, _token: u32) -> Result<(), ControllerError> {
        Err(ControllerError::NonBlocking)
    }
    fn get<'c>(&'c self, _msg: Message<AnimationState>) -> Result<Data<'c, AnimationState>, ControllerError> {
        return Ok(Data::Request(self.state));
    }
    fn send<'c>(&'c mut self, msg: Message<AnimationState>) -> Result<Data<'c, AnimationState>, ControllerError> {
        let Message(state, _) = msg;
        match state {
            AnimationState::Stop | AnimationState::Pause => self.block = false,
            _ => {}
        }
        self.state = state;
        Ok(Data::Null)
    }
    fn sync(&mut self) -> Result<Message<'static, AnimationState>, ControllerError> {
        if !self.block {
            match self.state {
                AnimationState::Stop => return Err(ControllerError::NonBlocking),
                AnimationState::Start => {
                    self.block = true;
                    return Ok(Message::new(self.state, ()));
                }
                AnimationState::Pause => {
                    self.block = true;
                    return Ok(Message::new(self.state, ()))
                }
            }
        }
        Err(ControllerError::Blocking)
    }
}

enum Curve {
    Quadratic,
    Linear,
    Sinus,
}

// Note
// The easer could have been much better done.
// I just wanted something that "worked".
// I recommend you use a library that provide better easing functions.

struct Easer {
    position: f32,
    end: f32,
    time: u32,
    frame_time: u32,
    curve: Curve,
}

impl Iterator for Easer {
    type Item = f32;
    fn next(&mut self) -> Option<Self::Item> {
        let cursor;
        if self.time == 0 {
            return None;
        }
        let frame = self.time / self.frame_time.max(1);
        match self.curve {
            Curve::Sinus => {
                self.position += PI / frame as f32;
                if self.position > PI {
                    cursor = self.end * (PI).sin().abs();
                    self.time = 0;
                } else {
                    cursor = self.end * (self.position).sin().abs();
                }
            }
            Curve::Linear => {
                self.position += self.end / frame as f32;
                cursor = self.position;
                if cursor > self.end {
                    self.time = 0;
                }
            }
            Curve::Quadratic => {
                let b = self.end;
                let h = b.sqrt();
                self.position += h * 2. / frame as f32;
                if self.position > 2. * h {
                    cursor = self.end - (2. * h - h).powi(2);
                    self.time = 0;
                } else {
                    cursor = self.end - (self.position - h).powi(2);
                }
            }
        }
        Some(cursor)
    }
}

impl Easer {
    fn new(start: f32, end: f32, time: u32, curve: Curve) -> Self {
        Easer {
            position: start,
            end,
            frame_time: 10,
            time,
            curve,
        }
    }
    fn frame_time(&mut self, frame_time: u32) {
        self.frame_time = frame_time;
    }
    fn reset(&mut self, time: u32) {
        self.time = time;
        self.frame_time = 10;
        self.position = 0.;
    }
}

struct Animate {
    start: bool,
    cursor: f32,
    easer: Easer,
}

impl Geometry for Animate {
    fn width(&self) -> f32 {
        400.
    }
    fn height(&self) -> f32 {
        30.
    }
}

impl Widget<AnimationState> for Animate {
    fn create_node(&mut self, x: f32, y: f32) -> scene::RenderNode {
        if self.start {
            if let Some(delta) = self.easer.next() {
                return Instruction::new(
                    x + delta,
                    y,
                    Rectangle::empty(self.cursor, 30.).background(style::RED),
                )
                .into();
            } else {
                self.start = false;
                self.easer.reset(1000);
            }
        }
        scene::RenderNode::None
    }
    fn sync<'d>(
        &'d mut self,
        ctx: &mut context::SyncContext<AnimationState>,
        event: &Event<AnimationState>,
    ) -> Damage {
        match event {
            Event::Callback(frame_time) => {
                if self.start {
                    self.easer.frame_time(*frame_time);
                    return Damage::Frame;
                } else {
                    ctx.request(AnimationState::Stop);
                }
            }
            Event::Message(msg) => {
                match msg.0 {
                    AnimationState::Start => {
                        self.start = true;
                        self.easer.end = self.width() - self.cursor;
                        return Damage::Frame;
                    }
                    AnimationState::Pause => {
                        self.start = false;
                    }
                    AnimationState::Stop => {
                        self.start = false;
                        self.easer.reset(1000);
                    }
                }
            },
            Event::Frame => {
                self.start = true;
                self.easer.end = self.width() - self.cursor;
                return Damage::Frame;
            }
            _ => {}
        }
        Damage::None
    }
}

impl Animate {
    fn new(curve: Curve) -> Self {
        Animate {
            start: false,
            cursor: 20.,
            easer: Easer::new(0., 0., 1000, curve),
        }
    }
}

fn ui() -> impl Widget<AnimationState> {
    let mut ui = WidgetLayout::new(0.).orientation(Orientation::Vertical);
    ui.add(Animate::new(Curve::Linear));
    ui.add(Animate::new(Curve::Sinus));
    ui.add(Animate::new(Curve::Quadratic));

    ui.add(
        Text::from(Label::default("Launch", 15.))
            .ext()
            .even_padding(5.)
            .background(style::BG1)
            .border(style::BG2, 2.)
            .button(move |this, ctx, p| match p {
                Pointer::MouseClick {
                    time: _,
                    button,
                    pressed,
                } => {
                    if button.is_left() && pressed {
                        if let Data::Request(state) = ctx.get(Message::new(AnimationState::Start, ())).unwrap() {
                            match state {
                                AnimationState::Start => {
                                    this.edit("Pause");
                                    ctx.request(AnimationState::Pause)
                                }
                                AnimationState::Pause | AnimationState::Stop => {
                                    this.edit("Run");
                                    ctx.request(AnimationState::Start)
                                }
                            }.unwrap();
                        }
                    }
                }
                _ => {}
            }),
    );
    ui.justify(CENTER);

    ui
}

fn main() {
    let (mut snui, mut event_loop) = Application::new(true);

    snui.create_inner_application(
        EaserCtl::default(),
        ui().ext()
            .background(style::BG0)
            .even_radius(5.)
            .border(style::BG2, 5.),
        event_loop.handle(),
        |_, _| {},
    );

    snui.run(&mut event_loop);
}
