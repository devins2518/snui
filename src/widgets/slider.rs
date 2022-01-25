use crate::controller::*;
use crate::*;
use widgets::shapes::rectangle::Rectangle;
use widgets::shapes::Style;

pub struct Slider<M: PartialEq + TryFromArg<f32> + TryInto<f32>> {
    step: f32,
    message: Option<M>,
    flip: bool,
    size: f32,
    pressed: bool,
    slider: Rectangle,
    orientation: Orientation,
}

impl<M: PartialEq + TryFromArg<f32> + TryInto<f32>> Slider<M> {
    pub fn new(width: u32, height: u32) -> Self {
        let orientation = if height > width {
            Orientation::Vertical
        } else {
            Orientation::Horizontal
        };
        Slider {
            step: 5.,
            message: None,
            flip: false,
            size: match &orientation {
                Orientation::Horizontal => width as f32,
                Orientation::Vertical => height as f32,
            },
            pressed: false,
            slider: match &orientation {
                Orientation::Horizontal => Rectangle::empty(width as f32 / 2., height as f32),
                Orientation::Vertical => Rectangle::empty(width as f32, height as f32 / 2.),
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

impl<M: PartialEq + TryFromArg<f32> + TryInto<f32>> Geometry for Slider<M> {
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

impl<M: PartialEq + TryFromArg<f32> + TryInto<f32>> Widget<M> for Slider<M> {
    fn create_node(&mut self, transform: Transform) -> RenderNode {
        let transform = if self.flip {
            match self.orientation {
                Orientation::Horizontal => {
                    let delta = self.width() - self.slider.width();
                    transform.pre_translate(delta, 0.)
                }
                Orientation::Vertical => {
                    let delta = self.height() - self.slider.height();
                    transform.pre_translate(0., delta)
                }
            }
        } else {
            transform
        };
        Widget::<()>::create_node(&mut self.slider, transform)
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<M>, event: Event<'d, M>) -> Damage {
        match event {
            Event::Pointer(x, y, pointer) => {
                if self.contains(x, y) {
                    match pointer {
                        Pointer::MouseClick {
                            serial: _,
                            button,
                            pressed,
                        } => {
                            self.pressed = pressed;
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
                                if let Ok(msg) = message.try_from_arg(ratio) {
                                    let _ = ctx.send(msg);
                                }
                            }
                            return Damage::Partial;
                        }
                        Pointer::Scroll {
                            orientation: _,
                            value,
                        } => {
                            let ratio = match &self.orientation {
                                Orientation::Horizontal => {
                                    let _ = self.slider.set_width(
                                        (self.slider.width()
                                            - match value {
                                                Move::Value(v) => v,
                                                Move::Step(s) => s as f32 * self.step,
                                            })
                                        .clamp(0., self.width()),
                                    );
                                    self.slider.width() / self.size
                                }
                                Orientation::Vertical => {
                                    let _ = self.slider.set_height(
                                        (self.slider.height()
                                            - match value {
                                                Move::Value(v) => v,
                                                Move::Step(s) => s as f32 * self.step,
                                            })
                                        .clamp(0., self.height()),
                                    );
                                    self.slider.height() / self.size
                                }
                            };
                            if let Some(message) = self.message.as_ref() {
                                if let Ok(msg) = message.try_from_arg(ratio) {
                                    let _ = ctx.send(msg);
                                }
                            }
                            return Damage::Partial;
                        }
                        Pointer::Hover => {
                            if self.pressed {
                                match &self.orientation {
                                    Orientation::Horizontal => {
                                        if let Ok(_) = self.slider.set_width(x.round()) {
                                            if let Some(message) = self.message.as_ref() {
                                                if let Ok(msg) = message
                                                    .try_from_arg(self.slider.width() / self.size)
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
                                                    .try_from_arg(self.slider.height() / self.size)
                                                {
                                                    let _ = ctx.send(msg);
                                                }
                                            }
                                        }
                                    }
                                }
                                return Damage::Partial;
                            }
                        }
                        _ => {}
                    }
                } else if self.pressed {
                    match pointer {
                        Pointer::MouseClick {
                            serial: _,
                            button,
                            pressed,
                        } => {
                            if button.is_left() {
                                self.pressed = pressed;
                                return Damage::Partial;
                            }
                        }
                        Pointer::Hover => match &self.orientation {
                            Orientation::Horizontal => {
                                if let Ok(_) = self.slider.set_width(x.min(self.size)) {
                                    if let Some(message) = self.message.as_ref() {
                                        if let Ok(msg) =
                                            message.try_from_arg(self.slider.width() / self.size)
                                        {
                                            let _ = ctx.send(msg);
                                        }
                                    }
                                    return Damage::Partial;
                                }
                            }
                            Orientation::Vertical => {
                                if let Ok(_) = self.slider.set_height(y.min(self.size)) {
                                    if let Some(message) = self.message.as_ref() {
                                        if let Ok(msg) =
                                            message.try_from_arg(self.slider.height() / self.size)
                                        {
                                            let _ = ctx.send(msg);
                                        }
                                    }
                                    return Damage::Partial;
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
                                return Damage::Partial;
                            }
                        }
                    }
                }
            }
            Event::Configure(_) => {
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
                            return Damage::Partial;
                        }
                    }
                }
            }
            _ => {}
        }
        Damage::None
    }
}

impl<M: PartialEq + TryFromArg<f32> + TryInto<f32>> Style for Slider<M> {
    fn set_background<B: Into<scene::Texture>>(&mut self, background: B) {
        self.slider.set_background(background);
    }
    fn set_border_texture<T: Into<scene::Texture>>(&mut self, texture: T) {
        self.slider.set_border_texture(texture);
    }
    fn set_radius(&mut self, tl: f32, tr: f32, br: f32, bl: f32) {
        self.slider.set_radius(tl, tr, br, bl);
    }
    fn set_border_size(&mut self, size: f32) {
        self.slider.set_border_size(size);
    }
}
