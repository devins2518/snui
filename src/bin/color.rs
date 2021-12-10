use snui::context::*;
use snui::data::{Controller, Data, Message, ControllerError};
use snui::scene::*;
use snui::wayland::shell::*;
use snui::widgets::{shapes::*, text::*, *};
use snui::*;

const BG0: u32 = 0xff_25_22_21;
const BG1: u32 = 0xa0_30_2c_2b;
const BG2: u32 = 0xff_30_2c_2b;
const YEL: u32 = 0xff_d9_b2_7c;
const GRN: u32 = 0xff_95_a8_82;
const BLU: u32 = 0xff_72_87_97;
const ORG: u32 = 0xff_d0_8b_65;
const RED: u32 = 0xff_c6_5f_5f;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Signal {
    Null = 0,
    Source = 1 << 1,
    Red = 1 << 2,
    Green = 1 << 3,
    Blue = 1 << 4,
    Alpha = 1 << 5,
}

impl<'d> From<Signal> for Data<'d> {
    fn from(this: Signal) -> Self {
        Data::Uint(this as u32)
    }
}

#[derive(Clone, Copy, Debug)]
struct ColorBlock {
    width: f32,
    height: f32,
    color: snui::ColorU8,
}

impl Geometry for ColorBlock {
    fn height(&self) -> f32 {
        self.height
    }
    fn width(&self) -> f32 {
        self.width
    }
    fn set_width(&mut self, width: f32) -> Result<(), f32> {
        self.width = width.max(0.);
        Ok(())
    }
    fn set_height(&mut self, height: f32) -> Result<(), f32> {
        self.height = height.max(0.);
        Ok(())
    }
}

impl Widget for ColorBlock {
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        Rectangle::empty(self.width, self.height)
            .background(self.color)
            .radius(5., 5., 5., 5.)
            .create_node(x, y)
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext, event: Event) {
        if let Event::Message(_) = event {
            ctx.request_draw();
            let color = ctx
                .get(Message::new(Signal::Source as u32, Data::Null))
                .unwrap();
            if let Data::Uint(color) = color {
                self.color = u32_to_source(color).to_color_u8();
            }
        }
    }
}

struct Cross {}

impl Geometry for Cross {
    fn height(&self) -> f32 {
        25.
    }
    fn width(&self) -> f32 {
        25.
    }
    fn set_width(&mut self, _width: f32) -> Result<(), f32> {
        Err(self.width())
    }
    fn set_height(&mut self, _height: f32) -> Result<(), f32> {
        Err(self.height())
    }
}

impl Widget for Cross {
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        let mut canvas = self.create_canvas(x, y);

        let (w, h) = (2., 16.);

        let b = Rectangle::empty(self.width(), self.height())
            .background(BG1)
            .radius(3., 3., 3., 3.);

        let r = Rectangle::empty(w, h).background(RED);

        let (x, y) = ((self.width() - w) / 2., (self.height() - h) / 2.);

        canvas.draw(0., 0., b);
        canvas.draw_at_angle(x, y, r.clone(), 45.);
        canvas.draw_at_angle(x, y, r, -45.);

        canvas.finish()
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext, event: Event) {
        if let Event::Pointer(x, y, p) = event {
            if let Pointer::MouseClick {
                time: _,
                button,
                pressed,
            } = p
            {
                if self.contains(x, y)
                {
                    if button == MouseButton::Left && pressed {
                        let _ = ctx.send(Message::new(Signal::Null as u32, Data::Null));
                    }
                }
            }
        }
    }
}

struct Listener {
    id: u32,
    text: Text,
}

impl Geometry for Listener {
    fn width(&self) -> f32 {
        self.text.width()
    }
    fn height(&self) -> f32 {
        self.text.height()
    }
    fn set_width(&mut self, width: f32) -> Result<(), f32> {
        self.text.set_width(width)
    }
    fn set_height(&mut self, height: f32) -> Result<(), f32> {
        self.text.set_height(height)
    }
}

impl Widget for Listener {
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        self.text.create_node(x, y)
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext, event: Event) {
        match event {
            Event::Message(msg) => {
                let Message(obj, _) = msg;
                if obj == Signal::Source as u32 {
                    ctx.request_draw();
                    if let Ok(data) = ctx.get(Message::new(self.id, Data::Null)) {
                        match data {
                            Data::Byte(b) => self.text.edit(&b.to_string()),
                            Data::Uint(uint) => {
                                self.text.edit(&format!("{:#010X}", uint));
                            }
                            _ => {}
                        }
                    }
                }
            }
            Event::Commit => {
                self.text.edit(&format!("Welcome"));
            }
            _ => {}
        }
        self.text.sync(ctx, event);
    }
}

#[derive(Clone, Copy, Debug)]
struct ColorControl {
    signal: Option<Signal>,
    color: tiny_skia::Color,
}

impl Controller for ColorControl {
    fn serialize(&mut self, _msg: Message) -> Result<u32, ControllerError> {
        Err(data::ControllerError::NonBlocking)
    }
    fn deserialize(&mut self, _token: u32) -> Result<(), ControllerError> {
        Err(data::ControllerError::NonBlocking)
    }
    fn get<'m>(&'m self, msg: Message) -> Result<Data<'m>, ControllerError> {
        let Message(obj, _) = msg;
        Ok(if obj == Signal::Red as u32 {
            self.color.red().into()
        } else if obj == Signal::Green as u32 {
            self.color.green().into()
        } else if obj == Signal::Blue as u32 {
            self.color.blue().into()
        } else if obj == Signal::Alpha as u32 {
            self.color.alpha().into()
        } else if obj == Signal::Source as u32 {
            self.color.to_color_u8().get().into()
        } else {
            Data::Null
        })
    }
    fn send<'m>(&'m mut self, msg: data::Message) -> Result<Data<'m>, ControllerError> {
        let Message(obj, value) = msg;
        if obj == Signal::Null as u32 {
            self.signal = Some(Signal::Null);
            return Ok(Data::Null);
        }
        match value {
            Data::Byte(b) => {
                if obj == Signal::Red as u32 {
                    self.signal = Some(Signal::Red);
                    self.color.set_red(b as f32 / 255.);
                } else if obj == Signal::Green as u32 {
                    self.signal = Some(Signal::Green);
                    self.color.set_green(b as f32 / 255.);
                } else if obj == Signal::Blue as u32 {
                    self.signal = Some(Signal::Blue);
                    self.color.set_blue(b as f32 / 255.);
                } else if obj == Signal::Alpha as u32 {
                    self.signal = Some(Signal::Alpha);
                    self.color.set_alpha(b as f32 / 255.);
                }
            }
            Data::Float(f) => {
                if obj == Signal::Red as u32 {
                    self.color.set_red(f);
                    self.signal = Some(Signal::Red);
                    return Ok(self.color.to_color_u8().get().into());
                } else if obj == Signal::Green as u32 {
                    self.color.set_green(f);
                    self.signal = Some(Signal::Green);
                    return Ok(self.color.to_color_u8().get().into());
                } else if obj == Signal::Blue as u32 {
                    self.color.set_blue(f);
                    self.signal = Some(Signal::Blue);
                    return Ok(self.color.to_color_u8().get().into());
                } else if obj == Signal::Alpha as u32 {
                    self.color.set_alpha(f);
                    self.signal = Some(Signal::Alpha);
                    return Ok(self.color.to_color_u8().get().into());
                }
            }
            Data::Uint(source) => {
                if obj == Signal::Source as u32 {
                    self.signal = Some(Signal::Source);
                    self.color = u32_to_source(source);
                }
            }
            _ => {}
        }
        Ok(Data::Null)
    }
    fn sync(&mut self) -> Result<Message<'static>, ControllerError> {
        if let Some(signal) = self.signal {
            if signal != Signal::Null {
                self.signal = None;
                return Ok(Message::new(Signal::Source as u32, signal as u32));
            }
        }
        Err(data::ControllerError::WrongObject)
    }
}

fn main() {
    let (mut snui, mut event_loop) = Application::new(true);

    let mut editor = WidgetLayout::vertical(5);

    editor.add(header());
    editor.add(
        core()
        .wrap()
        .padding(20., 20., 20., 20.));
    editor.justify(CENTER);

    snui.create_inner_application(
        ColorControl {
            signal: None,
            color: u32_to_source(0),
        },
        editor
            .wrap()
            .background(BG0)
            .padding(15., 15., 15., 15.)
            .radius(5., 5., 5., 5.)
            .border(BG2, 1.),
        event_loop.handle(),
        |core, _| {
            if let Some(signal) = core.controller.signal {
                if signal == Signal::Null {
                    core.destroy();
                    std::process::exit(0);
                }
            }
        },
    );

    snui.run(&mut event_loop);
}

fn header() -> impl Widget {
    let mut buttons = WidgetLayout::horizontal(5);
    let text: Text = Label::default("Copy", 15.).into();
    let icon = Label::default("", 21.).font(FontProperty::new("CaskaydiaCove Nerd Font Mono"));

    buttons.add(
        icon.wrap()
            .background(BG2)
            .radius(3., 3., 3., 3.)
            .padding(8., 8., 8., 8.)
            .border(BG2, 1.)
            .into_button(|this, _, p| match p {
                Pointer::MouseClick {
                    time: _,
                    pressed,
                    button,
                } => {
                    if button == MouseButton::Left && pressed {
                        eprintln!("color picker missing");
                    }
                }
                Pointer::Enter => this.set_background(Background::solid(BG0)),
                Pointer::Leave => {
                    this.set_background(Background::solid(BG2));
                }
                _ => {}
            }),
    );
    buttons.add(
        text.wrap()
            .padding(8., 8., 8., 8.)
            .background(BG2)
            .radius(3., 3., 3., 3.)
            .border(BG2, 2.)
            .into_button(|this, ctx, p| match p {
                Pointer::MouseClick {
                    time: _,
                    pressed,
                    button,
                } => {
                    if button == MouseButton::Left && pressed {
                        if let Data::Uint(source) = ctx
                            .get(Message::new(Signal::Source as u32, Data::Null))
                            .unwrap()
                        {
                            this.edit("Copied");
                            this.set_background(Background::solid(BG1));
                            println!("#{:X}", source);
                        }
                    } else if button == MouseButton::Left {
                        this.edit("Copy");
                    }
                }
                Pointer::Enter => this.set_background(Background::solid(BG0)),
                Pointer::Leave => {
                    this.edit("Copy");
                    this.set_background(Background::solid(BG2));
                }
                _ => {}
            }),
    );
    let header = container::Centerbox::horizontal(
        buttons,
        Label::default("app_name", 15.),
        Cross {}
    ).align();
    header
}

fn sliders() -> WidgetLayout {
    let mut layout = WidgetLayout::vertical(10);

    for color in [RED, GRN, BLU, BG0] {
        let id = match color {
            RED => Signal::Red,
            BLU => Signal::Blue,
            GRN => Signal::Green,
            BG0 => Signal::Alpha,
            _ => Signal::Null,
        };
        let slider =
            widgets::slider::Slider::horizontal(id as u32, 200, 6, ShapeStyle::solid(color))
                .wrap()
                .background(BG2)
                .radius(3., 3., 3., 3.);

        layout.add(slider);
    }

    layout.justify(CENTER);

    layout
}

fn core() -> WidgetLayout {
    let mut layout = WidgetLayout::vertical(15);

    let mut listener = Listener {
        id: Signal::Source as u32,
        text: Label::default("", 17.).into(),
    }
    .into_box()
    .anchor(CENTER, START)
    .constraint(Constraint::Downward);

    let _ = listener.set_height(18.);
    let _ = listener.set_width(200.);

    let mut indicator = WidgetLayout::vertical(0);

    indicator.add(
        listener.wrap().padding(10., 10., 10., 10.)
    );
    indicator.add(ColorBlock {
        width: 200.,
        height: 200.,
        color: Color::from_rgba(0., 0., 0., 0.5).unwrap().to_color_u8()
    });
    indicator.justify(CENTER);

    layout.add(indicator);
    layout.add(sliders());
    layout.justify(CENTER);

    layout
}
