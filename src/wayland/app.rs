use crate::*;
use crate::wayland::buffer::*;
use wayland_client::protocol::wl_pointer;
use wayland_client::protocol::wl_pointer::ButtonState;
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_client::Main;
use smithay_client_toolkit::shm::AutoMemPool;
use wayland_client::protocol::wl_buffer::WlBuffer;
use wayland_protocols::wlr::unstable::layer_shell::v1::client::{
    zwlr_layer_surface_v1, zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
};

pub struct Application {
    pub widget: Box<dyn Widget>,
    surface: WlSurface,
    mempool: AutoMemPool,
    buffer: Option<WlBuffer>,
}

impl Application {
    pub fn new(
        widget: impl Widget + 'static,
        surface: WlSurface,
        layer_surface: &Main<ZwlrLayerSurfaceV1>,
        mempool: AutoMemPool,
    ) -> Application {
        layer_surface.set_size(widget.get_width(), widget.get_height());
        surface.commit();
        Application {
            widget: Box::new(widget),
            surface,
            buffer: None,
            mempool,
        }
    }
    fn render(&mut self) {
        let width = self.widget.get_width();
        let mut buffer = Buffer::new(
            self.widget.get_width() as i32,
            self.widget.get_height() as i32,
            (4 * self.widget.get_width()) as i32,
            &mut self.mempool,
        );
        self.widget.draw(buffer.get_mut_buf(), width, 0, 0);
        buffer.attach(&self.surface, 0, 0);
        self.buffer = Some(buffer.get_wl_buffer());
    }
    pub fn hide(&mut self) {
        self.buffer = None;
        self.surface.attach(self.buffer.as_ref(), 0, 0);
        self.surface.damage(
            0,
            0,
            self.widget.get_width() as i32,
            self.widget.get_height() as i32,
        );
        self.surface.commit();
    }
}

impl Geometry for Application {
    fn get_width(&self) -> u32 {
        self.widget.get_width()
    }
    fn get_height(&self) -> u32 {
        self.widget.get_height()
    }
    fn contains<'d>(&'d mut self, widget_x: u32, widget_y: u32, x: u32, y: u32, event: Input) -> Damage<'d> {
        let width = self.get_width();
        let height = self.get_height();
        let res = self.widget.contains(widget_x, widget_y, x, y, event);
        if let Damage::Widget{ widget, x, y } = res {
            let mut buffer = Buffer::new(
                width as i32,
                height as i32,
                (4 * width) as i32,
                &mut self.mempool,
            );
            widget.draw(buffer.get_mut_buf(), width, x, y);
            self.buffer = Some(buffer.get_wl_buffer());
        }
        Damage::None
    }
}

impl LayerSurface for Application {
    fn get_surface(&self) -> &WlSurface {
        &self.surface
    }
    fn resize(&mut self, width: u32, height: u32) {
        self.mempool.resize((width * height) as usize).unwrap();
    }
    fn show(&mut self) {
        self.render();
        self.surface.attach(self.buffer.as_ref(), 0, 0);
        self.surface.damage(
            0,
            0,
            self.widget.get_width() as i32,
            self.widget.get_height() as i32,
        );
        self.surface.commit();
    }
}

pub trait LayerSurface {
    fn get_surface(&self) -> &WlSurface;
    fn resize(&mut self, width: u32, height: u32);
    fn show(&mut self);
}

pub fn assign_layer_surface<A>(surface: &WlSurface, layer_surface: &Main<ZwlrLayerSurfaceV1>)
where
    A: 'static + LayerSurface,
{
    let surface_handle = surface.clone();
    layer_surface.quick_assign(move |layer_surface, event, mut app| {
        let app = app.get::<Vec<A>>().unwrap();
        for widget in app.iter_mut() {
            if widget.get_surface() == &surface_handle {
                match event {
                    zwlr_layer_surface_v1::Event::Configure {
                        serial,
                        width,
                        height,
                    } => {
                        widget.resize(width, height);
                        layer_surface.ack_configure(serial);
                        layer_surface.set_size(width, height);

                        // The client should use commit to notify itself
                        // that it has been configured
                        // The client is also responsible for damage
                        widget.show();
                        widget.get_surface().commit();
                    }
                    zwlr_layer_surface_v1::Event::Closed => {
                        layer_surface.destroy();
                        widget.get_surface().destroy();
                    }
                    _ => {}
                }
            }
        }
    });
}

pub fn quick_assign_pointer<A: 'static + Geometry + LayerSurface>(
    pointer: &Main<wl_pointer::WlPointer>, mut widget_index: Option<usize>
) {
    let mut input = None;
    let (mut x, mut y) = (0, 0);
    let fixed = widget_index.is_some();
    pointer.quick_assign(move |_, event, mut app| {
        let app = app.get::<Vec<A>>().unwrap();
        match event {
            wl_pointer::Event::Enter {
                serial: _,
                surface,
                surface_x,
                surface_y,
            } => {
                if !fixed {
                    for (i, app_w) in app.iter().enumerate() {
                        if surface.eq(app_w.get_surface()) {
                            widget_index = Some(i);
                            break;
                        }
                    }
                }
                x = surface_x as u32;
                y = surface_y as u32;
                input = Some(Input::Enter);
            }
            wl_pointer::Event::Leave {
                serial: _,
                surface: _,
            } => {
                if !fixed {
                    widget_index = None;
                }
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
        if let Some(index) = widget_index {
            // Dispatching the event to widgets
            if let Some(ev) = input {
                let widget = &mut app[index];
                widget.contains(0, 0, x, y, ev);
                widget.show();
                input = None;
            }
        }
    });
}

