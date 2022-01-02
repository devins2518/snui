use crate::*;
use scene::Instruction;
use crate::controller::*;
use widgets::shapes::rectangle::Rectangle;
use widgets::shapes::{ShapeStyle, Style};

pub struct Slider<M: PartialEq + TryIntoMessage<f32> + TryInto<f32>> {
    message: Option<M>,
    flip: bool,
    size: f32,
    pressed: bool,
    slider: Rectangle,
    orientation: Orientation,
}

impl<M: PartialEq + TryIntoMessage<f32> + TryInto<f32>> Slider<M> {
    pub fn new(width: u32, height: u32) -> Self {
        let orientation = if height > width {
            Orientation::Vertical
        } else {
            Orientation::Horizontal
        };
        Slider {
            message: None,
            flip: false,
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
    pub fn flip(mut self) -> Self {
        self.flip = true;
        self
    }
    pub fn message(mut self, message: M) -> Self {
        self.message = Some(message);
        self
    }
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }
}

impl<M: PartialEq + TryIntoMessage<f32> + TryInto<f32>> Geometry for Slider<M> {
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

impl<M: PartialEq + TryIntoMessage<f32> + TryInto<f32>> Widget<M> for Slider<M> {
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        if self.flip {
            match self.orientation {
                Orientation::Horizontal => {
                    let delta = self.width() - self.slider.width();
                    RenderNode::Instruction(Instruction::new(x + delta, y, self.slider.clone()))
                }
                Orientation::Vertical => {
                    let delta = self.height() - self.slider.height();
                    RenderNode::Instruction(Instruction::new(x, y + delta, self.slider.clone()))
                }
            }
        } else {
            RenderNode::Instruction(Instruction::new(x, y, self.slider.clone()))
        }
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<M>, event: &'d Event<'d, M>) -> Damage {
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
                            if let Some(message) = self.message.as_ref() {
                                if let Ok(msg) = message.try_into(ratio) {
                                    let _ = ctx.send(msg);
                                }
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
                            if let Some(message) = self.message.as_ref() {
                                if let Ok(msg) = message.try_into(ratio) {
                                    let _ = ctx.send(msg);
                                }
                            }
                            return Damage::Some;
                        }
                        Pointer::Hover => {
                            if self.pressed {
                                match &self.orientation {
                                    Orientation::Horizontal => {
                                        if let Ok(_) = self.slider.set_width(x.round()) {
                                            if let Some(message) = self.message.as_ref() {
                                                if let Ok(msg) = message
                                                    .try_into(self.slider.width() / self.size)
                                                {
                                                    let _ = ctx.send(msg);
                                                }
                                            }
                                        }
                                    }
                                    Orientation::Vertical => {
                                        if let Ok(_) = self.slider.set_width(y.round()) {
                                            if let Some(message) = self.message.as_ref() {
                                                if let Ok(msg) = message
                                                    .try_into(self.slider.height() / self.size)
                                                {
                                                    let _ = ctx.send(msg);
                                                }
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
                                    if let Some(message) = self.message.as_ref() {
                                        if let Ok(msg) =
                                            message.try_into(self.slider.width() / self.size)
                                        {
                                            let _ = ctx.send(msg);
                                        }
                                    }
                                    return Damage::Some;
                                }
                            }
                            Orientation::Vertical => {
                                if let Ok(_) = self.slider.set_height(y.min(self.size)) {
                                    if let Some(message) = self.message.as_ref() {
                                        if let Ok(msg) =
                                            message.try_into(self.slider.height() / self.size)
                                        {
                                            let _ = ctx.send(msg);
                                        }
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
                if let Some(this) = self.message.as_ref() {
                    if this.eq(msg) {
                        if let Ok(msg) = ctx.get(msg) {
                            if let Ok(ratio) = msg.try_into() {
                                match &self.orientation {
                                    Orientation::Horizontal => {
                                        let _ = self.slider.set_width(ratio * self.size);
                                    }
                                    Orientation::Vertical => {
                                        let _ = self.slider.set_height(ratio * self.size);
                                    }
                                }
                                return Damage::Some;
                            }
                        }
                    }
                }
            }
            Event::Frame => {
                if let Some(message) = self.message.as_ref() {
                    if let Ok(msg) = ctx.get(message) {
                        if let Ok(ratio) = msg.try_into() {
                            match &self.orientation {
                                Orientation::Horizontal => {
                                    let _ = self.slider.set_width(ratio * self.size);
                                }
                                Orientation::Vertical => {
                                    let _ = self.slider.set_height(ratio * self.size);
                                }
                            }
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

impl<M: PartialEq + TryIntoMessage<f32> + TryInto<f32>> Style for Slider<M> {
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
