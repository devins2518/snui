// use snui::wayland::shell::*;
use snui::data::*;
use scene::Instruction;
use snui::widgets::shapes::*;
use snui::widgets::container::*;
use std::f32::consts::PI;
use snui::{*, widgets::{*, text::*}};

#[derive(Debug, Clone, Copy)]
enum Request {
    Stop,
    Start,
    Pause
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

impl Controller<Request> for EaserCtl {
    fn serialize(&mut self, _msg: Message<Request>) -> Result<u32, ControllerError> {
        Err(ControllerError::NonBlocking)
    }
    fn deserialize(&mut self, _token: u32) -> Result<(), ControllerError> {
        Err(ControllerError::NonBlocking)
    }
    fn get<'c>(&'c self, msg: Message<Request>) -> Result<Data<'c, Request>, ControllerError> {
        if let Some(request) = self.request {
            return Ok(Data::from(request as u32))
        }
        Err(ControllerError::WrongObject)
    }
    fn send<'c>(&'c mut self, msg: Message<Request>) -> Result<Data<'c, Request>, ControllerError> {
        let Message(request, _) = msg;
        self.request = Some(request);
        Err(ControllerError::WrongObject)
    }
    fn sync(&mut self) -> Result<Message<'static, Request>, ControllerError> {
        if let Some(request) = self.request {
            self.request = None;
            return Ok(Message::new(request, ()));
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

impl Widget<Request> for Animate {
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
                self.easer.reset(1000);
            }
        }
        scene::RenderNode::None
    }
    fn sync<'d>(&'d mut self, _ctx: &mut context::SyncContext<Request>, event: &Event<Request>) -> Damage {
        match event {
            Event::Callback(frame_time) => if self.start {
                self.easer.frame_time(*frame_time);
                return Damage::Frame;
            }
            Event::Message(msg) => {
                match msg.0 {
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
                        self.easer.reset(1000);
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

fn ui() -> impl Widget<Request> {
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
                    if ctx.send(Message::new(Request::Start, ())).is_ok() {
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
    // let (mut snui, mut event_loop) = Application::new(true);

    // snui.create_inner_application(
    //     EaserCtl::default(),
    //     ui()
    // 	.ext()
    // 	.background(style::BG0)
    // 	.even_radius(5.)
    // 	.border(style::BG2, 5.),
    //     event_loop.handle(),
    //     |_, _| {},
    // );

    // snui.run(&mut event_loop);
}
