pub mod pointer {
    use crate::widgets::*;
    use active::command::Command;
    use std::ops::Deref;
    use std::rc::Rc;

    #[derive(Copy, Clone, Debug)]
    pub enum Event {
        MouseClick {
            time: u32,
            button: u32,
            pressed: bool,
        },
        Enter,
        Leave,
    }

    #[derive(Clone)]
    pub struct Button<W: Widget + Clone> {
        widget: W,
        callback: Rc<dyn Fn(&mut W, u32, u32, u32, u32, Event) -> Damage>,
    }

    impl<W: Widget + Clone> Geometry for Button<W> {
        fn get_width(&self) -> u32 {
            self.widget.get_width()
        }
        fn get_height(&self) -> u32 {
            self.widget.get_height()
        }
        fn contains<'d>(
            &'d mut self,
            widget_x: u32,
            widget_y: u32,
            x: u32,
            y: u32,
            event: Event,
        ) -> Damage {
            if x > widget_x
                && y > widget_y
                && x < widget_x + self.get_width()
                && y < widget_y + self.get_height()
            {
                self.callback.deref()(&mut self.widget, widget_x, widget_y, x, y, event)
            } else {
                Damage::None
            }
        }
        fn resize(&mut self, width: u32, height: u32) -> Result<(), Error> {
            self.widget.resize(width, height)
        }
    }

    impl<W: Widget + Clone> Drawable for Button<W> {
        fn set_color(&mut self, color: u32) {
            self.widget.set_color(color);
        }
        fn draw(&self, canvas: &mut [u8], width: u32, x: u32, y: u32) {
            self.widget.draw(canvas, width, x, y)
        }
    }

    impl<W: Widget + Clone> Widget for Button<W> {
        fn send_command<'s>(&'s mut self, command: Command) -> Damage {
            self.widget.send_command(command)
        }
    }

    impl<W: Widget + Clone> Button<W> {
        pub fn new(
            widget: W,
            f: impl Fn(&mut W, u32, u32, u32, u32, Event) -> Damage + 'static,
        ) -> Button<W> {
            Button {
                widget: widget,
                callback: Rc::new(f),
            }
        }
    }
}

pub mod command {
    use active::pointer;
    use crate::widgets::*;
    use std::ops::Deref;
    use std::rc::Rc;

    #[derive(Copy, Clone, Debug)]
    pub enum Command<'a> {
        Name(&'a str),
        Key(&'a str, u32),
        Hide,
        Destroy,
        Data(&'a str, &'a dyn std::any::Any),
    }

    impl<'a> Command<'a> {
        pub fn eq(&self, value: &'a str) -> bool {
            match &self {
                Command::Name(name) => name.eq(&value),
                Command::Key(name, _) => name.eq(&value),
                Command::Data(name, _) => name.eq(&value),
                _ => false,
            }
        }
        pub fn get<T: std::any::Any>(&self) -> Option<&T> {
            match self {
                Command::Data(_, value) => value.downcast_ref(),
                _ => None,
            }
        }
    }

    #[derive(Clone)]
    pub struct Actionnable<W: Widget + Clone> {
        pub widget: W,
        callback: Rc<dyn for<'a> Fn(&'a mut W, Command) -> Damage<'a>>,
    }

    impl<W: Widget + Clone> Geometry for Actionnable<W> {
        fn get_width(&self) -> u32 {
            self.widget.get_width()
        }
        fn get_height(&self) -> u32 {
            self.widget.get_height()
        }
        fn contains<'d>(
            &'d mut self,
            widget_x: u32,
            widget_y: u32,
            x: u32,
            y: u32,
            event: pointer::Event,
        ) -> Damage {
            self.widget.contains(widget_x, widget_y, x, y, event)
        }
        fn resize(&mut self, width: u32, height: u32) -> Result<(), Error> {
            self.widget.resize(width, height)
        }
    }

    impl<W: Widget + Clone> Drawable for Actionnable<W> {
        fn set_color(&mut self, color: u32) {
            self.widget.set_color(color);
        }
        fn draw(&self, canvas: &mut [u8], width: u32, x: u32, y: u32) {
            self.widget.draw(canvas, width, x, y)
        }
    }

    impl<W: Widget + Clone> Widget for Actionnable<W> {
        fn send_command<'s>(&'s mut self, command: Command) -> Damage {
            self.callback.deref()(&mut self.widget, command)
        }
    }

    impl<W: Widget + Clone> Actionnable<W> {
        pub fn new(
            widget: W,
            f: impl for<'a> Fn(&'a mut W, Command) -> Damage<'a> + 'static,
        ) -> Self {
            Self {
                widget: widget,
                callback: Rc::new(f),
            }
        }
    }
}
