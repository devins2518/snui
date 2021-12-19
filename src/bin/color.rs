use snui::context::*;
use snui::data::{Controller, ControllerError, Data, Message};
use snui::scene::*;
use snui::wayland::shell::*;
use snui::widgets::{shapes::*, text::*, *};
use snui::{style::*, *};

#[derive(Clone, Copy, Debug, PartialEq)]
enum Signal {
    Close = 0,
    Source = 1 << 1,
    Red = 1 << 2,
    Green = 1 << 3,
    Blue = 1 << 4,
    Alpha = 1 << 5,
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
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext, event: Event) -> Damage {
        match event {
            Event::Message(msg) => {
                let Message(obj, _) = msg;
                if obj == Signal::Source as u32 {
                    if let Ok(data) = ctx.get(Message::new(self.id, Data::Null)) {
                        match data {
                            Data::Byte(b) => self.text.edit(&b.to_string()),
                            Data::Uint(uint) => {
                                self.text
                                    .edit(format!("{:#010X}", uint).replace("0x", "#").as_str());
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
        self.text.sync(ctx, event)
    }
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
            .even_radius(5.)
            .create_node(x, y)
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext, event: Event) -> Damage {
        if let Event::Message(_) = event {
            let color = ctx
                .get(Message::new(Signal::Source as u32, Data::Null))
                .unwrap();
            if let Data::Uint(color) = color {
                self.color = u32_to_source(color).to_color_u8();
            }
            return Damage::Some;
        }
        Damage::None
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
            .even_radius(3.);

        let r = Rectangle::empty(w, h).background(RED);

        let (x, y) = ((self.width() - w) / 2., (self.height() - h) / 2.);

        canvas.draw(0., 0., b);
        canvas.draw_at_angle(x, y, r.clone(), 45.);
        canvas.draw_at_angle(x, y, r, -45.);

        canvas.finish()
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext, event: Event) -> Damage {
        if let Event::Pointer(x, y, p) = event {
            if let Pointer::MouseClick {
                time: _,
                button,
                pressed,
            } = p
            {
                if self.contains(x, y) {
                    if button == MouseButton::Left && pressed {
                        let _ = ctx.send(Message::new(Signal::Close as u32, Data::Null));
                    }
                }
            }
        }
        Damage::None
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
        if obj == Signal::Close as u32 {
            self.signal = Some(Signal::Close);
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
            if signal != Signal::Close {
                self.signal = None;
                return Ok(Message::new(Signal::Source as u32, signal as u32));
            }
        }
        Err(data::ControllerError::WrongObject)
    }
}

fn main() {
    let (mut snui, mut event_loop) = Application::new(true);

    let mut color = WidgetLayout::new(5.).orientation(Orientation::Vertical);

    color.add(header());
    color.add(core().pad(20.));
    color.justify(CENTER);

    snui.create_inner_application(
        ColorControl {
            signal: None,
            color: Color::from_rgba(0.5, 0.5, 0.5, 0.5).unwrap(),
        },
        color
            .ext()
            .background(BG0)
            .even_padding(15.)
            .border(BG2, 2.)
            .even_radius(5.),
        event_loop.handle(),
        |core, _| {
            if let Some(signal) = core.controller.signal {
                if signal == Signal::Close {
                    core.destroy();
                    std::process::exit(0);
                }
            }
        },
    );

    snui.run(&mut event_loop);
}

fn header() -> impl Widget {
    let mut buttons = WidgetLayout::new(5.);
    let text: Text = Label::default("Copy", 15.).into();
    let icon = Label::new("", 21.)
        .color(YEL)
        .font(FontProperty::new("CaskaydiaCove Nerd Font Mono"));
    buttons.add(
        icon.clamp()
            .constraint(Constraint::Downward)
            .size(25., 25.)
            .ext()
            .background(BG2)
            .even_radius(3.)
            .border(BG2, 2.)
            .button(|this, _, p| match p {
                Pointer::MouseClick {
                    time: _,
                    pressed,
                    button,
                } => {
                    if button == MouseButton::Left && pressed {
                        eprintln!("color picker missing");
                    }
                }
                Pointer::Enter => this.set_background(BG0),
                Pointer::Leave => {
                    this.set_background(BG2);
                }
                _ => {}
            }),
    );

    buttons.add(
        text.clamp()
            .constraint(Constraint::Downward)
            .size(40., 25.)
            .ext()
            .background(BG2)
            .even_radius(3.)
            .even_padding(2.)
            .border(BG2, 2.)
            .button(|this, ctx, p| match p {
                Pointer::MouseClick {
                    time: _,
                    pressed,
                    button,
                } => {
                    if button == MouseButton::Left && pressed {
                        if let Data::Uint(_) = ctx.request(Signal::Source as u32).unwrap() {
                            this.edit("Copied");
                            this.set_background(Background::solid(BG1));
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

    let mut header =
        CenterBox::from(buttons, Label::default("app_name", 15.), Cross {}).with_width(330.);

    let _ = header.set_width(300.);
    header
}

fn sliders() -> WidgetLayout {
    [RED, GRN, BLU, BG0]
        .iter()
        .map(|color| {
            let id = match *color {
                RED => Signal::Red,
                BLU => Signal::Blue,
                GRN => Signal::Green,
                BG0 => Signal::Alpha,
                _ => Signal::Close,
            };
            widgets::slider::Slider::new(200, 8)
                .id(id as u32)
                .background(*color)
                .ext()
                .background(BG2)
                .even_radius(3.)
                .child()
        })
        .collect::<WidgetLayout>()
        .spacing(10.)
        .orientation(Orientation::Vertical)
}

fn core() -> WidgetLayout {
    let mut layout = WidgetLayout::new(15.).orientation(Orientation::Vertical);

    let listener = Listener {
        id: Signal::Source as u32,
        text: "Welcome".into(),
    }
    .clamp()
    .with_size(200., 22.)
    .anchor(CENTER, START)
    .constraint(Constraint::Downward);

    let mut indicator = WidgetLayout::new(0.).orientation(Orientation::Vertical);

    indicator.add(listener.ext().padding(10., 10., 10., 10.));
    indicator.add(ColorBlock {
        width: 200.,
        height: 200.,
        color: Color::from_rgba(0.5, 0.5, 0.5, 0.5).unwrap().to_color_u8(),
    });
    indicator.justify(CENTER);

    layout.add(indicator);
    layout.add(sliders());
    layout.justify(CENTER);

    layout
}
