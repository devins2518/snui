use snui::context::*;
use snui::controller::{Controller, ControllerError, TryFromArg};
use snui::scene::*;
use snui::wayland::shell::*;
use snui::widgets::window::WindowState;
use snui::widgets::{shapes::*, text::*, *};
use snui::{style::*, *};

#[derive(Clone, Debug)]
struct ColorControl {
    signal: Option<ColorMsg>,
    color: tiny_skia::Color,
}

#[derive(Clone, Debug, PartialEq)]
enum ColorMsg {
    Close,
    Null,
    Source(u32),
    Red(f32),
    Green(f32),
    Blue(f32),
    Alpha(f32),
}

impl TryInto<String> for ColorMsg {
    type Error = ();
    fn try_into(self) -> Result<String, Self::Error> {
        match self {
            Self::Source(color) => Ok(format!("{:#010X}", color).replace("0x", "#")),
            _ => Err(()),
        }
    }
}

impl TryInto<f32> for ColorMsg {
    type Error = ();
    fn try_into(self) -> Result<f32, Self::Error> {
        match self {
            Self::Red(f) => Ok(f),
            Self::Green(f) => Ok(f),
            Self::Blue(f) => Ok(f),
            Self::Alpha(f) => Ok(f),
            _ => Err(()),
        }
    }
}

impl TryFromArg<f32> for ColorMsg {
    type Error = ();
    fn try_from_arg(&self, f: f32) -> Result<Self, Self::Error> {
        match self {
            Self::Red(_) => Ok(Self::Red(f)),
            Self::Green(_) => Ok(Self::Green(f)),
            Self::Blue(_) => Ok(Self::Blue(f)),
            Self::Alpha(_) => Ok(Self::Alpha(f)),
            _ => Err(()),
        }
    }
}

impl TryInto<WindowState> for ColorMsg {
    type Error = ();
    fn try_into(self) -> Result<WindowState, Self::Error> {
        match self {
            Self::Close => Ok(WindowState::Close),
            _ => Err(()),
        }
    }
}

impl Controller<ColorMsg> for ColorControl {
    fn get<'m>(&'m self, msg: &'m ColorMsg) -> Result<ColorMsg, ControllerError> {
        match msg {
            ColorMsg::Alpha(_) => Ok(ColorMsg::Alpha(self.color.alpha())),
            ColorMsg::Red(_) => Ok(ColorMsg::Red(self.color.red())),
            ColorMsg::Green(_) => Ok(ColorMsg::Red(self.color.green())),
            ColorMsg::Blue(_) => Ok(ColorMsg::Blue(self.color.blue())),
            ColorMsg::Source(_) => {
                let color = self.color.to_color_u8().get();
                Ok(ColorMsg::Source(color))
            }
            _ => Err(ControllerError::Message),
        }
    }
    fn send<'m>(&'m mut self, msg: ColorMsg) -> Result<ColorMsg, ControllerError> {
        match msg {
            ColorMsg::Alpha(alpha) => self.color.set_alpha(alpha),
            ColorMsg::Red(red) => self.color.set_red(red),
            ColorMsg::Green(green) => self.color.set_green(green),
            ColorMsg::Blue(blue) => self.color.set_blue(blue),
            ColorMsg::Close => {}
            _ => return Err(ControllerError::Message),
        }
        self.signal = Some(msg);
        Ok(ColorMsg::Null)
    }
    fn sync(&mut self) -> Result<ColorMsg, ControllerError> {
        if let Some(signal) = self.signal.take() {
            match signal {
                ColorMsg::Close => return Ok(ColorMsg::Close),
                _ => return Ok(ColorMsg::Source(self.color.to_color_u8().get())),
            }
        }
        Err(ControllerError::Waiting)
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

impl Widget<ColorMsg> for ColorBlock {
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
        ctx: &mut SyncContext<ColorMsg>,
        event: Event<'d, ColorMsg>,
    ) -> Damage {
        if let Event::Message(_) = event {
            let msg = ctx.get(&ColorMsg::Source(0)).unwrap();
            if let ColorMsg::Source(color) = msg {
                ctx.window_state(WindowState::Title(msg.try_into().unwrap()));
                self.color = u32_to_source(color).to_color_u8();
            }
            return Damage::Partial;
        }
        Damage::None
    }
}

fn main() {
    let (mut client, mut event_queue) = WaylandClient::new().unwrap();

    let listener = Listener::from(Label::default("").font_size(20.)).message(ColorMsg::Source(0));
    let window = window::default_window(
        listener,
        body().clamp().ext().even_padding(10.).background(BG0),
    );

    client.new_window(
        ColorControl {
            signal: None,
            color: Color::from_rgba(0.5, 0.5, 0.5, 0.5).unwrap(),
        },
        window
            .background(BG2)
            .border(BG2, 2.)
            .even_radius(5.)
            .with_width(300.),
        &event_queue.handle(),
    );

    let listener = Listener::from(Label::default("").font_size(20.)).message(ColorMsg::Source(0));
    let window = window::default_window(
        listener,
        body().clamp().ext().even_padding(10.).background(BG0),
    );

    client.new_window(
        ColorControl {
            signal: None,
            color: Color::from_rgba(0.5, 0.5, 0.5, 0.5).unwrap(),
        },
        window
            .background(BG2)
            .border(BG2, 2.)
            .even_radius(5.)
            .with_width(300.),
        &event_queue.handle(),
    );

    while let Ok(_) =
        event_queue.blocking_dispatch(&mut client)
    {
        if !client.has_client() {
            break;
        }
    }
}

fn sliders() -> WidgetLayout<ColorMsg> {
    [RED, GRN, BLU, BG0]
        .iter()
        .map(|color| {
            let message = match *color {
                RED => ColorMsg::Red(0.),
                BLU => ColorMsg::Blue(0.),
                GRN => ColorMsg::Green(0.),
                BG0 => ColorMsg::Alpha(0.),
                _ => ColorMsg::Close,
            };
            widgets::slider::Slider::new(200, 8)
                .message(message)
                .background(*color)
                .ext()
                .background(BG2)
                .even_radius(3.)
                .child()
        })
        .collect::<WidgetLayout<ColorMsg>>()
        .spacing(10.)
        .orientation(Orientation::Vertical)
}

fn body() -> WidgetLayout<ColorMsg> {
    let mut layout = WidgetLayout::new(15.).orientation(Orientation::Vertical);

    let listener = Listener::from(Label::default("").font_size(20.))
        .message(ColorMsg::Source(0))
        .clamp()
        .with_size(200., 20.)
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
