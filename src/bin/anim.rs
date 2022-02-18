use snui::context::SyncContext;
use snui::mail::*;
use snui::wayland::backend::*;
use snui::widgets::extra::{switch::*, Easer, Quadratic, Sinus};

use snui::widgets::shapes::*;
use snui::{
    widgets::{label::*, layout::flex::Flex},
    *,
};

// The state of the animations in the Demo
#[derive(Debug, Clone, Copy, PartialEq)]
enum AnimationState {
    Stop,
    Start,
    Pause,
}

// Our Data.
// Holds the state and is responsible for communicating changes across the widget tree
#[derive(Debug, Clone, Copy, PartialEq)]
struct Demo {
    sync: bool,
    state: AnimationState,
}

// The message sent to the Demo when the want to change its animation state
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

// The Switch will use this trait to change the state of our Demo
impl Mail<Remote, bool, bool> for Demo {
    fn get(&self, _: Remote) -> Option<bool> {
        None
    }
    fn send(&mut self, _: Remote, data: bool) -> Option<bool> {
        // When the state of the animation is changed,
        // we want our Data to be shared again to the widgets so they are aware of the new state
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
            // Demo.sync is reset to false if true
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

// Moves a rectangle across a box
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
    fn draw_scene(&mut self, scene: scene::Scene) {
        scene
            .translate(self.position, 0.)
            .insert_primitive(&Rectangle::new(self.cursor, 30.).texture(theme::RED))
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
                                return Damage::Partial;
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
    fn layout(&mut self, _ctx: &mut context::LayoutCtx, _constraints: &BoxConstraints) -> Size {
        (self.width(), self.height()).into()
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

// Displays the frame rate of the animation
struct FrameRate {
    label: Label,
}

impl<D> Widget<D> for FrameRate {
    fn draw_scene(&mut self, scene: scene::Scene) {
        Widget::<()>::draw_scene(&mut self.label, scene)
    }
    fn sync<'d>(&'d mut self, ctx: &mut context::SyncContext<D>, event: Event<'d>) -> Damage {
        match event {
            Event::Callback(frame_time) => {
                if frame_time > 0 {
                    let frame_rate = 1000 / frame_time;
                    self.label.edit(frame_rate);
                }
                self.label.sync(ctx, event)
            }
            _ => self.label.sync(ctx, event),
        }
    }
    fn layout(&mut self, ctx: &mut context::LayoutCtx, constraints: &BoxConstraints) -> Size {
        Widget::<()>::layout(&mut self.label, ctx, constraints)
    }
}

// Creates our user interface
fn ui() -> Flex<Box<dyn Widget<Demo>>> {
    Flex::default()
        .with(
            FrameRate {
                label: "frame rate".into(),
            }
            .with_min_height(20.),
        )
        .with(Animate::quadratic().clamp())
        .with(Animate::sinus().clamp())
        .with(
            Switch::new(Remote {})
                .texture(theme::BG0)
                .duration(600)
                .background(theme::BG1)
                .radius(3.)
                .button(move |this, ctx: &mut SyncContext<Demo>, p| {
                    if p.left_button_click().is_some() {
                        match ctx.state {
                            AnimationState::Start => {
                                this.set_texture(theme::BG1);
                            }
                            AnimationState::Pause | AnimationState::Stop => {
                                this.set_texture(theme::RED);
                            }
                        }
                    }
                })
                .clamp(),
        )
        .orientation(Orientation::Vertical)
}

fn main() {
    let (mut client, mut event_queue) = WaylandClient::new().unwrap();

    client.new_window(
        Demo::default(),
        ui().clamp().background(theme::BG0).radius(5.),
        &event_queue.handle(),
    );

    while client.has_view() {
        event_queue.blocking_dispatch(&mut client).unwrap();
    }
}
