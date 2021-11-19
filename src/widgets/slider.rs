use crate::*;
use crate::data::*;
use widgets::shapes::rectangle::Rectangle;
use widgets::{shapes::Style, Shape};

pub struct Slider {
    size: f32,
    step: f32,
    pressed: bool,
    slider: Rectangle,
    orientation: Orientation,
}

impl Slider {
    pub fn new(width: f32, height: f32, style: Style) -> Self {
        Slider {
            step: 1.,
            size: width,
            pressed: false,
            orientation: Orientation::Horizontal,
            slider: Rectangle::new(width / 2., height, style),
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
    fn set_size(&mut self, width: f32, height: f32) -> Result<(), (f32, f32)> {
        match &self.orientation {
            Orientation::Horizontal => self.slider.set_size(width.clamp(0., self.size), height),
            Orientation::Vertical => self.slider.set_size(width, height.clamp(0., self.size)),
        }
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
                                    let min = self.slider.radius[0].min(self.slider.radius[3]);
                                    self.slider.width = x.max(min).round();
                                }
                                Orientation::Vertical => {
                                    let min = self.slider.radius[1].min(self.slider.radius[2]);
                                    self.slider.height = y.max(min).round();
                                }
                            }
                        } else {
                            let ratio = match &self.orientation {
                                Orientation::Horizontal => self.slider.width / self.size,
                                Orientation::Vertical => self.slider.height / self.size,
                            };
                            ctx.send(Message::new(0, ratio)).unwrap();
                        }
                    }
                    Pointer::Scroll {
                        orientation: _,
                        value,
                    } => match &self.orientation {
                        Orientation::Horizontal => {
                            let min = self.slider.radius[0].min(self.slider.radius[3]);
                            self.slider.width = (self.slider.width - value).clamp(min, self.width()).round()
                        }
                        Orientation::Vertical => {
                            let min = self.slider.radius[1].min(self.slider.radius[2]);
                            self.slider.height =
                                (self.slider.height - value).clamp(min, self.height()).round()
                        }
                    },
                    Pointer::Hover => {
                        if self.pressed {
                            match &self.orientation {
                                Orientation::Horizontal => {
                                    let min = self.slider.radius[0].min(self.slider.radius[3]);
                                    self.slider.width = x.max(min).round();
                                }
                                Orientation::Vertical => {
                                    let min = self.slider.radius[1].min(self.slider.radius[2]);
                                    self.slider.height = y.max(min).round();
                                }
                            }
                        }
                    }
                    _ => {}
                }
            } else if self.pressed {
                self.pressed = false;
                let ratio = match &self.orientation {
                    Orientation::Horizontal => self.slider.width / self.size,
                    Orientation::Vertical => self.slider.height / self.size,
                };
                ctx.send(Message::new(0, ratio)).unwrap();
            }
        }
    }
}

impl Shape for Slider {
    fn background(self, color: u32) -> Self {
        Self {
            size: self.size,
            step: self.step,
            pressed: false,
            orientation: self.orientation,
            slider: self.slider.background(color),
        }
    }
    fn border(self, color: u32, width: f32) -> Self {
        Self {
            size: self.size,
            step: self.step,
            pressed: false,
            orientation: self.orientation,
            slider: self.slider.border(color, width),
        }
    }
    fn border_color(self, color: u32) -> Self {
        Self {
            size: self.size,
            step: self.step,
            pressed: false,
            orientation: self.orientation,
            slider: self.slider.border_color(color),
        }
    }
    fn border_width(self, width: f32) -> Self {
        Self {
            size: self.size,
            step: self.step,
            pressed: false,
            orientation: self.orientation,
            slider: self.slider.border_width(width),
        }
    }
    fn radius(self, radius: f32) -> Self {
        Self {
            size: self.size,
            step: self.step,
            pressed: false,
            orientation: self.orientation,
            slider: self.slider.radius(radius),
        }
    }
}
