use crate::*;
use std::io::{BufWriter, Write};
use wayland_client::protocol::wl_pointer;
use wayland_client::protocol::wl_pointer::ButtonState;
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_client::Main;
use wayland_protocols::wlr::unstable::layer_shell::v1::client::{
    zwlr_layer_surface_v1, zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
};

const TRANSPARENT: u32 = 0x00_00_00_00;

pub trait LayerSurface {
    fn show(&mut self);
    fn get_surface(&self) -> &Main<WlSurface>;
    fn resize(&mut self, width: u32, height: u32);
}

pub fn assign_layer_surface<A>(layer_surface: &Main<ZwlrLayerSurfaceV1>)
where
    A: 'static + LayerSurface + Canvas,
{
    layer_surface.quick_assign(move |layer_surface, event, mut app| {
        match event {
            zwlr_layer_surface_v1::Event::Configure {
                serial,
                width,
                height,
            } => {
                let app = app.get::<A>().unwrap();
                app.resize(width, height);
                layer_surface.ack_configure(serial);
                layer_surface.set_size(width, height);

                // The client should use commit to notify itself
                // that it has been configured
                // The client is also responsible for damage
                app.show();
                app.get_surface().commit();
            }
            zwlr_layer_surface_v1::Event::Closed => {
                let app = app.get::<A>().unwrap();
                layer_surface.destroy();
                app.get_surface().destroy();
            }
            _ => {}
        }
    });
}

pub fn quick_assign_pointer<A: 'static + Geometry + Canvas + LayerSurface>(pointer: &Main<wl_pointer::WlPointer>) {
    let mut input = None;
    let (mut x, mut y) = (0, 0);
    pointer.quick_assign(move |_, event, mut app| {
        let app = app.get::<A>().unwrap();
        match event {
            wl_pointer::Event::Enter {
                serial: _,
                surface: _,
                surface_x,
                surface_y,
            } => {
                x = surface_x as u32;
                y = surface_y as u32;
                input = Some(Input::Enter);
            }
            wl_pointer::Event::Motion {
                time: _,
                surface_x,
                surface_y,
            } => {
                x = surface_x as u32;
                y = surface_y as u32;
                input = Some(Input::Hover);
            }
            wl_pointer::Event::Button {
                serial: _,
                time,
                button,
                state,
            } => {
                input = Some(Input::MouseClick {
                    time,
                    button,
                    pressed: match state {
                        ButtonState::Released => false,
                        ButtonState::Pressed => true,
                        _ => false,
                    },
                });
            }
            _ => {}
        }
        // Dispatching the event to widgets
        if let Some(input) = input {
            match app.contains(0, 0, x as u32, y as u32, input) {
                Damage::Area { surface, x, y } => {
                    app.composite(&surface, x, y);
                }
                Damage::Destroy => {
                    let size = app.size();
                    let mut writer = BufWriter::new(app.get_mut_buf());
                    for _ in 0..size {
                        writer.write_all(&TRANSPARENT.to_ne_bytes()).unwrap();
                    }
                    writer.flush().unwrap()
                }
                _ => {}
            }
        }
    });
}

pub fn assign_pointer<A>(
    pointer: &Main<wl_pointer::WlPointer>,
    f: impl Fn(Damage, &mut A) + 'static
)
where
    A: 'static + Geometry + Canvas
{
    let mut input = None;
    let (mut x, mut y) = (0, 0);
    pointer.quick_assign(move |_, event, mut app| {
        let app = app.get::<A>().unwrap();
        match event {
            wl_pointer::Event::Enter {
                serial: _,
                surface: _,
                surface_x,
                surface_y,
            } => {
                x = surface_x as u32;
                y = surface_y as u32;
                input = Some(Input::Enter);
            }
            wl_pointer::Event::Motion {
                time: _,
                surface_x,
                surface_y,
            } => {
                x = surface_x as u32;
                y = surface_y as u32;
                input = Some(Input::Hover);
            }
            wl_pointer::Event::Button {
                serial: _,
                time,
                button,
                state,
            } => {
                input = Some(Input::MouseClick {
                    time,
                    button,
                    pressed: match state {
                        ButtonState::Released => false,
                        ButtonState::Pressed => true,
                        _ => false,
                    },
                });
            }
            _ => {}
        }
        // Dispatching the event to widgets
        if let Some(input) = input {
            let damage = app.contains(0, 0, x as u32, y as u32, input);
            f(damage, app)
        }
    });
}
