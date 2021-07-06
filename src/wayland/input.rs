use crate::snui::*;
use wayland_client::protocol::wl_pointer;
use wayland_client::protocol::wl_pointer::ButtonState;
use wayland_client::Main;

pub fn assign_pointer<G: 'static + Geometry + Canvas>(pointer: &Main<wl_pointer::WlPointer>) {
    let mut input = Input::None;
    let (mut x, mut y) = (0, 0);
    pointer.quick_assign(move |pointer, event, mut shell| {
        let shell = shell.get::<G>().unwrap();
        match event {
            wl_pointer::Event::Enter {
                serial,
                surface,
                surface_x,
                surface_y,
            } => {
                x = surface_x as u32;
                y = surface_y as u32;
                input = Input::Enter;
            }
            wl_pointer::Event::Motion {
                time,
                surface_x,
                surface_y,
            } => {
                x = surface_x as u32;
                y = surface_y as u32;
                input = Input::Hover;
            }
            wl_pointer::Event::Button {
                serial,
                time,
                button,
                state,
            } => {
                input = Input::MouseClick {
                    time,
                    button,
                    pressed: match state {
                        ButtonState::Released => false,
                        ButtonState::Pressed => true,
                        _ => false,
                    },
                };
            }
            _ => {}
        }
        // Dispatching the event to widgets
        match input {
            Input::None => {}
            _ => {
                let msg = shell.contains(x as u32, y as u32, input);
                shell.damage(msg);
            }
        }
    });
}
