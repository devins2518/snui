use scene::Instruction;
use snui::controller::*;
use snui::wayland::shell::*;
use snui::widgets::container::*;
use snui::widgets::extra::{switch::*, Easer, Quadratic, Sinus};
use snui::widgets::shapes::*;
use snui::{
    widgets::{text::*, *},
    *,
};

#[derive(Debug, Clone, Copy, PartialEq)]
enum AnimationState {
    Stop,
    Start,
    Pause,
}

impl FromArg<SwitchState> for AnimationState {
    fn from_arg(&self, t: SwitchState) -> Self {
        match t {
            SwitchState::Activated => AnimationState::Start,
            SwitchState::Deactivated => AnimationState::Pause,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct EaserCtl {
    block: bool,
    state: AnimationState,
}

impl Default for EaserCtl {
    fn default() -> Self {
        EaserCtl {
            block: false,
            state: AnimationState::Stop,
        }
    }
}

impl Controller<AnimationState> for EaserCtl {
    fn get(&self, _msg: &AnimationState) -> Result<AnimationState, ControllerError> {
        return Ok(self.state);
    }
    fn send(&mut self, msg: AnimationState) -> Result<AnimationState, ControllerError> {
        self.block = false;
        self.state = msg;
        Ok(self.state)
    }
    fn sync<'s>(&mut self) -> Result<AnimationState, ControllerError> {
        if !self.block {
            match self.state {
                AnimationState::Stop => return Err(ControllerError::Waiting),
                AnimationState::Start => {
                    self.block = true;
                    return Ok(self.state);
                }
                AnimationState::Pause => {
                    self.block = true;
                    return Ok(self.state);
                }
            }
        }
        Err(ControllerError::Blocking)
    }
}

struct Animate<E: Easer> {
    start: bool,
    cursor: f32,
    position: f32,
    easer: E,
}

impl<E: Easer> Geometry for Animate<E> {
    fn width(&self) -> f32 {
        400.
    }
    fn height(&self) -> f32 {
        30.
    }
}

impl<E: Easer> Widget<AnimationState> for Animate<E> {
    fn create_node(&mut self, x: f32, y: f32) -> scene::RenderNode {
        let node = Instruction::new(
            x + self.position,
            y,
            Rectangle::empty(self.cursor, 30.).background(style::RED),
        )
        .into();

        scene::RenderNode::Clip {
            region: scene::Region::new(x + 50., y, self.width() - 100., self.height()),
            node: Box::new(node),
        }
    }
    fn sync<'d>(
        &'d mut self,
        ctx: &mut context::SyncContext<AnimationState>,
        event: Event<AnimationState>,
    ) -> Damage {
        match event {
            Event::Callback(frame_time) => {
                if self.start {
                    let steps = (frame_time * self.easer.steps() as u32) as usize / 5000;
                    for _ in 0..steps {
                        match self.easer.next() {
                            Some(position) => self.position = position,
                            None => {
                                ctx.send(AnimationState::Stop).unwrap();
                                self.start = false;
                                return Damage::None;
                            }
                        }
                    }
                    return Damage::Frame;
                }
            }
            Event::Message(msg) => match msg {
                AnimationState::Start => {
                    self.start = true;
                    return Damage::Frame;
                }
                AnimationState::Pause => {
                    self.start = false;
                }
                AnimationState::Stop => {
                    self.start = false;
                }
            },
            _ => {}
        }
        Damage::None
    }
}

impl Animate<Quadratic> {
    fn quadratic() -> Self {
        Animate {
            position: 0.,
            start: false,
            cursor: 20.,
            easer: Quadratic::new(0., 1., 400. - 20.),
        }
    }
}

impl Animate<Sinus> {
    fn sinus() -> Self {
        Animate {
            position: 0.,
            start: false,
            cursor: 20.,
            easer: Sinus::new(0., 1., 400. - 20.),
        }
    }
}

struct FrameRate {
    text: Text,
}

impl Geometry for FrameRate {
    fn width(&self) -> f32 {
        self.text.width()
    }
    fn height(&self) -> f32 {
        self.text.height()
    }
}

impl<M> Widget<M> for FrameRate {
    fn create_node(&mut self, x: f32, y: f32) -> scene::RenderNode {
        self.text.create_node(x, y)
    }
    fn sync<'d>(&'d mut self, ctx: &mut context::SyncContext<M>, event: Event<'d, M>) -> Damage {
        match event {
            Event::Callback(frame_time) => {
                if frame_time > 0 {
                    let frame_rate = 1000 / frame_time;
                    self.text.edit(frame_rate);
                    self.text.sync(ctx, event)
                } else {
                    self.text.sync(ctx, event)
                }
            }
            _ => self.text.sync(ctx, event),
        }
    }
}

fn ui() -> impl Widget<AnimationState> {
    let mut ui = WidgetLayout::new(0.).orientation(Orientation::Vertical);
    ui.add(
        FrameRate {
            text: "frame rate".into(),
        }
        .clamp()
        .with_size(40., 20.),
    );
    ui.add(Animate::quadratic());
    ui.add(Animate::sinus());

    ui.add(
        Switch::default()
            .message(AnimationState::Pause)
            .duration(600)
            .ext()
            .background(style::BG1)
            .even_radius(3.)
            .button(move |this, ctx, p| match p {
                Pointer::MouseClick {
                    serial: _,
                    button,
                    pressed,
                } => {
                    if button.is_left() && pressed {
                        if let Ok(state) = ctx.get(&AnimationState::Start) {
                            match state {
                                AnimationState::Start => {
                                    this.set_background(style::BG1);
                                }
                                AnimationState::Pause | AnimationState::Stop => {
                                    this.set_background(style::RED);
                                }
                            }
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
    let (mut client, mut event_queue) = WaylandClient::new().unwrap();

    let window = window::default_window(
        Label::from("Animation"),
        ui().clamp().ext().background(style::BG0),
    );

    client.new_window(
        EaserCtl::default(),
        window
            .background(style::BG2)
            .alternate_background(0xff58514F)
            .border(style::BG2, 2.),
        &event_queue.handle(),
    );

    while client.has_client() {
        event_queue.blocking_dispatch(&mut client).unwrap();
    }
}
