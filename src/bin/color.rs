use snui::context::*;
use snui::data::*;
use snui::scene::*;
use snui::wayland::shell::*;
use snui::widgets::window::WindowRequest;
use snui::widgets::{shapes::*, text::*, *};
use snui::{theme::*, *};

#[derive(Clone, Debug)]
struct Color {
    sync: bool,
    color: tiny_skia::Color,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Channel {
    Red,
    Green,
    Blue,
    Alpha,
}

impl Post<Channel, f32, f32> for Color {
    fn get(&self, message: Channel) -> Option<f32> {
        Some(match message {
            Channel::Alpha => self.color.alpha(),
            Channel::Red => self.color.red(),
            Channel::Green => self.color.green(),
            Channel::Blue => self.color.blue(),
        })
    }
    fn send(&mut self, message: Channel, data: f32) -> Option<f32> {
        self.sync = true;
        match message {
            Channel::Alpha => self.color.set_alpha(data),
            Channel::Red => self.color.set_red(data),
            Channel::Green => self.color.set_green(data),
            Channel::Blue => self.color.set_blue(data),
        }
        None
    }
}

impl Post<(), (), String> for Color {
    fn get(&self, _: ()) -> Option<String> {
        Some(self.as_string())
    }
    fn send(&mut self, _: (), _: ()) -> Option<String> {
        None
    }
}

impl Data for Color {
    fn sync(&mut self) -> bool {
        if self.sync {
            self.sync = false;
            true
        } else {
            false
        }
    }
}

impl Color {
    fn as_string(&self) -> String {
        format!("{:#010X}", self.color.to_color_u8().get()).replace("0x", "#")
    }
}

#[derive(Clone, Copy, Debug)]
struct ColorBlock {
    width: f32,
    height: f32,
    color: tiny_skia::Color,
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

impl Widget<Color> for ColorBlock {
    fn create_node(&mut self, transform: Transform) -> RenderNode {
        Widget::<()>::create_node(
            &mut Rectangle::empty(self.width, self.height)
                .background(self.color)
                .even_radius(5.),
            transform,
        )
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<Color>, event: Event) -> Damage {
        if let Event::Sync = event {
            self.color = ctx.color;
            ctx.window_request(WindowRequest::Title(ctx.as_string()));
            return Damage::Partial;
        }
        Damage::None
    }
}

fn main() {
    let (mut client, mut event_queue) = WaylandClient::new().unwrap();

    let listener = Listener::new("", ());
    let window = window::default_window(
        listener,
        body().clamp().style().even_padding(10.).background(BG0),
    );

    client.new_window(
        Color {
            sync: false,
            color: tiny_skia::Color::WHITE,
        },
        window.background(BG2).border(BG2, 2.).even_radius(5.),
        &event_queue.handle(),
    );

    while client.has_application() {
        event_queue.blocking_dispatch(&mut client).unwrap();
    }
}

fn sliders() -> SimpleLayout<impl Widget<Color>> {
    [RED, GRN, BLU, BG2]
        .iter()
        .map(|color| {
            let message = match *color {
                RED => Channel::Red,
                BLU => Channel::Blue,
                GRN => Channel::Green,
                BG2 => Channel::Alpha,
                _ => unreachable!(),
            };
            widgets::slider::Slider::new(200, 8)
                .message(message)
                .background(*color)
                .style()
                .border(BG2, 1.)
                .even_radius(3.)
        })
        .collect::<SimpleLayout<WidgetStyle<Slider<Channel>>>>()
        .spacing(10.)
        .orientation(Orientation::Vertical)
}

fn body() -> SimpleLayout<impl Widget<Color>> {
    let mut layout = SimpleLayout::new(15.).orientation(Orientation::Vertical);

    let listener = Listener::new("", ())
        .clamp()
        .with_size(200., 30.)
        .anchor(CENTER, START)
        .constraint(Constraint::Downward);

    let mut indicator = DynamicLayout::new().orientation(Orientation::Vertical);

    indicator.add(listener);
    indicator.add(ColorBlock {
        width: 200.,
        height: 200.,
        color: tiny_skia::Color::WHITE,
    });

    layout.add(indicator);
    layout.add(sliders());
    layout.justify(CENTER);

    layout
}
