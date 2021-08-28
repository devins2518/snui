use crate::wayland::Buffer;
use crate::*;
use smithay_client_toolkit::shm::AutoMemPool;
use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use wayland_client::protocol::wl_buffer::WlBuffer;
use wayland_client::protocol::wl_keyboard;
use wayland_client::protocol::wl_keyboard::KeyState;
use wayland_client::protocol::wl_pointer;
use wayland_client::protocol::wl_pointer::ButtonState;
use wayland_client::protocol::wl_shm::WlShm;
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_client::{Display, Main};
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

pub struct Application {
    shm: WlShm,
    hidden: bool,
    surface: WlSurface,
    buffer: Option<WlBuffer>,
    pub widget: Box<dyn Widget>,
    receiver: Receiver<Dispatch>,
    layer_surface: Option<ZwlrLayerSurfaceV1>,
    // Add later
    // xdg_shell : Option<Main<XdgSurface>>,
}

impl Application {
    pub fn new(
        widget: impl Widget + 'static,
        surface: WlSurface,
        shm: WlShm,
    ) -> (Application, Sender<Dispatch>) {
        let (sender, receiver) = channel();
        (
            Application {
                shm,
                surface,
                receiver,
                buffer: None,
                hidden: false,
                layer_surface: None,
                widget: Box::new(widget),
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
    pub fn run(mut self, display: Display) {
        thread::spawn(move || {
            let event_queue = display.create_event_queue();
            let attached = self.shm.as_ref().attach(event_queue.token());
            let mut mempool = AutoMemPool::new(attached).unwrap();
            self.render(&mut mempool);
            self.surface.damage(
                0,
                0,
                self.widget.get_width() as i32,
                self.widget.get_height() as i32,
            );
            self.widget.roundtrip(0, 0, &Dispatch::Commit);

            loop {
                let width = self.widget.get_width();
                let height = self.widget.get_height();
                if let Ok(dispatch) = self.receiver.recv() {
                    if let Dispatch::Commit = dispatch {
                        self.render(&mut mempool);
                        self.show();
                        self.widget.roundtrip(0, 0, &dispatch);
                    } else {
                        if let Some(damage) = self.widget.roundtrip(0, 0, &dispatch) {
                            let mut buffer = Buffer::new(
                                width as i32,
                                height as i32,
                                (4 * width) as i32,
                                &mut mempool,
                            );
                            damage
                                .widget
                                .draw(buffer.get_mut_buf(), width, damage.x, damage.y);
                            buffer.attach(&self.surface, 0, 0);
                            self.surface.damage(
                                damage.x as i32,
                                damage.y as i32,
                                damage.widget.get_width() as i32,
                                damage.widget.get_height() as i32,
                            );
                            self.buffer = buffer.get_wl_buffer();
                            self.surface.commit();
                        }
                    }
                }
            }
        });
    }
}

impl Shell for Application {
    fn is_hidden(&self) -> bool {
        self.hidden
    }
    fn get_surface(&self) -> &WlSurface {
        &self.surface
    }
    fn show(&mut self) {
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
    fn render(&mut self, mempool: &mut AutoMemPool) {
        let width = self.widget.get_width();
        if mempool
            .resize((width * self.widget.get_height()) as usize * 4)
            .is_ok()
        {
            let mut buffer = Buffer::new(
                self.widget.get_width() as i32,
                self.widget.get_height() as i32,
                (4 * self.widget.get_width()) as i32,
                mempool,
            );
            self.widget.draw(buffer.get_mut_buf(), width, 0, 0);
            buffer.attach(&self.surface, 0, 0);
            self.buffer = buffer.get_wl_buffer();
        }
    }
    fn hide(&mut self) {
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
    fn destroy(&self) {
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
