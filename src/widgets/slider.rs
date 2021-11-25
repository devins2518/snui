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
                self.slider.set_width(width.max(0.));
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
                self.slider.set_height(height.max(0.));
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
                            ctx.send(Message::new(self.id, ratio)).unwrap();
                        }
                    }
                    Pointer::Scroll {
                        orientation: _,
                        value,
                    } => {
                        let ratio = match &self.orientation {
                            Orientation::Horizontal => {
                                let min = self.slider.radius[0].min(self.slider.radius[3]);
                                self.slider.set_width(
                                    (self.slider.width - value).clamp(min, self.width())).unwrap();
                                self.slider.width / self.size
                            }
                            Orientation::Vertical => {
                                let min = self.slider.radius[1].min(self.slider.radius[2]);
                                self.slider.set_height(
                                    (self.slider.height - value).clamp(min, self.height())).unwrap();
                                self.slider.height / self.size
                            }
                        };
                        ctx.send(Message::new(self.id, ratio)).unwrap();
                    }
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
            id: self.id,
            size: self.size,
            step: self.step,
            pressed: false,
            orientation: self.orientation,
            slider: self.slider.background(color),
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
    fn radius(self, radius: f32) -> Self {
        Self {
            id: self.id,
            size: self.size,
            step: self.step,
            pressed: false,
            orientation: self.orientation,
            slider: self.slider.radius(radius),
        }
    }
}
