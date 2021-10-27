use crate::context::Backend;
use crate::context::{Context, DamageReport, DamageType};
use crate::wayland::Buffer;
use crate::*;
use smithay_client_toolkit::shm::{DoubleMemPool, MemPool};
use smithay_client_toolkit::WaylandSource;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use wayland_client::protocol::wl_buffer::WlBuffer;
use wayland_client::protocol::wl_compositor::WlCompositor;
use wayland_client::protocol::wl_keyboard;
use wayland_client::protocol::wl_keyboard::KeyState;
use wayland_client::protocol::wl_output::{self, WlOutput};
use wayland_client::protocol::wl_pointer;
use wayland_client::protocol::wl_pointer::ButtonState;
use wayland_client::protocol::wl_seat::{self, Capability, WlSeat};
use wayland_client::protocol::wl_shm::WlShm;
use wayland_client::protocol::wl_surface::WlSurface;
use smithay_client_toolkit::reexports::calloop::{EventLoop, LoopHandle, RegistrationToken};
use wayland_client::{
    Attached, Display, EventQueue, GlobalError, GlobalManager, Interface, Main, Proxy, QueueToken, DispatchData
};
use wayland_protocols::wlr::unstable::layer_shell::v1::client::{
    zwlr_layer_shell_v1, zwlr_layer_shell_v1::Layer, zwlr_layer_shell_v1::ZwlrLayerShellV1,
    zwlr_layer_surface_v1, zwlr_layer_surface_v1::ZwlrLayerSurfaceV1, zwlr_layer_surface_v1::Anchor
};

#[derive(Debug, Clone)]
enum Shell {
    LayerShell {
        layer: Layer,
        anchor: Anchor,
        namespace: String,
        margin: (i32, i32, i32, i32),
        surface: Main<ZwlrLayerSurfaceV1>
    }
}

impl Shell {
    fn destroy(&self) {
        match self {
            Shell::LayerShell {
                layer:_,
                anchor:_,
                namespace:_,
                margin:_,
                surface
            } => {
                surface.destroy();
            }
        }
    }
    fn set_size(&self, width: u32, height: u32) {
        match self {
            Shell::LayerShell {
                layer:_,
                anchor:_,
                namespace:_,
                margin:_,
                surface
            } => {
                surface.set_size(width, height);
            }
        }
    }
}

#[derive(Debug, Clone)]
struct Surface {
    surface: Main<WlSurface>,
    shell: Shell,
    buffer: Option<WlBuffer>,
    previous: Option<Box<Self>>,
}

struct Output {
    width: i32,
    height: i32,
    scale: i32,
    name: String,
    output: Main<WlOutput>,
}

struct Globals {
    outputs: Vec<Output>,
    seats: HashMap<Capability, Main<WlSeat>>,
    shm: Option<Main<WlShm>>,
    compositor: Option<Main<WlCompositor>>,
    shell: Option<Main<ZwlrLayerShellV1>>,
}

struct Application {
    globals: Rc<Globals>,
    global_manager: GlobalManager,
    inner: Vec<InnerApplication>,
    token: RegistrationToken,
    event_loop: EventLoop<'static, Vec<InnerApplication>>,
    // wayland_source: WaylandSource,
}

struct InnerApplication {
    ctx: Context,
    globals: Rc<Globals>,
    mempool: DoubleMemPool,
    widget: Box<dyn Widget>,
    surface: Option<Surface>,
    cb: Box<dyn FnMut(&mut Self, Dispatch)>,
}

impl Surface {
    fn new(
        surface: Main<WlSurface>,
        shell: Shell,
        previous: Option<Surface>,
    ) -> Self {
        Surface {
            surface,
            shell,
            previous: if let Some(surface) = previous {
                Some(Box::new(surface))
            } else {
                None
            },
            buffer: None,
        }
    }
    fn commit(&mut self) {
        self.surface.commit();
        self.previous = None;
    }
    fn destroy(&mut self) {
        self.surface.destroy();
        self.shell.destroy();
        self.buffer = None;
        self.previous = None;
    }
    fn set_size(&self, width: u32, height: u32) {
        self.shell.set_size(width, height);
    }
    fn damage(&self, report: &[DamageReport]) {
        self.surface.attach(self.buffer.as_ref(), 0, 0);
        for d in report {
            self.surface
                .damage(d.x as i32, d.y as i32, d.width as i32, d.height as i32);
        }
    }
    fn attach_buffer(&mut self, buffer: WlBuffer, x: i32, y: i32) {
        self.buffer = Some(buffer);
    }
}

impl Globals {
    fn new() -> Self {
        Self {
            outputs: Vec::new(),
            seats: HashMap::new(),
            shm: None,
            compositor: None,
            shell: None,
        }
    }
    fn create_shell_surface(
        &self,
        geometry: &dyn Widget,
        namespace: &str,
        layer: Layer,
        anchor: Anchor,
        margin: (i32, i32, i32, i32),
        previous: Option<Surface>,
    ) -> Option<Surface> {
        if self.compositor.is_none() || self.shell.is_none() {
            None
        } else {
            let surface = self.compositor.as_ref().unwrap().create_surface();
            let shell = self.shell.as_ref().unwrap().get_layer_surface(
                &surface,
                None,
                layer,
                namespace.to_string(),
            );
            surface.quick_assign(|_, _, _| {});
            shell.set_anchor(anchor);
            shell.set_size(geometry.width() as u32, geometry.height() as u32);
            shell.set_margin(margin.0, margin.1, margin.2, margin.3);
            assign_surface(&shell);
            surface.commit();
            Some(
                Surface::new(
                    surface,
                    Shell::LayerShell {
                        layer,
                        surface: shell,
                        margin,
                        namespace: namespace.to_string(),
                        anchor
                    },
                    previous
                )
            )
        }
    }
    fn create_shell_surface_from(
        &self,
        geometry: &dyn Widget,
        shell: Shell,
        previous: Option<Surface>,
    ) -> Option<Surface> {
        if self.compositor.is_none() || self.shell.is_none() {
            None
        } else {
            let surface = self.compositor.as_ref().unwrap().create_surface();
            surface.quick_assign(|_, _, _| {});
            match shell {
                Shell::LayerShell { layer, anchor, namespace, margin, surface:_ } => {
                    let shell = self.shell.as_ref().unwrap().get_layer_surface(
                        &surface,
                        None,
                        layer,
                        namespace.clone(),
                    );
                    shell.set_anchor(anchor);
                    shell.set_size(geometry.width() as u32, geometry.height() as u32);
                    shell.set_margin(margin.0, margin.1, margin.2, margin.3);
                    surface.commit();
                    assign_surface(&shell);
                    Some(
                        Surface::new(
                            surface,
                            Shell::LayerShell {
                                layer,
                                surface: shell,
                                margin,
                                namespace,
                                anchor
                            },
                            previous
                        )
                    )
                }
            }
        }
    }
    fn create_mempool(&self) -> DoubleMemPool {
        let attached = Attached::from(self.shm.clone().unwrap());
        DoubleMemPool::new(attached, |_| {}).unwrap()
    }
}

impl Output {
    fn new(output: Main<WlOutput>) -> Self {
        Output {
            width: 0,
            height: 0,
            scale: 1,
            name: String::new(),
            output,
        }
    }
}

impl Application {
    fn new(pointer: bool, keyboard: bool) -> Self {
        let display = Display::connect_to_env().unwrap();
        let mut event_queue = display.create_event_queue();
        let attached_display = (*display).clone().attach(event_queue.token());

        // Creating the Globals struct
        let mut globals = Globals::new();

        let global_manager = GlobalManager::new_with_cb(
            &attached_display,
            wayland_client::global_filter!(
                [
                    ZwlrLayerShellV1,
                    1,
                    |shell: Main<ZwlrLayerShellV1>, mut globals: DispatchData| {
                        if let Some(globals) = globals.get::<Globals>() {
                            globals.shell = Some(shell);
                        }
                    }
                ],
                [WlShm, 1, |shm: Main<WlShm>, mut globals: DispatchData| {
                    if let Some(globals) = globals.get::<Globals>() {
                        globals.shm = Some(shm);
                    }
                }],
                [
                    WlCompositor,
                    4,
                    |compositor: Main<WlCompositor>, mut globals: DispatchData| {
                        if let Some(globals) = globals.get::<Globals>() {
                            globals.compositor = Some(compositor);
                        }
                    }
                ],
                [WlSeat, 7, |seat: Main<WlSeat>, _: DispatchData| {
                    seat.quick_assign(move |seat, event, mut globals| match event {
                        wl_seat::Event::Capabilities { capabilities } => {
                            if let Some(globals) = globals.get::<Globals>() {
                                globals.seats.insert(capabilities, seat);
                            }
                        }
                        _ => {}
                    });
                }],
                [
                    WlOutput,
                    3,
                    |output: Main<WlOutput>, _globals: DispatchData| {
                        output.quick_assign(move |wl_output, event, mut globals| match event {
                            wl_output::Event::Geometry {
                                x: _,
                                y: _,
                                physical_width: _,
                                physical_height: _,
                                subpixel: _,
                                make,
                                model: _,
                                transform: _,
                            } => {
                                if let Some(globals) = globals.get::<Globals>() {
                                    if let Some(output) = globals.outputs.last_mut() {
                                        output.name = make;
                                    } else {
                                        let mut output = Output::new(wl_output);
                                        output.name = make;
                                        globals.outputs.push(output);
                                    }
                                }
                            }
                            wl_output::Event::Mode {
                                flags: _,
                                width,
                                height,
                                refresh: _,
                            } => {
                                if let Some(globals) = globals.get::<Globals>() {
                                    if let Some(output) = globals.outputs.last_mut() {
                                        output.width = width;
                                        output.height = height;
                                    } else {
                                        let mut output = Output::new(wl_output);
                                        output.width = width;
                                        output.height = height;
                                        globals.outputs.push(output);
                                    }
                                }
                            }
                            wl_output::Event::Scale { factor } => {
                                if let Some(globals) = globals.get::<Globals>() {
                                    if let Some(output) = globals.outputs.last_mut() {
                                        output.scale = factor;
                                    } else {
                                        let mut output = Output::new(wl_output);
                                        output.scale = factor;
                                        globals.outputs.push(output);
                                    }
                                }
                            }
                            _ => {}
                        });
                    }
                ]
            ),
        );

        event_queue
            .sync_roundtrip(&mut globals, |_, _, _| {})
            .unwrap();
        event_queue
            .sync_roundtrip(&mut globals, |_, _, _| {})
            .unwrap();

        if pointer {}

        if keyboard {}

        let event_loop: EventLoop<'_, DispatchData<'static>> = EventLoop::try_new().expect("Failed to initialize the event loop!");
        let token = WaylandSource::new(event_queue).quick_insert(event_loop.handle()).unwrap();

        Application {
            globals: Rc::new(globals),
            global_manager,
            inner: Vec::new(),
            event_loop: EventLoop::try_new().unwrap(),
            token,
        }
    }
    fn get_outputs(&self) -> &[Output] {
        &self.globals.as_ref().outputs
    }
    fn get_index(inner: &[InnerApplication], surface: &WlSurface) -> usize {
        for i in 0..inner.len() {
            if inner[i].eq(surface) {
                return i;
            }
        }
        0
    }
    fn get_global<I>(&self) -> Result<Main<I>, GlobalError>
    where
        I: Interface + AsRef<Proxy<I>> + From<Proxy<I>>,
    {
        self.global_manager.instantiate_range::<I>(0, 1 << 8)
    }
    fn run(mut self) {
        loop {
            self.event_loop.run(std::time::Duration::from_millis(20), &mut self.inner, |applications| {
                for a in applications {
                    a.dispatch(Dispatch::Prepare);
                }
            }).unwrap();
        }
    }
    fn insert_inner_application(&mut self, iapp: InnerApplication) {
        self.inner.push(iapp);
    }
}

impl InnerApplication {
    pub fn new(
        widget: impl Widget + 'static,
        backend: Backend,
        shell: Shell,
        globals: Rc<Globals>,
        cb: impl FnMut(&mut Self, Dispatch) + 'static,
    ) -> Self {
        InnerApplication {
            cb: Box::new(cb),
            ctx: Context::new(backend),
            surface: globals.as_ref().create_shell_surface_from(&widget, shell, None),
            widget: Box::new(widget),
            mempool: globals.as_ref().create_mempool(),
            globals,
        }
    }
    fn eq(&self, wl_surface: &WlSurface) -> bool {
        if let Some(surface) = &self.surface {
            return surface.surface.detach().eq(wl_surface);
        }
        false
    }
    pub fn replace_surface(&mut self) {
        if let Some(surface) = self.surface.as_mut() {
            surface.destroy();
            match &surface.shell {
                Shell::LayerShell{
                    layer,
                    anchor,
                    namespace,
                    margin,
                    surface:_
                } => {
                self.surface =
                    self.globals
                        .as_ref()
                        .create_shell_surface(self.widget.deref(), namespace, *layer, *anchor, *margin, Some(surface.clone()));
                }
            }
        }
    }
    fn dispatch(&mut self, ev: Dispatch) {
        self.widget.roundtrip(0., 0., &mut self.ctx, &ev);

		let mut show = true;
        match self.ctx.damage_type() {
            DamageType::Full => self.widget.draw(&mut self.ctx, 0., 0.),
            DamageType::Partial => if self.ctx.is_damaged() {
                show = true;
            }
            DamageType::Resize => if let Some(surface) = &self.surface {
                show = true;
                surface.set_size(self.widget.width() as u32, self.widget.height() as u32);
                self.widget.draw(&mut self.ctx, 0., 0.);
            }
        }

        if show {
            if let Some(pool) = self.mempool.pool() {
                if let Some(surface) = &mut self.surface {
                    surface.commit();
                }
            }
        }

        self.ctx.flush();
    }
}

fn assign_surface(shell: &Main<ZwlrLayerSurfaceV1>) {
    shell.quick_assign(|shell, event, mut inner| {
        match event {
            zwlr_layer_surface_v1::Event::Configure {
                serial,
                width,
                height
            } => {
                shell.ack_configure(serial);
                if let Some(inner) = inner.get::<Vec<InnerApplication>>() {
                    for a in inner {
                        if let Some(surface) = &a.surface {
                            match &surface.shell {
                                Shell::LayerShell { layer:_, anchor:_, namespace:_, margin:_, surface } => {
                                    if shell.eq(surface) {
                                        a.dispatch(Dispatch::ForceDraw);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            _ => unreachable!()
        }
    });
}
