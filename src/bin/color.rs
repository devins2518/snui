use snui::context::*;
use snui::mail::*;
use snui::scene::*;
use snui::wayland::backend::*;
use snui::widgets::{label::*, layout::flex::Flex, shapes::*, *};
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

impl<'s> Mail<(), &'s str, String> for Color {
    fn get(&self, _: ()) -> Option<String> {
        Some(self.as_string())
    }
    fn send(&mut self, _: (), _: &'s str) -> Option<String> {
        Some(self.as_string())
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
    abs: Coords,
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
    fn draw_scene(&mut self, mut scene: Scene) {
        self.abs = scene.position();
        scene.insert_primitive(
            &Rectangle::new(self.width, self.height)
                .texture(self.color)
                .radius(5.),
        )
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<Color>, event: Event) -> Damage {
        match event {
            Event::Configure | Event::Sync => {
                self.color = ctx.color;
                let title = ctx.as_string();
                ctx.window().set_title(title);
                return Damage::Partial;
            }
            Event::Pointer(x, y, p) => {
                if self.contains(x, y) {
                    if let Some(_) = p.left_button_click() {
                        ctx.create_popup(|color, mut ctx| {
                            let mut label = Label::default(color.as_string())
                                .background(theme::BEIGE)
                                .padding(5.)
                                .button(|_, ctx: &mut SyncContext<Color>, p| {
                                    if p.left_button_click().is_some() {
                                        ctx.window().close();
                                    }
                                });
                            Menu::Popup {
                                data: color.clone(),
                                offset: Coords::new(self.abs.x + x, self.abs.y + y),
                                anchor: (START, START),
                                size: label.layout(&mut ctx, &BoxConstraints::default()),
                                widget: Box::new(label),
                            }
                        });
                    }
                }
            }
            _ => {}
        }
        Damage::None
    }
    fn layout(&mut self, _ctx: &mut LayoutCtx, _constraints: &BoxConstraints) -> Size {
        (self.width(), self.height()).into()
    }
}

fn sliders() -> Flex<impl Widget<Color>> {
    [RED, GREEN, BLUE, BG2]
        .into_iter()
        .map(|color| {
            let message = match color {
                RED => Channel::Red,
                BLUE => Channel::Blue,
                GREEN => Channel::Green,
                BG2 => Channel::Alpha,
                _ => unreachable!(),
            };
            Padding::new(
                widgets::slider::Slider::new(message)
                    .texture(color)
                    .with_size(200., 8.)
                    .border(BG2, 1.)
                    .radius(3.),
            )
            .padding_top(5.)
            .padding_bottom(5.)
        })
        .collect::<Flex<_>>()
        .orientation(Orientation::Vertical)
}

fn ui_builder() -> Flex<impl Widget<Color>> {
    let listener = Listener::new("", ())
        .with_fixed_height(25.)
        .anchor(CENTER, START);

    let indicator = Flex::column().with_child(listener).with_child(
        ColorBlock {
            width: 200.,
            height: 200.,
            abs: Coords::default(),
            color: tiny_skia::Color::WHITE,
        }
        .padding_top(5.)
        .padding_bottom(5.)
        .clamp(),
    );

    Flex::column()
        .with_child(indicator)
        .with_child(sliders().clamp())
}

fn main() {
    let (mut client, mut event_queue) = WaylandClient::new().unwrap();

    let window =
        extra::window::default_window(Listener::new("", ()), ui_builder().clamp().padding(10.));

    client.new_window(
        Color {
            sync: false,
            color: tiny_skia::Color::WHITE,
        },
        window
            .texture(theme::BG0)
            .radius(5.)
            .decoration(theme::BG2, 1.),
        &event_queue.handle(),
    );

    while client.has_view() {
        event_queue.blocking_dispatch(&mut client).unwrap();
    }
}
