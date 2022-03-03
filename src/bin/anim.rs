use snui::context::{UpdateContext, WindowHandle};
use snui::mail::*;
use snui::wayland::backend::*;
use snui::widgets::extra::{switch::*, window, Easer, Quadratic, Sinus};

use snui::widgets::shapes::*;
use snui::{
    widgets::{label::*, layout::flex::Flex},
    *,
};

use scene::LinearGradient;
use tiny_skia::GradientStop;

// The state of the animations in the Demo
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
impl Mail<'_, Remote, bool, bool> for Demo {
    fn get(&self, _: Remote) -> Option<bool> {
        None
    }
    fn send(&mut self, _: Remote, data: bool) -> Option<bool> {
        match data {
            true => self.start(),
            false => self.pause(),
        }
        None
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
    cursor: Rectangle,
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
            .insert_primitive(&self.cursor)
    }
    fn update<'s>(&'s mut self, ctx: &mut UpdateContext<Demo>) -> Damage {
        match ctx.state {
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
        }
        Damage::None
    }
    fn event<'s>(&'s mut self, ctx: &mut UpdateContext<Demo>, event: Event<'s>) -> Damage {
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
            _ => {}
        }
        Damage::None
    }
    fn layout(&mut self, _ctx: &mut context::LayoutCtx, _constraints: &BoxConstraints) -> Size {
        self.size()
    }
}

impl Animate<Quadratic> {
    fn quadratic() -> Self {
        Animate {
            position: 0.,
            start: false,
            cursor: Rectangle::new(20., 30.).texture(theme::RED),
            easer: Quadratic::new(0., 1., 400. - 20.),
        }
    }
}

impl Animate<Sinus> {
    fn sinus() -> Self {
        Animate {
            position: 0.,
            start: false,
            cursor: Rectangle::new(20., 30.).texture(theme::RED),
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
    fn update<'s>(&'s mut self, ctx: &mut UpdateContext<D>) -> Damage {
        self.label.update(ctx)
    }
    fn event<'s>(&'s mut self, ctx: &mut UpdateContext<D>, event: Event<'s>) -> Damage {
        match event {
            Event::Callback(frame_time) => {
                if frame_time > 0 {
                    let frame_rate = 1000 / frame_time;
                    self.label.edit(&frame_rate.to_string());
                }
                self.label.event(ctx, event)
            }
            _ => self.label.event(ctx, event),
        }
    }
    fn layout(&mut self, ctx: &mut context::LayoutCtx, constraints: &BoxConstraints) -> Size {
        Widget::<()>::layout(&mut self.label, ctx, constraints)
    }
}

// Creates our user interface
fn ui() -> Flex<Box<dyn Widget<Demo>>> {
    Flex::column()
        .with_child(
            FrameRate {
                label: "frame rate".into(),
            }
            .with_min_height(20.),
        )
        .with_child(Animate::quadratic().clamp())
        .with_child(Animate::sinus().clamp())
        .with_child(
            Switch::default(Remote {})
                .texture(theme::BG0)
                .duration(600)
                .background(theme::BG1)
                .padding(2.)
                .radius(4.)
                .button(move |this, ctx: &mut UpdateContext<Demo>, p| {
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
}

fn main() {
    let window = window::default_window(Label::new("Animation"), ui());
    WaylandClient::init(|client, conn, qh| {
        client
            .create_window(
                Demo::default(),
                window
                    .decoration(theme::BG2, 2.)
                    .alternate_decoration(LinearGradient::new(vec![
                        GradientStop::new(0., to_color(theme::BLUE)),
                        GradientStop::new(1., to_color(theme::PURPLE)),
                    ]))
                    .texture(theme::BG0)
                    .radius(5.),
                conn,
                qh,
            )
            .set_title("Animation".to_string())
    })
}
