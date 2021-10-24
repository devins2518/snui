use crate::canvas::Backend;
use crate::wayland::Buffer;
use crate::*;
use raqote::*;
use std::ops::{Deref, DerefMut};
use smithay_client_toolkit::shm::{DoubleMemPool, MemPool};
use std::sync::mpsc::{Receiver, SyncSender};
use wayland_client::protocol::wl_buffer::WlBuffer;
use wayland_client::protocol::wl_keyboard;
use wayland_client::protocol::wl_keyboard::KeyState;
use wayland_client::protocol::wl_pointer;
use wayland_client::protocol::wl_pointer::ButtonState;
use wayland_client::protocol::wl_shm::WlShm;
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_client::protocol::wl_compositor::WlCompositor;
use wayland_client::protocol::wl_output::WlOutput;
use wayland_client::{Display, Main, Attached, QueueToken, EventQueue};
use wayland_protocols::wlr::unstable::layer_shell::v1::client::{
    zwlr_layer_shell_v1, zwlr_layer_shell_v1::ZwlrLayerShellV1,
    zwlr_layer_surface_v1, zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
};

pub struct InnerApplication {
    configured: bool,
    token: QueueToken,
    buffer: Option<WlBuffer>,
    output: Option<WlOutput>,
    compositor: Attached<WlCompositor>,
    shell: Option<Attached<ZwlrLayerShellV1>>,
    surface: Option<WlSurface>,
    shell_surface: Option<ZwlrLayerSurfaceV1>,
    old_surface: Option<(WlSurface, ZwlrLayerSurfaceV1)>
}

impl InnerApplication {
    fn new(compositor: &WlCompositor, token: QueueToken) -> Self {
        InnerApplication {
            compositor: compositor.as_ref().attach(token.clone()),
            token,
            configured: false,
            buffer: None,
            output: None,
            shell: None,
            surface: None,
            shell_surface: None,
            old_surface: None,
        }
    }
    pub fn attach_shell(&mut self, shell: &ZwlrLayerShellV1) {
        self.shell = Some(shell.as_ref().attach(self.token.clone()));
    }
    pub fn create_shell_surface(&mut self, namespace: &str, output: Option<&WlOutput>) -> Option<(Main<WlSurface>, Main<ZwlrLayerSurfaceV1>)> {
        if let Some(shell) = &self.shell {
            self.output = output.cloned();

            let surface = self.compositor.create_surface();

            let shell_surface = shell.get_layer_surface(
                &surface,
                output,
                zwlr_layer_shell_v1::Layer::Top,
                namespace.to_string()
            );

    		surface.quick_assign(|_, _, _| {});
        	self.configured = true;
    		let surface_handle = surface.detach();
            shell_surface.set_size(1, 1);
            shell_surface.quick_assign(move |shell_surface, event, mut inner| match event {
                zwlr_layer_surface_v1::Event::Configure {
                    serial,
                    width: _,
                    height: _,
                } => {
                    shell_surface.ack_configure(serial);
                	if let Some(mut inner) = inner.get::<InnerApplication>() {
                    	if inner.shell_surface.is_some() {
                        	inner.old_surface = Some(
                            	(inner.surface.clone().unwrap(), inner.shell_surface.clone().unwrap())
                        	);
                    	}
                    	inner.surface = Some(surface_handle.clone());
                    	inner.shell_surface = Some(shell_surface.detach());
                	}
                }
                _ => unreachable!()
            });
            surface.commit();

            return Some((surface, shell_surface))
        }
        None
    }
    pub fn set_size(&self, width: u32, height: u32) {
       if let Some(shell_surface) = &self.shell_surface {
           shell_surface.set_size(width, height);
       }
    }
    fn swap_buffer(&mut self, wl_buffer: WlBuffer) {
        self.buffer = Some(wl_buffer);
    }
    pub fn destroy(&mut self) {
        if let Some(surface) = &self.surface {
            surface.destroy();
        }
        if let Some(shell_surface) = &self.shell_surface {
            shell_surface.destroy()
        }
        self.surface = None;
        self.shell_surface = None;
    }
    pub fn hide(&mut self) {
        self.buffer = None;
    }
    pub fn get_surface(&self) -> &WlSurface {
        self.surface.as_ref().unwrap()
    }
    pub fn get_layer_surface(&self) -> &ZwlrLayerSurfaceV1 {
        self.shell_surface.as_ref().unwrap()
    }
    pub fn show(&self) {
        if let Some(surface) = self.surface.as_ref() {
            surface.attach(self.buffer.as_ref(), 0, 0);
            surface.commit();
        }
    }
}

pub struct Application<W: Widget> {
    pub widget: W,
    event_queue: EventQueue,
    running: bool,
    pub canvas: Canvas,
    inner: InnerApplication,
    receiver: Receiver<Dispatch>,
}

impl<W: Widget> Application<W> {
    pub fn new(
        widget: W,
        display: Display,
        compositor: &WlCompositor,
        receiver: Receiver<Dispatch>,
    ) -> Application<W> {
        let event_queue = display.create_event_queue();
        Application {
            receiver,
            running: false,
            inner: InnerApplication::new(compositor, event_queue.token()),
            event_queue,
            canvas: Canvas::new(Backend::Raqote(DrawTarget::new(
                widget.width() as i32,
                widget.height() as i32,
            ))),
            widget,
        }
    }
    pub fn run(mut self, shm: WlShm, mut cb: impl FnMut(&mut Self, &mut MemPool, Dispatch)) {
        let attached = shm.as_ref().attach(self.event_queue.token());
        let mut mempool = DoubleMemPool::new(attached, |_| {}).unwrap();

        self.running = true;
        self.widget.roundtrip(0., 0., &mut self.canvas, &Dispatch::Prepare);

        while self.running {
            if self.inner.configured {
                if self.inner.surface.is_some() {
                    self.inner.set_size(self.widget.width() as u32, self.widget.height() as u32);
                    if let Some(pool) = mempool.pool() {
                        cb(&mut self, pool, Dispatch::Commit);
                        self.render(pool);
                        self.inner.show();
                    }
                    if let Some((surface, shell_surface)) = &self.inner.old_surface {
                        surface.destroy();
                        shell_surface.destroy();
                        self.inner.old_surface = None;
                    }
                    self.inner.configured = false;
                    self.event_queue
                        .sync_roundtrip(&mut self.inner, |_, _, _| unreachable!())
                        .unwrap();
                } else {
                    self.event_queue
                        .dispatch(&mut self.inner, |_, _, _| unreachable!())
                        .unwrap();
                }
            } else {
                if let Ok(dispatch) = self.receiver.recv() {
                    if let Some(mut pool) = mempool.pool() {
                        cb(&mut self, &mut pool, dispatch);
                        self.event_queue
                            .sync_roundtrip(&mut self.inner, |_, _, _| unreachable!())
                            .unwrap();
                    }
                }
            }
        }
    }
    pub fn damage(&mut self, dispatch: &Dispatch, pool: &mut MemPool) {
        let (w, h) = (self.widget.width(), self.widget.height());
        let draw = match &dispatch {
            Dispatch::Commit => {
                self.widget.draw(&mut self.canvas, 0., 0.);
                self.canvas.is_damaged()
            }
            _ => {
                self.widget.roundtrip(0., 0., &mut self.canvas, &dispatch);
                if w != self.widget.width() || h != self.widget.height() {
                    self.canvas.resize(self.widget.width() as i32, self.widget.height() as i32);
                    self.inner.set_size(self.widget.width() as u32, self.widget.width() as u32);
                    self.widget.draw(&mut self.canvas, 0., 0.);
                    true
                } else if self.canvas.is_damaged() {
                    true
                } else {
                    self.widget
                        .roundtrip(0., 0., &mut self.canvas, &Dispatch::Commit);
                    self.canvas.is_damaged()
                }
            }
        };
        if draw {
            if let Ok((mut buffer, wlbuf)) = Buffer::new(pool, &mut self.canvas) {
                self.inner.swap_buffer(wlbuf);
                if let Some(surface) = self.inner.surface.as_ref() {
                    surface.attach(self.inner.buffer.as_ref(), 0, 0);
                    for damage in buffer.canvas().report() {
                       surface.damage(
                            damage.x as i32,
                            damage.y as i32,
                            damage.width as i32,
                            damage.height as i32,
                        );
                    }
                    buffer.merge();
                    surface.commit();
                }
            }
        }
    }
    pub fn render(&mut self, mempool: &mut MemPool) {
        if let Ok((mut buffer, wlbuf)) = Buffer::new(mempool, &mut self.canvas) {
            let canvas = buffer.canvas();
            self.widget.draw(canvas, 0., 0.);
            if let Some(surface) = self.inner.surface.as_ref() {
                for damage in canvas.report() {
                    surface.damage(
                        damage.x as i32,
                        damage.y as i32,
                        damage.width as i32,
                        damage.height as i32,
                    );
                }
            }
            buffer.merge();
            self.inner.swap_buffer(wlbuf);
        }
    }
    pub fn clear(&mut self) {
        self.canvas.clear();
    }
}

impl<W: Widget> Deref for Application<W> {
    type Target = InnerApplication;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<W: Widget> DerefMut for Application<W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
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
                if sender
                    .send(Dispatch::Pointer(x as f32, y as f32, input))
                    .is_ok()
                {
                    input = Pointer::Leave;
                }
            }
        }
        _ => {}
    });
}
