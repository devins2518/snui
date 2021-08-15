use crate::*;

pub enum Action {
    Command( String ),
    Dispatch( String ),
}

pub struct Button<W: Widget> {
    widget: W,
    action: Action,
}

impl<W: Widget> Button<W> {
    pub fn new(widget: W, action: Action) -> Self {
        Button {
            widget,
            action
        }
    }
    pub fn change(&mut self, action: Action) {
        self.action = action;
    }
}

impl<W: Widget> Geometry for Button<W> {
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
            match event {
                // I need to make a distinction between left-click, right-click and middle-click
                Event::MouseClick { time:_, button:_, pressed } => {
                    match &self.action {
                        Action::Command( string ) => {
                            if pressed {
                                run_command(string);
                            }
                            Damage::None
                        }
                        Action::Dispatch( string ) => Damage::Command(Command::Name(string)),
                    }
                }
                _ => Damage::None
            }
        } else {
            Damage::None
        }
    }
    fn resize(&mut self, width: u32, height: u32) -> Result<(), Error> {
        self.widget.resize(width, height)
    }
}

impl<W: Widget> Drawable for Button<W> {
    fn set_color(&mut self, color: u32) {
        self.widget.set_color(color);
    }
    fn draw(&self, canvas: &mut [u8], width: u32, x: u32, y: u32) {
        self.widget.draw(canvas, width, x, y)
    }
}

impl<W: Widget> Widget for Button<W> {
    fn send_command<'s>(
        &'s mut self,
        command: Command,
        damage_queue: &mut Vec<Damage<'s>>,
        x: u32,
        y: u32,
    ) {
        self.widget.send_command(command, damage_queue, x, y)
    }
}

fn run_command<'s>(value: &'s str) {
    use std::process::Command;
    let mut string = value.split_whitespace();
    let mut command = Command::new(string.next().unwrap());
    command.args(string.collect::<Vec<&str>>());
    command.spawn().expect("Error");
}
