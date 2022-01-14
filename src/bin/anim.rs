use scene::Instruction;
use snui::wayland::shell::*;
use snui::controller::*;
use snui::widgets::window::*;
use snui::widgets::container::*;
use snui::widgets::extra::{switch::*, Quadratic, Sinus};
use snui::widgets::shapes::*;
use snui::{widgets::{*, text::*}, *};

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

impl TryInto<String> for AnimationState {
    type Error = ();
    fn try_into(self) -> Result<String, Self::Error> {
        Err(())
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
        match msg {
            AnimationState::Stop | AnimationState::Pause => self.block = false,
            _ => {}
        }
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

struct Animate {
    start: bool,
    cursor: f32,
    easer: Quadratic,
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
            }
        }
        scene::RenderNode::None
    }
    fn sync<'d>(
        &'d mut self,
        ctx: &mut context::SyncContext<AnimationState>,
        event: Event<AnimationState>,
    ) -> Damage {
        match event {
            Event::Callback(frame_time) => {
                let steps = (frame_time as usize * self.easer.steps()) / 7000;
                if self.start {
                    for _ in 1..steps {
                        if let None = self.easer.next() {
                            return Damage::None;
                        }
                    }
                    return Damage::Frame;
                } else {
                    ctx.send(AnimationState::Stop);
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

impl Animate {
    fn new() -> Self {
        Animate {
            start: false,
            cursor: 20.,
            easer: Quadratic::new(0., 0.5, 400. - 20.),
        }
    }
}

fn ui() -> impl Widget<AnimationState> {
    let mut ui = WidgetLayout::new(0.).orientation(Orientation::Vertical);
    ui.add(Animate::new());
    ui.add(Animate::new());

    ui.add(
        Switch::default()
            .message(AnimationState::Pause)
            .duration(600)
            .ext()
            .background(style::BG1)
            .even_radius(3.)
            .button(move |this, ctx, p| match p {
                Pointer::MouseClick {
                    time: _,
                    button,
                    pressed,
                } => {
                    if button.is_left() && pressed {
                        if let Ok(state) = ctx.get(&AnimationState::Start) {
                            match state {
                                AnimationState::Start => {
                                    this.set_background(style::BG1);
                                    ctx.send(AnimationState::Pause)
                                }
                                AnimationState::Pause | AnimationState::Stop => {
                                    this.set_background(style::RED);
                                    ctx.send(AnimationState::Start)
                                }
                            }
                            .unwrap();
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
        Label::default("Animation", 15.).into(),
        ui().clamp().ext().background(style::BG0),
    );

    client.new_window(
        EaserCtl::default(),
        window.background(style::BG2),
        &event_queue.handle()
    );

    let window = window::default_window(
        Label::default("Animation", 15.).into(),
        ui().clamp().ext().background(style::BG0),
    );

    client.new_window(
        EaserCtl::default(),
        window.background(style::BG2),
        &event_queue.handle()
    );

	loop {
        event_queue.blocking_dispatch(&mut client).unwrap();
	}
}
