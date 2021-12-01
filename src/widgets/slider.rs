use crate::data::*;
use crate::*;
use widgets::shapes::rectangle::Rectangle;
use widgets::{shapes::Style, Shape};

pub struct Slider {
    id: u32,
    size: f32,
    step: f32,
    pressed: bool,
    slider: Rectangle,
    orientation: Orientation,
}

impl Slider {
    pub fn vertical(id: u32, width: u32, height: u32, style: Style) -> Self {
        Slider {
            id,
            step: 1.,
            size: height as f32,
            pressed: false,
            orientation: Orientation::Vertical,
            slider: Rectangle::new(width as f32, height as f32 / 2., style),
        }
    }
    pub fn horizontal(id: u32, width: u32, height: u32, style: Style) -> Self {
        Slider {
            id,
            step: 1.,
            size: width as f32,
            pressed: false,
            orientation: Orientation::Horizontal,
            slider: Rectangle::new(width as f32 / 2., height as f32, style),
        }
    }
}

impl Geometry for Slider {
    fn width(&self) -> f32 {
        if let Orientation::Horizontal = &self.orientation {
            self.size
        } else {
            self.slider.width()
        }
    }
    fn height(&self) -> f32 {
        if let Orientation::Vertical = &self.orientation {
            self.size
        } else {
            self.slider.height()
        }
    }
    fn set_width(&mut self, width: f32) -> Result<(), f32> {
        match &self.orientation {
            Orientation::Horizontal => {
                self.size = width.max(0.);
            }
            Orientation::Vertical => {
                self.slider.set_width(width.max(0.)).unwrap();
            }
        }
        Ok(())
    }
    fn set_height(&mut self, height: f32) -> Result<(), f32> {
        match &self.orientation {
            Orientation::Vertical => {
                self.size = height.max(0.);
            }
            Orientation::Horizontal => {
                self.slider.set_height(height.max(0.)).unwrap();
            }
        }
        Ok(())
    }
}

impl Widget for Slider {
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        self.slider.create_node(x, y)
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext, event: Event) {
        if let Event::Pointer(x, y, pointer) = event {
            if x > 0. && y > 0. && x < self.width() && y < self.height() {
                match pointer {
                    Pointer::MouseClick {
                        time: _,
                        button,
                        pressed,
                    } => {
                        self.pressed = pressed;
                        if pressed && button == MouseButton::Left {
                            match &self.orientation {
                                Orientation::Horizontal => {
                                    self.slider.set_width(x.round());
                                }
                                Orientation::Vertical => {
                                    self.slider.set_height(y.round());
                                }
                            }
                        } else {
                            let ratio = match &self.orientation {
                                Orientation::Horizontal => self.slider.width() / self.size,
                                Orientation::Vertical => self.slider.height() / self.size,
                            };
                            ctx.request_draw();
                            match ctx.send(Message::new(self.id, ratio)) {
                                Err(e) => eprintln!("{} : {:?}", self.id, e),
                                _ => {}
                            }
                        }
                    }
                    Pointer::Scroll {
                        orientation: _,
                        value,
                    } => {
                        let ratio = match &self.orientation {
                            Orientation::Horizontal => {
                                self.slider.set_width(
                                    (self.slider.width() - value).clamp(0., self.width()),
                                );
                                self.slider.width() / self.size
                            }
                            Orientation::Vertical => {
                                self.slider.set_height(
                                    (self.slider.height() - value).clamp(0., self.height()),
                                );
                                self.slider.height() / self.size
                            }
                        };
                        ctx.request_draw();
                        match ctx.send(Message::new(self.id, ratio)) {
                            Err(e) => eprintln!("{} : {:?}", self.id, e),
                            _ => {}
                        }
                    }
                    Pointer::Hover => {
                        if self.pressed {
                            match &self.orientation {
                                Orientation::Horizontal => {
                                    self.slider.set_width(x.round());
                                }
                                Orientation::Vertical => {
                                    self.slider.set_width(y.round());
                                }
                            }
                            ctx.request_draw();
                        }
                    }
                    _ => {}
                }
            } else if self.pressed {
                self.pressed = false;
                let ratio = match &self.orientation {
                    Orientation::Horizontal => self.slider.width() / self.size,
                    Orientation::Vertical => self.slider.height() / self.size,
                };
                ctx.request_draw();
                match ctx.send(Message::new(self.id, ratio)) {
                    Err(e) => eprintln!("{} : {:?}", self.id, e),
                    _ => {}
                }
            }
        }
    }
}

impl Shape for Slider {
    fn set_background<B: Into<scene::Background>>(&mut self, background: B) {
        self.slider.set_background(background);
    }
    fn set_border(&mut self, color: u32, width: f32) {
        self.slider.set_border(color, width);
    }
    fn set_border_color(&mut self, color: u32) {
        self.slider.set_border_color(color);
    }
    fn set_radius(&mut self, tl: f32, tr: f32, br: f32, bl: f32) {
        self.slider.set_radius(tl, tr, br, bl);
    }
    fn set_border_width(&mut self, width: f32) {
        self.slider.set_border_width(width);
    }
    fn background<B: Into<scene::Background>>(self, background: B) -> Self {
        Self {
            id: self.id,
            size: self.size,
            step: self.step,
            pressed: false,
            orientation: self.orientation,
            slider: self.slider.background(background),
        }
    }
    fn border(self, color: u32, width: f32) -> Self {
        Self {
            id: self.id,
            size: self.size,
            step: self.step,
            pressed: false,
            orientation: self.orientation,
            slider: self.slider.border(color, width),
        }
    }
    fn border_color(self, color: u32) -> Self {
        Self {
            id: self.id,
            size: self.size,
            step: self.step,
            pressed: false,
            orientation: self.orientation,
            slider: self.slider.border_color(color),
        }
    }
    fn border_width(self, width: f32) -> Self {
        Self {
            id: self.id,
            size: self.size,
            step: self.step,
            pressed: false,
            orientation: self.orientation,
            slider: self.slider.border_width(width),
        }
    }
    fn radius(self, tl: f32, tr: f32, br: f32, bl: f32) -> Self {
        Self {
            id: self.id,
            size: self.size,
            step: self.step,
            pressed: false,
            orientation: self.orientation,
            slider: self.slider.radius(tl, tr, br, bl),
        }
    }
}
