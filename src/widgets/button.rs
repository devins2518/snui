use crate::widgets::*;
use crate::widgets::inner::Inner;

pub struct Button {
    inner: Inner,
    callback: Box<dyn FnMut(&mut Inner, Input) -> Damage>,
}

impl Container for Button {
    fn len(&self) -> u32 {
        1
    }
    fn add(&mut self, _object: impl Widget + 'static) -> Result<(), Error> {
        Err(Error::Overflow("button", 1))
    }
    /*
    fn get_child(&self) -> Vec<&Inner> {
        vec![&self.inner]
    }
    */
}

impl Geometry for Button {
    fn get_width(&self) -> u32 {
        self.inner.get_width()
    }
    fn get_height(&self) -> u32 {
        self.inner.get_height()
    }
    fn get_location(&self) -> (u32, u32) {
        self.inner.get_location()
    }
    fn set_location(&mut self, x: u32, y: u32) {
        self.inner.set_location(x, y);
    }
    fn contains(&mut self, x: u32, y: u32, event: Input) -> Damage {
        let (sx, sy) = self.inner.get_location();
        if x > sx
            && y > sy
            && x < sx + self.inner.get_width()
            && y < sy + self.inner.get_height()
        {
            (self.callback)(&mut self.inner, event)
        } else {
            Damage::None
        }
    }
}

impl Drawable for Button {
    fn set_content(&mut self, content: Content) {
        self.inner.set_content(content);
    }
    fn draw(&self, canvas: &mut Surface, x: u32, y: u32) {
        self.inner.draw(canvas, x, y)
    }
}

impl Widget for Button { }

impl Button {
    pub fn new(inner: impl Widget + 'static, f: impl FnMut(&mut Inner, Input) -> Damage + 'static) -> Button {
        Button {
            inner: Inner::new(inner),
            callback: Box::new(f),
        }
    }
}
