use crate::snui::*;
use wayland_client::protocol::wl_pointer;
use wayland_client::protocol::wl_pointer::ButtonState;
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_client::Main;
use wayland_protocols::wlr::unstable::layer_shell::v1::client::{
    zwlr_layer_surface_v1, zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
};

pub trait LayerSurface {
    // fn get_widget(&self) -> &Widget;
    fn get_surface(&self) -> &Main<WlSurface>;
    fn resize(&mut self, width: u32, height: u32);
}

pub fn assign_layer_surface<S>(layer_surface: &Main<ZwlrLayerSurfaceV1>)
where
    // Certaines surface peuvent ne pas etre des Canvas
    // Mettre display dans l'implementation de LayerSurface
    S: 'static + LayerSurface + Canvas,
{
    layer_surface.quick_assign(move |layer_surface, event, mut app| {
        match event {
            zwlr_layer_surface_v1::Event::Configure {
                serial,
                width,
                height,
            } => {
                let app = app.get::<S>().unwrap();
                app.resize(width, height);
                layer_surface.ack_configure(serial);
                layer_surface.set_size(width, height);

                // The client should use commit to notify itself
                // that it has been configured
                // The client is also responsible for damage
                app.display();
                app.get_surface().commit();
            }
            zwlr_layer_surface_v1::Event::Closed => {
                let app = app.get::<S>().unwrap();
                layer_surface.destroy();
                app.get_surface().destroy();
            }
            _ => {}
        }
    });
}

pub fn assign_pointer<G: 'static + Geometry + Canvas>(pointer: &Main<wl_pointer::WlPointer>) {
    let mut input = Input::None;
    let (mut x, mut y) = (0, 0);
    pointer.quick_assign(move |pointer, event, mut app| {
        let app = app.get::<G>().unwrap();
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
                let msg = app.contains(0, 0, x as u32, y as u32, input);
                app.damage(msg);
            }
        }
    });
}
