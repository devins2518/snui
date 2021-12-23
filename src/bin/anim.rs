use snui::wayland::shell::*;
use snui::data::*;
use scene::Instruction;
use snui::widgets::shapes::*;
use snui::widgets::container::*;
use std::f32::consts::PI;
use snui::{*, widgets::{*, text::*}};

#[derive(Debug, Clone, Copy)]
enum Request {
    Stop = 0,
    Start = 1,
    Pause = 2
}

impl From<u32> for Request {
    fn from(uint: u32) -> Self {
        match uint {
            0 => Request::Stop,
            1 => Request::Start,
            2 => Request::Pause,
            _ => panic!("invalid value")
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct EaserCtl {
    request: Option<Request>,
}

impl Default for EaserCtl {
    fn default() -> Self {
        EaserCtl {
            request: None
        }
    }
}

impl Controller for EaserCtl {
    fn serialize(&mut self, _msg: Message) -> Result<u32, ControllerError> {
        Err(ControllerError::NonBlocking)
    }
    fn deserialize(&mut self, _token: u32) -> Result<(), ControllerError> {
        Err(ControllerError::NonBlocking)
    }
    fn get<'m>(&'m self, _msg: Message) -> Result<Data<'m>, ControllerError> {
        if let Some(request) = self.request {
            return Ok(Data::from(request as u32))
        }
        Err(ControllerError::WrongObject)
    }
    fn send<'m>(&'m mut self, msg: Message) -> Result<Data<'m>, ControllerError> {
        let Message(request, _) = msg;
        match request {
            0 => self.request = Some(Request::Stop),
            1 => self.request = Some(Request::Start),
            2 => self.request = Some(Request::Stop),
            _ => return Err(ControllerError::WrongObject)
        }
        Ok(Data::Null)
    }
    fn sync(&mut self) -> Result<Message<'static>, ControllerError> {
        if let Some(request) = self.request {
            self.request = None;
            return Ok(Message::new(request as u32, Data::Null));
        }
        Err(ControllerError::NonBlocking)
    }
}

enum Curve {
    Quadratic,
    Linear,
    Sinus
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
    curve: Curve
}

impl Iterator for Easer {
    type Item = f32;
    fn next(&mut self) -> Option<Self::Item> {
        let cursor;
        let frame = self.time / self.frame_time;
        match self.curve {
            Curve::Sinus => {
                self.position += PI / frame as f32;
                cursor = self.end * (self.position).sin().abs();
                if self.position > PI { return None }
            }
            Curve::Linear => {
                self.position += self.end / frame as f32;
                cursor = self.position;
                if cursor > self.end { return None }
            }
            Curve::Quadratic => {
                let b = self.end;
                let h = b.sqrt();
                self.position += h * 2. / frame as f32;
                cursor = self.end - (self.position - h).powi(2);
                if self.position > 2. * h { return None }
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
    fn fps(&mut self, frame_time: u32) {
        self.frame_time = frame_time;
    }
    fn reset(&mut self) {
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

impl Widget for Animate {
    fn create_node(&mut self, x: f32, y: f32) -> scene::RenderNode {
        if self.start {
            if let Some(delta) = self.easer.next() {
                return Instruction::new(
                    x + delta,
                    y,
                    Rectangle::empty(self.cursor, 30.)
                    .background(style::RED)
                ).into()
            } else {
                self.start = false;
                self.easer.reset();
            }
        }
        scene::RenderNode::None
    }
    fn sync<'d>(&'d mut self, _ctx: &mut context::SyncContext, event: Event) -> Damage {
        match event {
            Event::Callback(frame_time) => if self.start {
                self.easer.fps(frame_time);
                return Damage::Frame;
            }
            Event::Message(msg) => {
                match Request::from(msg.0) {
                    Request::Start => {
                        self.start = true;
                        self.easer.end = self.width() - self.cursor;
                        return Damage::Frame;
                    }
                    Request::Pause => {
                        self.start = false;
                    }
                    Request::Stop => {
                        self.start = false;
                        self.easer.reset();
                    }
                }
            }
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
            easer: Easer::new(0., 0., 1000, curve)
        }
    }
}

fn ui() -> impl Widget {
    let mut ui =
    	WidgetLayout::new(0.)
    	.orientation(Orientation::Vertical);
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
            Pointer::MouseClick { time:_, button, pressed } => {
                if button.is_left() && pressed {
                    if ctx.send(Message::new(Request::Start as u32, Data::Null)).is_ok() {
                        this.set_background(style::RED);
                    }
                } else if button.is_left() {
                    this.set_background(style::BG1);
                }
            }
            _ => {}
        })
    );
    ui.justify(CENTER);

	ui
}

fn main() {
    let (mut snui, mut event_loop) = Application::new(true);

    snui.create_inner_application(
        EaserCtl::default(),
        ui()
    	.ext()
    	.background(style::BG0)
    	.even_radius(5.)
    	.border(style::BG2, 5.),
        event_loop.handle(),
        |_, _| {},
    );

    snui.run(&mut event_loop);
}
