use scene::Instruction;
use snui::controller::*;
use snui::wayland::shell::*;
use snui::widgets::container::*;
use snui::widgets::extra::{switch::*, Curve, Easer, Start};
use snui::widgets::shapes::*;
use snui::{widgets::*, *};

#[derive(Debug, Clone, Copy, PartialEq)]
enum AnimationState {
    Stop,
    Start,
    Pause,
}

impl IntoMessage<SwitchState> for AnimationState {
    fn into(&self, t: SwitchState) -> Self {
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
                    ctx.send(AnimationState::Stop);
                }
            }
            Event::Message(msg) => match msg {
                AnimationState::Start => {
                    self.start = true;
                    self.easer.set_max(self.width() - self.cursor);
                    return Damage::Frame;
                }
                AnimationState::Pause => {
                    self.start = false;
                }
                AnimationState::Stop => {
                    self.start = false;
                    self.easer.reset(1000);
                }
            },
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
            easer: Easer::new(Start::Min, 0., 1000, curve),
        }
    }
}

fn ui() -> impl Widget<AnimationState> {
    let mut ui = WidgetLayout::new(0.).orientation(Orientation::Vertical);
    ui.add(Animate::new(Curve::Linear));
    ui.add(Animate::new(Curve::Sinus));
    ui.add(Animate::new(Curve::Quadratic));

    ui.add(
        Switch::default()
            .message(AnimationState::Pause)
            .duration(200)
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
    let (mut snui, mut event_loop) = Application::new(true);

    snui.create_inner_application(
        EaserCtl::default(),
        ui().ext()
            .background(style::BG0)
            .even_radius(4.)
            .border(style::BG2, 5.),
        event_loop.handle(),
        |_, _| {},
    );

    snui.run(&mut event_loop);
}
