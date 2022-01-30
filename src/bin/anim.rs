use scene::Instruction;
use snui::data::*;
use snui::wayland::shell::*;
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

#[derive(Debug, Clone, Copy, PartialEq)]
struct Demo {
    sync: bool,
    state: AnimationState,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Remote {}

impl Default for Demo {
    fn default() -> Self {
        Demo {
            sync: false,
            state: AnimationState::Stop,
        }
    }
}

impl Post<Remote, bool, bool> for Demo {
    fn get(&self, _: Remote) -> Option<bool> {
        None
    }
    fn send(&mut self, _: Remote, data: bool) -> Option<bool> {
        self.sync = true;
        match data {
            true => self.start(),
            false => self.pause(),
        }
        None
    }
}

impl Data for Demo {
    fn sync(&mut self) -> bool {
        if self.sync {
            self.sync = false;
            true
        } else {
            false
        }
    }
}

impl Demo {
    fn start(&mut self) {
        self.state = AnimationState::Start
    }
    fn pause(&mut self) {
        self.state = AnimationState::Pause
    }
    fn stop(&mut self) {
        self.state = AnimationState::Stop
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

impl<E: Easer> Widget<Demo> for Animate<E> {
    fn create_node(&mut self, transform: Transform) -> scene::RenderNode {
        Instruction::new(
            transform.pre_translate(self.position, 0.),
            Rectangle::empty(self.cursor, 30.).background(theme::RED),
        )
        .into()
    }
    fn sync<'d>(&'d mut self, ctx: &mut context::SyncContext<Demo>, event: Event) -> Damage {
        match event {
            Event::Callback(frame_time) => {
                if self.start {
                    let steps = (frame_time * self.easer.steps() as u32) as usize / 5000;
                    for _ in 0..steps {
                        match self.easer.next() {
                            Some(position) => self.position = position,
                            None => {
                                ctx.stop();
                                self.start = false;
                                return Damage::None;
                            }
                        }
                    }
                    return Damage::Frame;
                }
            }
            Event::Sync => match ctx.state {
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

impl<D> Widget<D> for FrameRate {
    fn create_node(&mut self, transform: Transform) -> scene::RenderNode {
        Widget::<()>::create_node(&mut self.text, transform)
    }
    fn sync<'d>(&'d mut self, ctx: &mut context::SyncContext<D>, event: Event<'d>) -> Damage {
        match event {
            Event::Callback(frame_time) => {
                if frame_time > 0 {
                    let frame_rate = 1000 / frame_time;
                    self.text.edit(frame_rate);
                }
                self.text.sync(ctx, event)
            }
            _ => self.text.sync(ctx, event),
        }
    }
}

fn ui() -> impl Widget<Demo> {
    let mut ui = SimpleLayout::new(0.).orientation(Orientation::Vertical);
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
        Switch::new(Remote {})
            .duration(600)
            .style()
            .background(theme::BG1)
            .even_radius(3.)
            .button::<Demo, _>(move |this, ctx, p| match p {
                Pointer::MouseClick {
                    serial: _,
                    button,
                    pressed,
                } => {
                    if button.is_left() && pressed {
                        match ctx.state {
                            AnimationState::Start => {
                                this.set_background(theme::BG1);
                            }
                            AnimationState::Pause | AnimationState::Stop => {
                                this.set_background(theme::RED);
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
        ui().clamp().style().background(theme::BG0),
    );

    client.new_window(
        Demo::default(),
        window
            .background(theme::BG2)
            .alternate_background(0xff58514F)
            .border(theme::BG2, 2.),
        &event_queue.handle(),
    );

    while client.has_application() {
        event_queue.blocking_dispatch(&mut client).unwrap();
    }
}
