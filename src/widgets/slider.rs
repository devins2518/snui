use crate::mail::*;
use crate::*;
use widgets::shapes::rectangle::Rectangle;
use widgets::shapes::Style;

pub struct Slider<M> {
    message: M,
    flip: bool,
    size: f32,
    pressed: bool,
    slider: Rectangle,
    orientation: Orientation,
}

impl<M> Slider<M> {
    pub fn new(message: M) -> Self {
        Slider {
            message,
            flip: false,
            size: 100.,
            pressed: false,
            slider: Rectangle::new(50., 10.),
            orientation: Orientation::Horizontal,
        }
    }
    pub fn flip(mut self) -> Self {
        self.flip = true;
        self
    }
    pub fn message(mut self, message: M) -> Self {
        self.message = message;
        self
    }
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }
}

impl<M> Geometry for Slider<M> {
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
}

impl<M, T> Widget<T> for Slider<M>
where
    M: Clone + Copy,
    T: for<'a> Mail<'a, M, f32, f32>,
{
    fn draw_scene(&mut self, scene: Scene) {
        let (x, y) = if self.flip {
            match self.orientation {
                Orientation::Horizontal => (self.width() - self.slider.width(), 0.),
                Orientation::Vertical => (0., self.height() - self.slider.height()),
            }
        } else {
            (0., 0.)
        };
        Widget::<()>::draw_scene(&mut self.slider, scene.translate(x, y))
    }
    fn event<'s>(&'s mut self, ctx: &mut SyncContext<T>, event: Event<'s>) -> Damage {
        match event {
            Event::Pointer(MouseEvent {
                pointer,
                ref position,
            }) => {
                if self.contains(position) {
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
                                        self.slider.set_width(position.x.round());
                                    }
                                    Orientation::Vertical => {
                                        self.slider.set_height(position.y.round());
                                    }
                                }
                            }
                            if pressed && button.is_left() {
                                ctx.window().set_cursor(Cursor::Hand);
                            } else {
                                ctx.window().set_cursor(Cursor::Arrow);
                            }
                            let ratio = match &self.orientation {
                                Orientation::Horizontal => self.slider.width() / self.size,
                                Orientation::Vertical => self.slider.height() / self.size,
                            };
                            ctx.send(self.message, ratio);
                            return Damage::Partial;
                        }
                        Pointer::Scroll {
                            orientation: _,
                            step,
                        } => {
                            let ratio = match &self.orientation {
                                Orientation::Horizontal => {
                                    let width = (self.slider.width()
                                        - match step {
                                            Step::Value(v) => v,
                                            Step::Increment(s) => (s as f32 * self.size) / 100.,
                                        })
                                    .clamp(0., self.size);
                                    self.slider.set_width(width);
                                    width / self.size
                                }
                                Orientation::Vertical => {
                                    let height = (self.slider.height()
                                        - match step {
                                            Step::Value(v) => v,
                                            Step::Increment(s) => (s as f32 * self.size) / 100.,
                                        })
                                    .clamp(0., self.size);
                                    self.slider.set_height(height);
                                    height / self.size
                                }
                            };
                            ctx.send(self.message, ratio);
                            return Damage::Partial;
                        }
                        Pointer::Hover => {
                            if self.pressed {
                                match &self.orientation {
                                    Orientation::Horizontal => {
                                        let width = position.x.clamp(0., self.size);
                                        self.slider.set_width(position.x.round());
                                        ctx.send(self.message, width / self.size);
                                        return Damage::Partial;
                                    }
                                    Orientation::Vertical => {
                                        let height = position.y.clamp(0., self.size);
                                        self.slider.set_width(height);
                                        ctx.send(self.message, height / self.size);
                                        return Damage::Partial;
                                    }
                                }
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
                                if pressed {
                                    ctx.window().set_cursor(Cursor::Hand);
                                } else {
                                    ctx.window().set_cursor(Cursor::Arrow);
                                }
                                return Damage::Partial;
                            }
                        }
                        Pointer::Hover => match &self.orientation {
                            Orientation::Horizontal => {
                                let width = position.x.clamp(0., self.size);
                                self.slider.set_width(position.x.round());
                                ctx.send(self.message, width / self.size);
                                return Damage::Partial;
                            }
                            Orientation::Vertical => {
                                let height = position.y.clamp(0., self.size);
                                self.slider.set_width(height);
                                ctx.send(self.message, height / self.size);
                                return Damage::Partial;
                            }
                        },
                        Pointer::Leave => {
                            return Damage::Partial;
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        Damage::None
    }
    fn update<'s>(&'s mut self, ctx: &mut SyncContext<T>) -> Damage {
        if let Some(ratio) = ctx.get(self.message) {
            match &self.orientation {
                Orientation::Horizontal => {
                    let width = self.slider.width();
                    self.slider.set_width(ratio * self.size);
                    if width != self.slider.width() {
                        return Damage::Partial;
                    }
                }
                Orientation::Vertical => {
                    let height = self.slider.height();
                    self.slider.set_height(ratio * self.size);
                    if height != self.slider.height() {
                        return Damage::Partial;
                    }
                }
            }
        }
        Damage::None
    }
    fn layout(&mut self, _: &mut LayoutCtx, _: &BoxConstraints) -> Size {
        (self.width(), self.height()).into()
    }
}

impl<M> GeometryExt for Slider<M> {
    fn set_width(&mut self, width: f32) {
        if let Orientation::Horizontal = &self.orientation {
            let ratio = self.slider.width() / self.size;
            self.size = width.max(0.);
            self.slider.set_width(width * ratio)
        } else {
            self.slider.set_width(width)
        }
    }
    fn set_height(&mut self, height: f32) {
        if let Orientation::Vertical = &self.orientation {
            let ratio = self.slider.height() / self.size;
            self.size = height.max(0.);
            self.slider.set_height(height * ratio)
        } else {
            self.slider.set_height(height)
        }
    }
}

impl<M> Style for Slider<M> {
    fn set_top_left_radius(&mut self, radius: f32) {
        self.slider.set_top_left_radius(radius);
    }
    fn set_top_right_radius(&mut self, radius: f32) {
        self.slider.set_top_right_radius(radius);
    }
    fn set_bottom_right_radius(&mut self, radius: f32) {
        self.slider.set_bottom_right_radius(radius);
    }
    fn set_bottom_left_radius(&mut self, radius: f32) {
        self.slider.set_bottom_left_radius(radius);
    }
    fn set_texture<B: Into<scene::Texture>>(&mut self, texture: B) {
        self.slider.set_texture(texture);
    }
}
