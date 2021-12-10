use snui::font::{FontProperty, FontStyle};
use snui::data::{Controller, Data};
use snui::scene::*;
use snui::wayland::shell::*;
use snui::widgets::container::*;
use snui::widgets::shapes::*;
use snui::widgets::text::Label;
use snui::widgets::*;
use snui::*;

const BG0: u32 = 0xff_25_22_21;
const BG1: u32 = 0xa0_26_23_22;
const BG2: u32 = 0xff_30_2c_2b;
const YEL: u32 = 0xff_d9_b2_7c;
const GRN: u32 = 0xff_95_a8_82;
const BLU: u32 = 0xff_72_87_97;
const ORG: u32 = 0xff_d0_8b_65;
const RED: u32 = 0xff_c6_5f_5f;

enum Object {
    Null = 0,
    Some = 1 << 1,
    Slider = 1 << 2,
}

impl<'d> From<Object> for Data<'d> {
    fn from(this: Object) -> Self {
        Data::Uint(this as u32)
    }
}

struct Test {}

impl Geometry for Test {
    fn width(&self) -> f32 {
        100.
    }
    fn height(&self) -> f32 {
        100.
    }
    fn set_width(&mut self, _width: f32) -> Result<(), f32> {
        Err(self.width())
    }
    fn set_height(&mut self, _height: f32) -> Result<(), f32> {
        Err(self.height())
    }
}

impl Widget for Test {
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        let mut canvas = self.create_canvas(x, y);

        let b = Rectangle::square(
            50.,
            ShapeStyle::solid(BLU),
        );
        let r = Rectangle::square(
            80.,
            ShapeStyle::solid(RED),
        );
        let c = Rectangle::square(
            50.,
            ShapeStyle::border(ORG, 3.),
        )
        .radius(20., 20., 20., 20.);

        canvas.draw(5., 5., b);
        canvas.draw_at_angle(20., 20., r, 25.);
        canvas.draw_at_angle(40., 25., c, -5.);

        canvas.finish()
    }
    fn sync<'d>(&'d mut self, _ctx: &mut context::SyncContext, _event: Event) {}
}

fn main() {
    let (mut snui, mut event_loop) = Application::new(true);

    let slider =
        slider::Slider::horizontal(
            Object::Slider as u32,
            200,
            5,
            ShapeStyle::solid(BG1)
        )
        .wrap()
        .radius(3., 3., 3., 3.);

    let mut serial =
    	WidgetLayout::horizontal(5)
    		.wrap()
    		.padding(10.)
    		.background(BG1)
    		.radius(5.,5.,5.,5.);

	let mut token = 0;

    let button =
    	shapes::Rectangle::square(47., ShapeStyle::solid(YEL))
    	.into_button(move |square, ctx, pointer| {
        match pointer {
            Pointer::MouseClick {
                time: _,
                button,
                pressed,
            } => {
                if pressed && button == MouseButton::Left {
                    match ctx.serialize(data::Message::new(0, Object::Slider)) {
                        Ok(t) => {
                            token = t;
                            square.set_background(GRN);
                        }
                        Err(_) => {
                            if let Err(e) = ctx.deserialize(token) {
                                println!("{:?}", e);
                            }
                            square.set_background(YEL);
                        }
                    }
                }
            }
            _ => {}
        }
    });

    serial.add(
        Label::default("serialize: ", 18.)
    );
    serial.add(button);
	serial.justify(CENTER);


    let mut icons = WidgetLayout::horizontal(5);
    icons.add(shapes::Rectangle::square(20., ShapeStyle::solid(YEL)));
    icons.add(shapes::Rectangle::square(20., ShapeStyle::solid(ORG)));
    icons.add(shapes::Rectangle::square(20., ShapeStyle::solid(RED)));

    let mut titlebar =
        Centerbox::horizontal(
            Label::new("shell", 18.)
            	.font(FontProperty {
                	name: "sans serif".to_owned(),
                	style: FontStyle::Italic
            	}),
            Label::default("snui demo", 18.)
            	.color(BG1),
            icons,
        )
        .wrap()
        .padding(5.);

    titlebar.0.set_anchor(Alignment::Start, Alignment::Center);
    titlebar.1.set_anchor(Alignment::Center, Alignment::Center);
    titlebar.2.set_anchor(Alignment::End, Alignment::Center);
    

    let mut demo = WidgetLayout::vertical(10);
    demo.add(titlebar);
    demo.add(slider);
    demo.add(serial);
    demo.add(Test {});
    demo.justify(Alignment::Center);

    snui.create_inner_application(
        data::DummyController::new(),
        demo
        	.wrap()
            .background(Background::linear_gradient(
                vec![
                	GradientStop::new(0., widgets::u32_to_source(RED)),
                	GradientStop::new(0.5, widgets::u32_to_source(GRN)),
                	GradientStop::new(1., widgets::u32_to_source(BLU)),
                ],
                SpreadMode::Pad,
                0.5
            ))
            .padding(15.)
            .border(RED, 3.)
            .radius(5., 5., 5., 5.),
        event_loop.handle(),
        |_, _| {},
    );

    snui.run(&mut event_loop);
}
