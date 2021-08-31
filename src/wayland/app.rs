use crate::wayland::Buffer;
use crate::*;
use smithay_client_toolkit::shm::{AutoMemPool, MemPool, DoubleMemPool, Format};
use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, Sender};
use wayland_client::protocol::wl_buffer::WlBuffer;
use wayland_client::protocol::wl_keyboard;
use wayland_client::protocol::wl_keyboard::KeyState;
use wayland_client::protocol::wl_pointer;
use wayland_client::protocol::wl_pointer::ButtonState;
use wayland_client::protocol::wl_shm::WlShm;
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_client::{Display, Main, EventQueue};
use wayland_protocols::wlr::unstable::layer_shell::v1::client::{
    zwlr_layer_surface_v1, zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
};

pub trait Shell {
    fn hide(&mut self);
    fn show(&mut self);
    fn destroy(&self);
    fn render(&mut self, mempool: &mut AutoMemPool);
    fn is_hidden(&self) -> bool;
    fn get_surface(&self) -> &WlSurface;
}

pub struct Application<W: Widget> {
    shm: WlShm,
    hidden: bool,
    receiver: Receiver<Dispatch>,
    pub widget: W,
    pub surface: WlSurface,
    pub buffer: Option<WlBuffer>,
    pub layer_surface: Option<ZwlrLayerSurfaceV1>,
    // Add later
    // xdg_shell : Option<Main<XdgSurface>>,
}

impl<W: Widget> Application<W> {
    pub fn new(
        widget: W,
        surface: WlSurface,
        shm: WlShm,
    ) -> (Application<W>, Sender<Dispatch>) {
        let (sender, receiver) = channel();
        (
            Application {
                shm,
                surface,
                receiver,
                buffer: None,
                hidden: false,
                layer_surface: None,
                widget,
            },
            sender,
        )
    }
    pub fn attach_layer_surface(&mut self, layer_surface: &Main<ZwlrLayerSurfaceV1>) {
        layer_surface.set_size(self.widget.get_width(), self.widget.get_height());
        self.surface.commit();
        assign_layer_surface(self.get_surface(), &layer_surface);
        self.layer_surface = Some(layer_surface.detach());
    }
    pub fn run(mut self, display: Display, mut cb: impl FnMut(&mut Self, &mut EventQueue, &mut MemPool, Dispatch)) {
        let mut event_queue = display.create_event_queue();
        let attached = self.shm.as_ref().attach(event_queue.token());
        let mut mempool= DoubleMemPool::new(attached, |_| {}).unwrap();
        if let Some(mut pool) = mempool.pool() {
            self.render(&mut pool);
            self.surface.damage(
                0,
                0,
                self.widget.get_width() as i32,
                self.widget.get_height() as i32,
            );
            self.widget.roundtrip(0, 0, &Dispatch::Commit);
        }

        loop {
            if let Ok(dispatch) = self.receiver.recv() {
                if let Some(mut pool) = mempool.pool() {
                    cb(&mut self, &mut event_queue, &mut pool, dispatch);
                }
            }
        }
    }
    pub fn is_hidden(&self) -> bool {
        self.hidden
    }
    pub fn get_surface(&self) -> &WlSurface {
        &self.surface
    }
    pub fn show(&mut self) {
        self.hidden = false;
        self.surface.attach(self.buffer.as_ref(), 0, 0);
        self.surface.damage(
            0,
            0,
            self.widget.get_width() as i32,
            self.widget.get_height() as i32,
        );
        self.surface.commit();
    }
    pub fn render(&mut self, mempool: &mut MemPool) {
        if let Ok(mut buf) = Buffer::new(
            self.widget.get_width(),
            self.widget.get_height(),
            mempool,
        ) {
            self.widget.draw(&mut buf.canvas, 0, 0);
            buf.attach(&self.surface, 0, 0);
            self.buffer = buf.get();
        }
    }
    pub fn hide(&mut self) {
        self.hidden = true;
        self.surface.attach(None, 0, 0);
        self.surface.damage(
            0,
            0,
            self.widget.get_width() as i32,
            self.widget.get_height() as i32,
        );
        self.surface.commit();
    }
    pub fn destroy(&self) {
        self.surface.destroy();
        if let Some(layer_surface) = &self.layer_surface {
            layer_surface.destroy();
        }
    }
}

pub fn assign_layer_surface(surface: &WlSurface, layer_surface: &Main<ZwlrLayerSurfaceV1>) {
    let surface_handle = surface.clone();
    layer_surface.quick_assign(move |layer_surface, event, _| match event {
        zwlr_layer_surface_v1::Event::Configure {
            serial,
            width,
            height,
        } => {
            layer_surface.ack_configure(serial);
            layer_surface.set_size(width, height);
            surface_handle.commit();
        }
        zwlr_layer_surface_v1::Event::Closed => {
            surface_handle.destroy();
            layer_surface.destroy();
        }
        _ => {}
    });
}

pub fn quick_assign_keyboard(keyboard: &Main<wl_keyboard::WlKeyboard>) {
    let mut sender = None;
    let mut kb_key = None;
    keyboard.quick_assign(move |_, event, mut hashmap| {
        if let Some(hashmap) = hashmap.get::<Vec<(WlSurface, Sender<Dispatch>)>>() {
            match event {
                wl_keyboard::Event::Keymap {
                    format: _,
                    fd: _,
                    size: _,
                } => {}
                wl_keyboard::Event::Enter {
                    serial: _,
                    surface,
                    keys: _,
                } => {
                    for app in hashmap {
                        if surface.eq(&app.0) {
                            sender = Some(app.1.clone());
                            break;
                        }
                    }
                }
                wl_keyboard::Event::Leave {
                    serial: _,
                    surface: _,
                } => {
                    sender = None;
                }
                wl_keyboard::Event::Key {
                    serial: _,
                    time: _,
                    key,
                    state,
                } => {
                    kb_key = Some(Key {
                        key,
                        pressed: state == KeyState::Pressed,
                        modifier: None,
                    });
                }
                wl_keyboard::Event::Modifiers {
                    serial: _,
                    mods_depressed: _,
                    mods_latched: _,
                    mods_locked: _,
                    group: _,
                } => {}
                wl_keyboard::Event::RepeatInfo { rate: _, delay: _ } => {}
                _ => {}
            }
            // Dispatching the event to widgets
            if let Some(ev) = kb_key {
                if let Some(sender) = &sender {
                    if sender.send(Dispatch::Keyboard(ev)).is_ok() {
                        kb_key = None;
                    }
                }
            }
        }
    });
}

pub fn quick_assign_pointer(pointer: &Main<wl_pointer::WlPointer>) {
    let mut input = None;
    let mut sender = None;
    let (mut x, mut y) = (0, 0);
    pointer.quick_assign(move |_, event, mut hashmap| {
        match event {
            wl_pointer::Event::Enter {
                serial: _,
                surface,
                surface_x,
                surface_y,
            } => {
                if let Some(hashmap) = hashmap.get::<Vec<(WlSurface, Sender<Dispatch>)>>() {
                    for app in hashmap {
                        if surface.eq(&app.0) {
                            sender = Some(app.1.clone());
                            break;
                        }
                    }
                } else if let Some(focused) = hashmap.get::<Sender<Dispatch>>() {
                    sender = Some(focused.clone());
                }
                x = surface_x as u32;
                y = surface_y as u32;
                input = Some(Pointer::Enter);
            }
            wl_pointer::Event::Leave {
                serial: _,
                surface: _,
            } => {
                sender = None;
            }
            wl_pointer::Event::Motion {
                time: _,
                surface_x,
                surface_y,
            } => {
                x = surface_x as u32;
                y = surface_y as u32;
            }
            wl_pointer::Event::Button {
                serial: _,
                time,
                button,
                state,
            } => {
                input = Some(Pointer::MouseClick {
                    time,
                    button,
                    pressed: state == ButtonState::Pressed,
                });
            }
            _ => {}
        }
        if let Some(ev) = input {
            if let Some(sender) = &sender {
                if sender.send(Dispatch::Pointer(x, y, ev)).is_ok() {
                    input = None;
                }
            }
        }
    });
}
