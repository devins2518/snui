use crate::*;
use crate::wayland::Buffer;
use wayland_client::protocol::wl_pointer;
use wayland_client::protocol::wl_pointer::ButtonState;
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_client::Main;
use smithay_client_toolkit::shm::AutoMemPool;
use wayland_client::protocol::wl_buffer::WlBuffer;
use wayland_protocols::wlr::unstable::layer_shell::v1::client::{
    zwlr_layer_surface_v1, zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
};

pub trait Shell {
    fn get_surface(&self) -> &WlSurface;
    fn hide(&mut self);
    fn show(&mut self);
    fn destroy(&self);
    fn render(&mut self);
}

pub struct Application {
    pub widget: Box<dyn Widget>,
    surface: WlSurface,
    mempool: AutoMemPool,
    buffer: Option<WlBuffer>,
    layer_surface: Option<Main<ZwlrLayerSurfaceV1>>,
    // Add later
    // xdg_shell : Option<Main<ZwlrLayerSurfaceV1>>,
}

impl Application {
    pub fn new(
        widget: impl Widget + 'static,
        surface: WlSurface,
        mempool: AutoMemPool,
    ) -> Application {
        Application {
            widget: Box::new(widget),
            surface,
            mempool,
            buffer: None,
            layer_surface: None,
        }
    }
    pub fn attach_layer_surface(&mut self, layer_surface: &Main<ZwlrLayerSurfaceV1>) {
        layer_surface.set_size(self.get_width(), self.get_height());
        self.surface.commit();
        assign_layer_surface::<Self>(self.get_surface(), &layer_surface);
        self.layer_surface = Some(layer_surface.clone());
    }
    pub fn dispatch(&mut self, action: Action) {
        let mut event_loop = Vec::new();
        let width = self.get_width();
        let height = self.get_height();
        self.widget.action(action, &mut event_loop, 0, 0);
        let mut damage = false;
        for ev in event_loop {
            match ev {
                Damage::Widget {
                    widget, x, y
                } => {
                    damage = true;
                    let mut buffer = Buffer::new(
                        width as i32,
                        height as i32,
                        (4 * width) as i32,
                        &mut self.mempool,
                    );
                    widget.draw(buffer.get_mut_buf(), width, x, y);
                    self.buffer = Some(buffer.get_wl_buffer());
                    self.surface.attach(self.buffer.as_ref(), 0, 0);
                    self.surface.damage(
                        x as i32,
                        y as i32,
                        widget.get_width() as i32,
                        widget.get_height() as i32,
                    );
                }
                Damage::Clone {
                    widget, x, y
                } => {
                    damage = true;
                    let mut buffer = Buffer::new(
                        width as i32,
                        height as i32,
                        (4 * width) as i32,
                        &mut self.mempool,
                    );
                    widget.draw(buffer.get_mut_buf(), width, x, y);
                    self.buffer = Some(buffer.get_wl_buffer());
                    self.surface.attach(self.buffer.as_ref(), 0, 0);
                    self.surface.damage(
                        x as i32,
                        y as i32,
                        widget.get_width() as i32,
                        widget.get_height() as i32,
                    );
                }
                Damage::Destroy => {
                    self.destroy();
                    std::process::exit(0);
                }
                _ => {
                    damage = false;
                }
            }
        }
        if damage {
            self.surface.commit();
        }
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
        match res {
            Damage::Widget {
                widget, x, y
            } => {
                let mut buffer = Buffer::new(
                    width as i32,
                    height as i32,
                    (4 * width) as i32,
                    &mut self.mempool,
                );
                widget.draw(buffer.get_mut_buf(), width, x, y);
                self.buffer = Some(buffer.get_wl_buffer());
                self.surface.attach(self.buffer.as_ref(), 0, 0);
                self.surface.damage(
                    x as i32,
                    y as i32,
                    widget.get_width() as i32,
                    widget.get_height() as i32,
                );
                self.surface.commit();
            }
            Damage::Clone {
                widget, x, y
            } => {
                let mut buffer = Buffer::new(
                    width as i32,
                    height as i32,
                    (4 * width) as i32,
                    &mut self.mempool,
                );
                widget.draw(buffer.get_mut_buf(), width, x, y);
                self.buffer = Some(buffer.get_wl_buffer());
                self.surface.attach(self.buffer.as_ref(), 0, 0);
                self.surface.damage(
                    x as i32,
                    y as i32,
                    widget.get_width() as i32,
                    widget.get_height() as i32,
                );
                self.surface.commit();
            }
            Damage::Hide => self.hide(),
            Damage::Destroy => {
                self.destroy();
                std::process::exit(0);
            }
            Damage::None => {}
        }
        Damage::None
    }
    fn resize(&mut self, width: u32, height: u32) -> Result<(), Error> {
        self.mempool.resize((width * height) as usize).unwrap();
        if let Some(layer_surface) = &self.layer_surface {
            layer_surface.set_size(self.get_width(), self.get_height());
        }
        Ok(())
    }
}

impl Shell for Application {
    fn get_surface(&self) -> &WlSurface {
        &self.surface
    }
    fn show(&mut self) {
        self.surface.attach(self.buffer.as_ref(), 0, 0);
        self.surface.damage(
            0,
            0,
            self.widget.get_width() as i32,
            self.widget.get_height() as i32,
        );
        self.resize(self.widget.get_width(),self.widget.get_height()).unwrap();
        self.surface.commit();
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
    fn hide(&mut self) {
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
    fn destroy(&self) {
        self.surface.destroy();
    }
}

pub fn assign_layer_surface<A>(surface: &WlSurface, layer_surface: &Main<ZwlrLayerSurfaceV1>)
where
    A: 'static + Shell + Geometry,
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
                        widget.resize(width, height).unwrap();
                        layer_surface.ack_configure(serial);
                        layer_surface.set_size(width, height);

                        // The client should use commit to notify itself
                        // that it has been configured
                        // The client is also responsible for damage
                        widget.render();
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

pub fn quick_assign_pointer<A: 'static + Geometry + Shell>(
    pointer: &Main<wl_pointer::WlPointer>, mut widget_index: Option<usize>
) {
    let mut input = None;
    let (mut x, mut y) = (0, 0);
    pointer.quick_assign(move |_, event, mut app| {
        let app = app.get::<Vec<A>>().unwrap();
        match event {
            wl_pointer::Event::Enter {
                serial: _,
                surface,
                surface_x,
                surface_y,
            } => {
                for (i, app_w) in app.iter().enumerate() {
                    if surface.eq(app_w.get_surface()) {
                        widget_index = Some(i);
                        break;
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
                app[widget_index.unwrap()].contains(0, 0, x, y, Input::Leave);
                widget_index = None;
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
                    pressed: state == ButtonState::Pressed,
                });
            }
            _ => {}
        }
        if let Some(index) = widget_index {
            // Dispatching the event to widgets
            if let Some(ev) = input {
                let widget = &mut app[index];
                widget.contains(0, 0, x, y, ev);
                input = None;
            }
        }
    });
}

