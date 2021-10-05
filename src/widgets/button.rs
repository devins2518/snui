use crate::*;

pub struct Button<W: Widget> {
    pub widget: W,
    command: String,
}

impl<W: Widget> Button<W> {
    pub fn new(widget: W, command: String) -> Self {
        Button { widget, command }
    }
    pub fn change(&mut self, command: String) {
        self.command = command;
    }
}

impl<W: Widget> Geometry for Button<W> {
    fn width(&self) -> u32 {
        self.widget.width()
    }
    fn height(&self) -> u32 {
        self.widget.height()
    }
}

impl<W: Widget> Drawable for Button<W> {
    fn set_color(&mut self, color: u32) {
        self.widget.set_color(color);
    }
    fn draw(&self, canvas: &mut Canvas, x: u32, y: u32) {
        if self.damaged() {
            self.widget.draw(canvas, x, y)
        }
    }
}

impl<W: Widget> Widget for Button<W> {
    fn damaged(&self) -> bool {
        self.widget.damaged()
    }
    fn roundtrip<'d>(
        &'d mut self,
        widx: u32,
        widy: u32,
        dispatched: &Dispatch,
    ) -> Option<Damage> {
        if let Dispatch::Pointer(x, y, pointer) = dispatched {
            if *x > widx
                && *y > widy
                && *x < widx + self.width()
                && *y < widy + self.height()
            {
                match pointer {
                    // I need to make a distinction between left-click, right-click and middle-click
                    Pointer::MouseClick {
                        time: _,
                        button: _,
                        pressed,
                    } => {
                        if *pressed {
                            run_command(&self.command);
                        }
                        None
                    }
                    _ => None,
                }
            } else {
                None
            }
        } else {
            None
        }
    }
}

fn run_command<'s>(value: &'s str) {
    use std::process::Command;
    let mut string = value.split_whitespace();
    let mut command = Command::new(string.next().unwrap());
    command.args(string.collect::<Vec<&str>>());
    command.spawn().expect("Error");
}
