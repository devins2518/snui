use snui::context::*;
use snui::mail::*;
use snui::scene::*;
use snui::wayland::backend::*;
use snui::widgets::{
    layout::simple::SimpleLayout,
    layout::dynamic::DynamicLayout,
    label::*,
    shapes::*,
    *
};
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

impl Mail<Channel, f32, f32> for Color {
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

impl Mail<(), (), String> for Color {
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
}

impl Widget<Color> for ColorBlock {
    fn create_node(&mut self, transform: tiny_skia::Transform) -> RenderNode {
        Instruction::new(
            transform,
            Rectangle::empty(self.width, self.height)
                .background(self.color)
                .radius(5.),
        )
        .into()
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<Color>, event: Event) -> Damage {
        if let Event::Sync = event {
            self.color = ctx.color;
            ctx.set_title(ctx.as_string());
            return Damage::Partial;
        }
        Damage::None
    }
}

fn sliders() -> DynamicLayout<impl Widget<Color>> {
    [RED, GRN, BLU, BG2]
        .into_iter()
        .map(|color| {
            let message = match color {
                RED => Channel::Red,
                BLU => Channel::Blue,
                GRN => Channel::Green,
                BG2 => Channel::Alpha,
                _ => unreachable!(),
            };
            Padding::new(
                widgets::slider::Slider::new(message)
                    .with_size(200., 8.)
                    .background(color)
                    .style()
                    .border(BG2, 1.)
                    .radius(3.),
            )
            .padding_top(5.)
            .padding_bottom(5.)
        })
        .collect::<DynamicLayout<_>>()
        .orientation(Orientation::Vertical)
}

fn ui_builder() -> SimpleLayout<impl Widget<Color>> {
    let mut layout = SimpleLayout::new().orientation(Orientation::Vertical);

    let listener = Listener::new("", ())
        .clamp()
        .anchor(CENTER, START)
        .with_height(20.);

    let mut indicator = DynamicLayout::new().orientation(Orientation::Vertical);

    indicator.add(listener);
    indicator.add(
        ColorBlock {
            width: 200.,
            height: 200.,
            color: tiny_skia::Color::WHITE,
        }
        .style()
        .padding(10.)
        .clamp(),
    );

    layout.add(indicator);
    layout.add(sliders().clamp());
    layout.justify(CENTER);

    layout
}

fn main() {
    let (mut client, mut event_queue) = WaylandClient::new().unwrap();

    let listener = Listener::new("", ());
    let window = window::default_window(listener, ui_builder().clamp().style().padding(10.));

    client.new_window(
        Color {
            sync: false,
            color: tiny_skia::Color::WHITE,
        },
        window.background(BG0).border(BG2, 1.).radius(5.),
        &event_queue.handle(),
    );

    while client.has_view() {
        event_queue.blocking_dispatch(&mut client).unwrap();
    }
}
