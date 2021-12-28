use crate::data::*;
use crate::*;
use scene::Instruction;
use widgets::shapes::rectangle::Rectangle;
use widgets::shapes::{ShapeStyle, Style};

pub struct Slider<R: PartialEq + Clone> {
    request: Option<R>,
    size: f32,
    pressed: bool,
    slider: Rectangle,
    orientation: Orientation,
}

impl<R: PartialEq + Clone> Slider<R> {
    pub fn new(width: u32, height: u32) -> Self {
        let orientation = if height > width {
            Orientation::Vertical
        } else {
            Orientation::Horizontal
        };
        Slider {
            request: None,
            size: match &orientation {
                Orientation::Horizontal => width as f32,
                Orientation::Vertical => height as f32,
            },
            pressed: false,
            slider: match &orientation {
                Orientation::Horizontal => Rectangle::new(
                    width as f32 / 2.,
                    height as f32,
                    ShapeStyle::Background(scene::Background::Transparent),
                ),
                Orientation::Vertical => Rectangle::new(
                    width as f32,
                    height as f32 / 2.,
                    ShapeStyle::Background(scene::Background::Transparent),
                ),
            },
            orientation,
        }
    }
    pub fn request(mut self, request: R) -> Self {
        self.request = Some(request);
        self
    }
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }
    fn filter(&mut self, data: Data<R>) -> Result<(), f32> {
        match data {
            Data::Float(ratio) => match &self.orientation {
                Orientation::Horizontal => {
                    return self.slider.set_width(ratio * self.size);
                }
                Orientation::Vertical => {
                    return self.slider.set_height(ratio * self.size);
                }
            },
            Data::Double(ratio) => match &self.orientation {
                Orientation::Horizontal => {
                    return self.slider.set_width(ratio as f32 * self.size);
                }
                Orientation::Vertical => {
                    return self.slider.set_height(ratio as f32 * self.size);
                }
            },
            _ => {}
        }
        Err(0.)
    }
}

impl<R: PartialEq + Clone> Geometry for Slider<R> {
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

impl<R: PartialEq + Clone> Widget<R> for Slider<R> {
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        RenderNode::Instruction(
            Instruction::new(x, y, self.slider.clone()))
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<R>, event: &Event<R>) -> Damage {
        match event {
            Event::Pointer(x, y, pointer) => {
                if self.contains(*x, *y) {
                    match pointer {
                        Pointer::MouseClick {
                            time: _,
                            button,
                            pressed,
                        } => {
                            self.pressed = *pressed;
                            if self.pressed && button.is_left() {
                                match &self.orientation {
                                    Orientation::Horizontal => {
                                        let _ = self.slider.set_width(x.round());
                                    }
                                    Orientation::Vertical => {
                                        let _ = self.slider.set_height(y.round());
                                    }
                                }
                            }
                            let ratio = match &self.orientation {
                                Orientation::Horizontal => self.slider.width() / self.size,
                                Orientation::Vertical => self.slider.height() / self.size,
                            };
                            if let Some(request) = self.request.as_ref() {
                                let _ = ctx.send(Message::new(request.clone(), ratio));
                            }
                            return Damage::Some;
                        }
                        Pointer::Scroll {
                            orientation: _,
                            value,
                        } => {
                            let ratio = match &self.orientation {
                                Orientation::Horizontal => {
                                    let _ = self.slider.set_width(
                                        (self.slider.width() - value).clamp(0., self.width()),
                                    );
                                    self.slider.width() / self.size
                                }
                                Orientation::Vertical => {
                                    let _ = self.slider.set_height(
                                        (self.slider.height() - value).clamp(0., self.height()),
                                    );
                                    self.slider.height() / self.size
                                }
                            };
                            if let Some(request) = self.request.as_ref() {
                                let _ = ctx.send(Message::new(request.clone(), ratio));
                            }
                            return Damage::Some;
                        }
                        Pointer::Hover => {
                            if self.pressed {
                                match &self.orientation {
                                    Orientation::Horizontal => {
                                        if let Ok(_) = self.slider.set_width(x.round()) {
                                            if let Some(request) = self.request.as_ref() {
                                                let _ = ctx.send(Message::new(
                                                    request.clone(),
                                                    self.slider.width() / self.size,
                                                ));
                                            }
                                        }
                                    }
                                    Orientation::Vertical => {
                                        if let Ok(_) = self.slider.set_width(y.round()) {
                                            if let Some(request) = self.request.as_ref() {
                                                let _ = ctx.send(Message::new(
                                                    request.clone(),
                                                    self.slider.height() / self.size,
                                                ));
                                            }
                                        }
                                    }
                                }
                                return Damage::Some;
                            }
                        }
                        _ => {}
                    }
                } else if self.pressed {
                    match pointer {
                        Pointer::MouseClick {
                            time: _,
                            button,
                            pressed,
                        } => {
                            if button.is_left() {
                                self.pressed = *pressed;
                                return Damage::Some;
                            }
                        }
                        Pointer::Hover => match &self.orientation {
                            Orientation::Horizontal => {
                                if let Ok(_) = self.slider.set_width(x.min(self.size)) {
                                    if let Some(request) = self.request.as_ref() {
                                        let _ = ctx.send(Message::new(
                                            request.clone(),
                                            self.slider.width() / self.size,
                                        ));
                                    }
                                    return Damage::Some;
                                }
                            }
                            Orientation::Vertical => {
                                if let Ok(_) = self.slider.set_height(y.min(self.size)) {
                                    if let Some(request) = self.request.as_ref() {
                                        let _ = ctx.send(Message::new(
                                            request.clone(),
                                            self.slider.height() / self.size,
                                        ));
                                    }
                                    return Damage::Some;
                                }
                            }
                        },
                        Pointer::Leave => self.pressed = false,
                        _ => {}
                    }
                }
            }
            Event::Message(msg) => {
                let Message(request, data) = msg;
                if let Some(this) = self.request.as_ref() {
                    if this.eq(request) {
                    if let Ok(data) = ctx.get(Message::new(this.clone(), data.clone())) {
                        if self.filter(data).is_ok() {
                            return Damage::Some;
                        }
                    }
                    }
                }
            }
            Event::Frame => {
                if let Some(request) = self.request.as_ref() {
                    if let Ok(data) = ctx.get(Message::new(request.clone(), ())) {
                        if self.filter(data).is_ok() {
                            return Damage::Some;
                        }
                    }
                }
            }
            _ => {}
        }
        Damage::None
    }
}

impl<R: PartialEq + Clone> Style for Slider<R> {
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
    fn set_border_size(&mut self, size: f32) {
        self.slider.set_border_size(size);
    }
    fn background<B: Into<scene::Background>>(mut self, background: B) -> Self {
        self.set_background(background);
        self
    }
    fn border(mut self, color: u32, size: f32) -> Self {
        self.set_border(color, size);
        self
    }
    fn border_color(mut self, color: u32) -> Self {
        self.set_border_color(color);
        self
    }
    fn border_size(mut self, size: f32) -> Self {
        self.set_border_size(size);
        self
    }
    fn radius(mut self, tl: f32, tr: f32, br: f32, bl: f32) -> Self {
        self.set_radius(tl, tr, br, bl);
        self
    }
}
