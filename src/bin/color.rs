use snui::context::*;
use snui::data::{Controller, ControllerError, Data, Message};
use snui::scene::*;
use snui::wayland::shell::*;
use snui::widgets::{shapes::*, text::*, *};
use snui::{style::*, *};

#[derive(Clone, Copy, Debug)]
struct ColorControl {
    signal: Option<Request>,
    color: tiny_skia::Color,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Format {
    Hex,
    Uint,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Request {
    Close,
    Source(Format),
    Red,
    Green,
    Blue,
    Alpha,
}

impl Controller<Request> for ColorControl {
    fn serialize(&mut self, _msg: Message<Request>) -> Result<u32, ControllerError> {
        Err(data::ControllerError::WrongObject)
    }
    fn deserialize(&mut self, _token: u32) -> Result<(), ControllerError> {
        Err(data::ControllerError::WrongObject)
    }
    fn get<'c>(&'c self, msg: Message<Request>) -> Result<Data<'c, Request>, ControllerError> {
        let Message(signal, _) = msg;
        match signal {
            Request::Alpha => return Ok(self.color.alpha().into()),
            Request::Red => return Ok(self.color.red().into()),
            Request::Green => return Ok(self.color.green().into()),
            Request::Blue => return Ok(self.color.blue().into()),
            Request::Source(format) => {
                let color = self.color.to_color_u8().get();
                match format {
                    Format::Uint => return Ok(color.into()),
                    Format::Hex => return Ok(format!("{:#010X}", color).replace("0x", "#").into()),
                }
            }
            _ => {}
        }
        Err(data::ControllerError::WrongObject)
    }
    fn send<'c>(&'c mut self, msg: Message<Request>) -> Result<Data<'c, Request>, ControllerError> {
        let Message(signal, value) = msg;
        match value {
            Data::Float(f) => match signal {
                Request::Alpha => self.color.set_alpha(f),
                Request::Red => self.color.set_red(f),
                Request::Green => self.color.set_green(f),
                Request::Blue => self.color.set_blue(f),
                Request::Close => {}
                _ => return Err(ControllerError::WrongObject),
            },
            Data::Uint(color) => {
                self.color = u32_to_source(color);
            }
            _ => return Err(ControllerError::WrongObject),
        }
        self.signal = Some(signal);
        Ok(Data::Null)
    }
    fn sync(&mut self) -> Result<Message<'static, Request>, ControllerError> {
        if let Some(signal) = self.signal {
            if signal != Request::Close {
                self.signal = None;
                return Ok(Message::new(Request::Source(Format::Uint), ()));
            }
        }
        Err(data::ControllerError::WrongObject)
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

impl Widget<Request> for ColorBlock {
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        Instruction::new(
            x,
            y,
            Rectangle::empty(self.width, self.height)
                .background(self.color)
                .even_radius(5.),
        )
        .into()
    }
    fn sync<'d>(
        &'d mut self,
        ctx: &mut SyncContext<Request>,
        event: &'d Event<'d, Request>,
    ) -> Damage {
        if let Event::Message(_) = event {
            let color = ctx
                .get(Message::new(Request::Source(Format::Uint), ()))
                .unwrap();
            if let Data::Uint(color) = color {
                self.color = u32_to_source(color).to_color_u8();
            }
            return Damage::Some;
        }
        Damage::None
    }
}

// This is essentially the close button
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

impl Widget<Request> for Cross {
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
    fn sync<'d>(
        &'d mut self,
        ctx: &mut SyncContext<Request>,
        event: &'d Event<'d, Request>,
    ) -> Damage {
        if let Event::Pointer(x, y, p) = *event {
            if let Pointer::MouseClick {
                time: _,
                button,
                pressed,
            } = p
            {
                if self.contains(x, y) {
                    if button.is_left() && pressed {
                        let _ = ctx.request(Request::Close);
                    }
                }
            }
        }
        Damage::None
    }
}

fn main() {
    let (mut snui, mut event_loop) = Application::new(true);

    let mut color = WidgetLayout::new(5.).orientation(Orientation::Vertical);

    color.add(header());
    color.add(body().pad(20.));
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
                if signal == Request::Close {
                    core.destroy();
                    std::process::exit(0);
                }
            }
        },
    );

    snui.run(&mut event_loop);
}

fn sliders() -> WidgetLayout<Request> {
    [RED, GRN, BLU, BG0]
        .iter()
        .map(|color| {
            let request = match *color {
                RED => Request::Red,
                BLU => Request::Blue,
                GRN => Request::Green,
                BG0 => Request::Alpha,
                _ => Request::Close,
            };
            widgets::slider::Slider::new(200, 8)
                .request(request)
                .background(*color)
                .ext()
                .background(BG2)
                .even_radius(3.)
                .child()
        })
        .collect::<WidgetLayout<Request>>()
        .spacing(10.)
        .orientation(Orientation::Vertical)
}

fn header() -> impl Widget<Request> {
    let mut buttons = WidgetLayout::new(5.);
    let text: Text = Label::default("Copy", 15.).into();
    let icon = Label::new("", 21.)
        .color(YEL)
        .font(FontProperty::new("CaskaydiaCove Nerd Font Mono"));
    buttons.add(
        icon.clamp()
            .constraint(Constraint::Downward)
            .with_size(25., 25.)
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
            .with_size(40., 25.)
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
                        if let Data::Uint(_) = ctx
                            .get(Message::new(Request::Source(Format::Hex), ()))
                            .unwrap()
                        {
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

    let header =
        CenterBox::from(buttons, Label::default("app_name", 15.), Cross {}).with_width(305.);

    header
}

fn body() -> WidgetLayout<Request> {
    let mut layout = WidgetLayout::new(15.).orientation(Orientation::Vertical);

    let listener = Listener::from(Label::default("", 18.))
        .request(Request::Source(Format::Hex))
        .poll()
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
