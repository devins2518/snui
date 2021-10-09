use crate::wayland::buffer;
use crate::*;
use smithay_client_toolkit::shm::{DoubleMemPool, MemPool};
use std::sync::mpsc::{Receiver, SyncSender};
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

pub struct Application<W: Widget> {
    shm: WlShm,
    buffer: Option<WlBuffer>,
    canvas: Canvas,
    pub widget: W,
    pub running: bool,
    pub surface: WlSurface,
    pub receiver: Receiver<Dispatch>,
    pub layer_surface: Option<ZwlrLayerSurfaceV1>,
}

impl<W: Widget> Application<W> {
    pub fn new(
        widget: W,
        surface: WlSurface,
        shm: WlShm,
        receiver: Receiver<Dispatch>,
    ) -> Application<W> {
        Application {
            running: false,
            shm,
            surface,
            receiver,
            canvas: Canvas::new(widget.width() as u32, widget.height() as u32),
            buffer: None,
            layer_surface: None,
            widget,
        }
    }
    pub fn run(mut self, display: Display, mut cb: impl FnMut(&mut Self, &mut MemPool, Dispatch)) {
        let mut event_queue = display.create_event_queue();
        let attached = self.shm.as_ref().attach(event_queue.token());
        let mut mempool = DoubleMemPool::new(attached, |_| {}).unwrap();

        self.running = true;

        while self.running {
            if let Ok(dispatch) = self.receiver.recv() {
                if let Some(mut pool) = mempool.pool() {
                    cb(&mut self, &mut pool, dispatch);
                    event_queue
                        .sync_roundtrip(&mut (), |_, _, _| unreachable!())
                        .unwrap();
                }
            }
        }
    }
    pub fn damage(&mut self, dispatch: Dispatch, pool: &mut MemPool) {
        let width = self.widget.width();
        let height = self.widget.height();
        if let Some(damage) = self.widget.roundtrip(0., 0., &dispatch) {
            if self.canvas.width() != width && self.canvas.height() != height {
                self.canvas = Canvas::new(width as u32, height as u32);
            }
            if let Ok((mut buffer, wlbuf)) = buffer(&mut self.canvas, pool) {
                damage.widget.draw(buffer.canvas(), damage.x, damage.y);
                self.buffer = Some(wlbuf);
                self.surface.attach(self.buffer.as_ref(), 0, 0);
                self.surface.damage(
                    damage.x as i32,
                    damage.y as i32,
                    damage.widget.width() as i32,
                    damage.widget.height() as i32,
                );
                buffer.merge();
                self.surface.commit();
            }
        } else {
            self.widget.roundtrip(0., 0., &Dispatch::Commit);
            if self.widget.damaged() {
                self.render(pool);
                self.show();
            }
        }
    }
    pub fn show(&self) {
        self.surface.attach(self.buffer.as_ref(), 0, 0);
        self.surface.commit();
    }
    pub fn render(&mut self, mempool: &mut MemPool) {
        let width = self.widget.width();
        let height = self.widget.height();
        if self.canvas.width() != width
        && self.canvas.height() != height {
            self.canvas = Canvas::new(width as u32, height as u32);
        }
        if let Ok((mut buffer, wlbuf)) = buffer(&mut self.canvas, mempool)
        {
            let canvas = buffer.canvas();
            if let Some(layer_surface) = &self.layer_surface {
                layer_surface.set_size(self.widget.width() as u32, self.widget.height() as u32);
            }
            self.widget.draw(canvas, 0., 0.);
            for damage in canvas.report() {
                println!("{:?}", damage);
                self.surface.damage(
                    damage.x as i32,
                    damage.y as i32,
                    damage.width as i32,
                    damage.height as i32,
                );
            }
            buffer.merge();
            self.buffer = Some(wlbuf);
        }
    }
    pub fn hide(&mut self) {
        self.buffer = None;
        self.show();
    }
    pub fn destroy(&mut self) {
        self.surface.destroy();
        if let Some(layer_surface) = &self.layer_surface {
            layer_surface.destroy();
        }
        self.running = false;
    }
}

fn get_sender(
    surface: &WlSurface,
    slice: &[(WlSurface, SyncSender<Dispatch>)],
) -> Option<SyncSender<Dispatch>> {
    for app in slice {
        if surface.eq(&app.0) {
            return Some(app.1.clone());
        }
    }
    None
}

pub fn assign_layer_surface(surface: &WlSurface, layer_surface: &Main<ZwlrLayerSurfaceV1>) {
    let surface_handle = surface.clone();
    layer_surface.set_size(1 << 10, 1 << 10);
    surface_handle.commit();
    layer_surface.quick_assign(move |layer_surface, event, mut senders| match event {
        zwlr_layer_surface_v1::Event::Configure {
            serial,
            width,
            height,
        } => {
            layer_surface.ack_configure(serial);
            layer_surface.set_size(width, height);
            if let Some(senders) = senders.get::<Vec<(WlSurface, SyncSender<Dispatch>)>>() {
                for (wlsurface, sender) in senders {
                    if wlsurface == &surface_handle {
                        if !sender.send(Dispatch::Commit).is_ok() {
                            surface_handle.commit();
                        }
                        break;
                    }
                }
            } else if let Some(sender) = senders.get::<SyncSender<Dispatch>>() {
                if !sender.send(Dispatch::Commit).is_ok() {
                    surface_handle.commit();
                }
            }
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
    let mut kb: Option<Key> = None;
    keyboard.quick_assign(move |_, event, mut senders| {
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
                if let Some(senders) = senders.get::<Vec<(WlSurface, SyncSender<Dispatch>)>>() {
                    sender = get_sender(&surface, &senders);
                } else if let Some(focused) = senders.get::<SyncSender<Dispatch>>() {
                    sender = Some(focused.clone());
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
                if let Some(kb) = kb.as_mut() {
                    kb.value = key;
                    kb.pressed = state == KeyState::Pressed;
                } else {
                    kb = Some(Key {
                        value: key,
                        pressed: state == KeyState::Pressed,
                        modifier: None,
                    });
                }
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
        if let Some(ev) = kb {
            if let Some(sender) = &sender {
                if sender.send(Dispatch::Keyboard(ev)).is_ok() {
                    kb = None;
                }
            }
        }
    });
}

pub fn quick_assign_pointer(pointer: &Main<wl_pointer::WlPointer>) {
    let mut sender = None;
    let mut input = Pointer::Enter;
    let (mut x, mut y) = (0., 0.);
    pointer.quick_assign(move |_, event, mut senders| match event {
        wl_pointer::Event::Enter {
            serial: _,
            surface,
            surface_x,
            surface_y,
        } => {
            if let Some(senders) = senders.get::<Vec<(WlSurface, SyncSender<Dispatch>)>>() {
                sender = get_sender(&surface, &senders);
            } else if let Some(focused) = senders.get::<SyncSender<Dispatch>>() {
                sender = Some(focused.clone());
            }
            x = surface_x;
            y = surface_y;
            input = Pointer::Enter;
        }
        wl_pointer::Event::Leave {
            serial: _,
            surface: _,
        } => {
            if sender
                .as_ref()
                .unwrap()
                .send(Dispatch::Pointer(0., 0., Pointer::Leave))
                .is_ok()
            {
                sender = None;
            }
        }
        wl_pointer::Event::Motion {
            time: _,
            surface_x,
            surface_y,
        } => {
            x = surface_x;
            y = surface_y;
            input = Pointer::Hover;
        }
        wl_pointer::Event::Button {
            serial: _,
            time,
            button,
            state,
        } => {
            input = Pointer::MouseClick {
                time,
                button,
                pressed: state == ButtonState::Pressed,
            };
        }
        wl_pointer::Event::Frame => {
            if let Some(sender) = &sender {
                if sender.send(Dispatch::Pointer(x as f32, y as f32, input)).is_ok() {
                    input = Pointer::Leave;
                }
            }
        }
        _ => {}
    });
}
