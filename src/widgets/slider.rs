use crate::mail::*;
use crate::*;
use widgets::shapes::rectangle::Rectangle;
use widgets::shapes::Style;

pub struct Slider<M> {
    step: f32,
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
            step: 5.,
            message,
            flip: false,
            size: 100.,
            pressed: false,
            slider: Rectangle::empty(50., 10.),
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
    fn set_width(&mut self, width: f32) -> Result<(), f32> {
        if let Orientation::Horizontal = &self.orientation {
            let ratio = self.slider.width() / self.size;
            self.size = width.max(0.);
            self.slider.set_width(width * ratio)
        } else {
            self.slider.set_width(width)
        }
    }
    fn set_height(&mut self, height: f32) -> Result<(), f32> {
        if let Orientation::Vertical = &self.orientation {
            let ratio = self.slider.height() / self.size;
            self.size = height.max(0.);
            self.slider.set_height(height * ratio)
        } else {
            self.slider.set_height(height)
        }
    }
}

impl<M, D> Widget<D> for Slider<M>
where
    M: Clone + Copy,
    D: Mail<M, f32, f32>,
{
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
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext<D>, event: Event<'d>) -> Damage {
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
                            if pressed && button.is_left() {
                                ctx.set_cursor(Cursor::Hand);
                            } else {
                                ctx.set_cursor(Cursor::Arrow);
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
                            ctx.send(self.message, ratio);
                            return Damage::Partial;
                        }
                        Pointer::Hover => {
                            if self.pressed {
                                match &self.orientation {
                                    Orientation::Horizontal => {
                                        if let Ok(_) = self.slider.set_width(x.round()) {
                                            ctx.send(self.message, self.slider.width() / self.size);
                                            return Damage::Partial;
                                        }
                                    }
                                    Orientation::Vertical => {
                                        if let Ok(_) = self.slider.set_width(y.round()) {
                                            ctx.send(
                                                self.message,
                                                self.slider.height() / self.size,
                                            );
                                            return Damage::Partial;
                                        }
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
                                    ctx.set_cursor(Cursor::Hand);
                                } else {
                                    ctx.set_cursor(Cursor::Arrow);
                                }
                                return Damage::Partial;
                            }
                        }
                        Pointer::Hover => match &self.orientation {
                            Orientation::Horizontal => {
                                let _ = self.slider.set_width(x.min(self.size));
                                ctx.send(self.message, self.slider.width() / self.size);
                                return Damage::Partial;
                            }
                            Orientation::Vertical => {
                                let _ = self.slider.set_height(y.min(self.size));
                                ctx.send(self.message, self.slider.height() / self.size);
                                return Damage::Partial;
                            }
                        },
                        Pointer::Leave => {
                            self.pressed = false;
                        }
                        _ => {}
                    }
                }
            }
            Event::Configure(_) | Event::Sync => {
                if let Some(ratio) = ctx.get(self.message) {
                    match &self.orientation {
                        Orientation::Horizontal => {
                            let width = self.slider.width();
                            let _ = self.slider.set_width(ratio * self.size);
                            if width.round() != (ratio * self.size).round() {
                                return Damage::Partial;
                            }
                        }
                        Orientation::Vertical => {
                            let height = self.slider.height();
                            let _ = self.slider.set_height(ratio * self.size);
                            if height.round() != (ratio * self.size).round() {
                                return Damage::Partial;
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        Damage::None
    }
}

impl<M> Style for Slider<M> {
    fn set_radius_top_left(&mut self, radius: f32) {
        self.slider.set_radius_top_left(radius);
    }
    fn set_radius_top_right(&mut self, radius: f32) {
        self.slider.set_radius_top_right(radius);
    }
    fn set_radius_bottom_right(&mut self, radius: f32) {
        self.slider.set_radius_bottom_right(radius);
    }
    fn set_radius_bottom_left(&mut self, radius: f32) {
        self.slider.set_radius_bottom_left(radius);
    }
    fn set_background<B: Into<scene::Texture>>(&mut self, background: B) {
        self.slider.set_background(background);
    }
    fn set_border_texture<T: Into<scene::Texture>>(&mut self, texture: T) {
        self.slider.set_border_texture(texture);
    }
    fn set_border_size(&mut self, size: f32) {
        self.slider.set_border_size(size);
    }
}
