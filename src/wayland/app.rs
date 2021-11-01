use crate::context::Backend;
use crate::context::{Context, DamageType, Region};
use crate::wayland::Buffer;
use crate::*;
use raqote::*;
use smithay_client_toolkit::reexports::calloop::{EventLoop, RegistrationToken, LoopHandle};
use smithay_client_toolkit::seat::keyboard::{
    keysyms, map_keyboard_repeat, KeyState, RepeatKind, RMLVO, ModifiersState, self
};
use smithay_client_toolkit::shm::{DoubleMemPool};
use smithay_client_toolkit::WaylandSource;

use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use wayland_client::protocol::wl_buffer::WlBuffer;
use wayland_client::protocol::wl_compositor::WlCompositor;

use wayland_client::protocol::wl_output::{self, WlOutput};
use wayland_client::protocol::wl_pointer::{self, WlPointer};
use wayland_client::protocol::wl_region::{WlRegion};
use wayland_client::protocol::wl_seat::{self, Capability, WlSeat};
use wayland_client::protocol::wl_shm::WlShm;
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_client::{
    Attached, DispatchData, Display, EventQueue, GlobalError, GlobalManager, Interface, Main,
    Proxy, QueueToken,
};
use wayland_protocols::wlr::unstable::layer_shell::v1::client::{
    zwlr_layer_shell_v1::Layer, zwlr_layer_shell_v1::ZwlrLayerShellV1,
    zwlr_layer_surface_v1, zwlr_layer_surface_v1::Anchor,
    zwlr_layer_surface_v1::KeyboardInteractivity, zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
};

#[derive(Debug, Clone)]
pub enum Shell {
    LayerShell {
        config: ShellConfig,
        surface: Main<ZwlrLayerSurfaceV1>,
    },
}

#[derive(Debug, Clone)]
pub struct ShellConfig {
    pub layer: Layer,
    pub anchor: Option<Anchor>,
    pub output: Option<WlOutput>,
    pub namespace: String,
    pub exclusive: i32,
    pub interactivity: KeyboardInteractivity,
    pub margin: [i32; 4],
}

impl ShellConfig {
    pub fn default_layer_shell() -> Self {
        ShellConfig {
            layer: Layer::Top,
            anchor: None,
            output: None,
            exclusive: 0,
            interactivity: KeyboardInteractivity::None,
            namespace: "".to_string(),
            margin: [0; 4],
        }
    }
    pub fn layer_shell(
        layer: Layer,
        anchor: Option<Anchor>,
        output: Option<WlOutput>,
        namespace: &str,
        margin: [i32; 4],
    ) -> Self {
        ShellConfig {
            layer,
            anchor,
            output,
            exclusive: 0,
            interactivity: KeyboardInteractivity::None,
            namespace: namespace.to_string(),
            margin,
        }
    }
}

impl Shell {
    fn destroy(&self) {
        match self {
            Shell::LayerShell { config: _, surface } => {
                surface.destroy();
            }
        }
    }
    fn set_size(&self, width: u32, height: u32) {
        match self {
            Shell::LayerShell { config: _, surface } => {
                surface.set_size(width, height);
            }
        }
    }
}

#[derive(Debug, Clone)]
struct Surface {
    shell: Shell,
    region: Main<WlRegion>,
    surface: Main<WlSurface>,
    buffer: Option<WlBuffer>,
    previous: Option<Box<Self>>,
}

#[derive(Debug, Clone)]
pub struct Output {
    pub width: i32,
    pub height: i32,
    pub scale: i32,
    pub name: String,
    pub output: Main<WlOutput>,
}

#[derive(Debug, Clone)]
pub struct Seat {
    pub seat: Main<WlSeat>,
    pub capabilities: Capability,
}

struct Globals {
    outputs: Vec<Output>,
    seats: Vec<Seat>,
    shm: Option<Main<WlShm>>,
    compositor: Option<Main<WlCompositor>>,
    shell: Option<Main<ZwlrLayerShellV1>>,
}

pub struct Application {
    display: Display,
    globals: Rc<Globals>,
    global_manager: GlobalManager,
    pub inner: Vec<InnerApplication>,
    token: RegistrationToken,
}

pub struct CoreApplication {
    ctx: Context,
    globals: Rc<Globals>,
    mempool: DoubleMemPool,
    widget: Box<dyn Widget>,
    surface: Option<Surface>,
}

pub struct InnerApplication {
    core: CoreApplication,
    cb: Box<dyn FnMut(&mut CoreApplication, Dispatch)>,
}

impl Surface {
    fn new(
        surface: Main<WlSurface>,
        shell: Shell,
        region: Main<WlRegion>,
        previous: Option<Surface>,
    ) -> Self {
        Surface {
            surface,
            shell,
            region,
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
        self.region.destroy();
        self.shell.destroy();
        self.buffer = None;
        self.previous = None;
    }
    fn set_size(&self, width: u32, height: u32) {
        self.shell.set_size(width, height);
    }
    fn add_input(&self, report: &[Region]) {
        // if !report.is_empty() {
        //     for r in report {
        //         self.region
        //             .add(r.x as i32, r.y as i32, r.width as i32, r.height as i32);
        //     }
        //     self.surface.set_input_region(Some(&self.region));
        // } else {
        //     self.surface.set_input_region(Some(&self.region));
        // }
    }
    fn damage(&self, report: &[Region]) {
        self.surface.attach(self.buffer.as_ref(), 0, 0);
        for d in report {
            self.surface
                .damage(d.x as i32, d.y as i32, d.width as i32, d.height as i32);
        }
    }
    fn attach_buffer(&mut self, buffer: WlBuffer, _x: i32, _y: i32) {
        self.buffer = Some(buffer);
    }
}

impl Globals {
    fn new() -> Self {
        Self {
            outputs: Vec::new(),
            seats: Vec::new(),
            shm: None,
            compositor: None,
            shell: None,
        }
    }
    pub fn create_shell_surface(
        &self,
        geometry: &dyn Widget,
        namespace: &str,
        layer: Layer,
        anchor: Option<Anchor>,
        output: Option<WlOutput>,
        margin: [i32; 4],
        previous: Option<Surface>,
    ) -> Option<Surface> {
        if self.compositor.is_none() || self.shell.is_none() {
            None
        } else {
            let region = self.compositor.as_ref().unwrap().create_region();
            let surface = self.compositor.as_ref().unwrap().create_surface();
            let shell = self.shell.as_ref().unwrap().get_layer_surface(
                &surface,
                output.as_ref(),
                layer,
                namespace.to_string(),
            );
            surface.quick_assign(|_, _, _| {});
            if let Some(anchor) = &anchor {
                shell.set_anchor(*anchor);
            }
            shell.set_size(geometry.width() as u32, geometry.height() as u32);
            shell.set_margin(margin[0], margin[1], margin[2], margin[3]);
            assign_surface(&shell);
            surface.commit();
            Some(Surface::new(
                surface,
                Shell::LayerShell {
                    surface: shell,
                    config: ShellConfig::layer_shell(layer, anchor, output, namespace, margin),
                },
                region,
                previous,
            ))
        }
    }
    pub fn create_shell_surface_from(
        &self,
        geometry: &dyn Widget,
        config: ShellConfig,
        previous: Option<Surface>,
    ) -> Option<Surface> {
        if self.compositor.is_none() || self.shell.is_none() {
            None
        } else {
            let region = self.compositor.as_ref().unwrap().create_region();
            let surface = self.compositor.as_ref().unwrap().create_surface();
            let shell = self.shell.as_ref().unwrap().get_layer_surface(
                &surface,
                config.output.as_ref(),
                config.layer,
                config.namespace.clone(),
            );
            if let Some(anchor) = &config.anchor {
                shell.set_anchor(*anchor);
            }
            surface.quick_assign(|_, _, _| {});
            shell.set_exclusive_zone(config.exclusive);
            shell.set_keyboard_interactivity(config.interactivity);
            shell.set_size(geometry.width() as u32, geometry.height() as u32);
            shell.set_margin(
                config.margin[0],
                config.margin[1],
                config.margin[2],
                config.margin[3],
            );
            surface.commit();
            assign_surface(&shell);
            Some(Surface::new(
                surface,
                Shell::LayerShell {
                    surface: shell,
                    config,
                },
                region,
                previous,
            ))
        }
    }
    pub fn create_mempool(&self) -> DoubleMemPool {
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
    pub fn new(pointer: bool, keyboard: bool) -> (
        Self,
        EventLoop<'static, Application>
    ) {
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
                                globals.seats.push(Seat { capabilities, seat });
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
                                        if wl_output == output.output {
                                            output.name = make;
                                        } else {
                                            let mut output = Output::new(wl_output);
                                            output.name = make;
                                            globals.outputs.push(output);
                                        }
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
                                        if wl_output == output.output {
                                            output.width = width;
                                            output.height = height;
                                        } else {
                                            let mut output = Output::new(wl_output);
                                            output.width = width;
                                            output.height = height;
                                            globals.outputs.push(output);
                                        }
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
                                        if wl_output == output.output {
                                            output.scale = factor;
                                        } else {
                                            let mut output = Output::new(wl_output);
                                            output.scale = factor;
                                            globals.outputs.push(output);
                                        }
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

        for _ in 0..2 {
            event_queue
                .sync_roundtrip(&mut globals, |_, _, _| {})
                .unwrap();
        }

        let event_loop = EventLoop::try_new().expect("Failed to initialize the event loop!");
        let token = WaylandSource::new(event_queue)
            .quick_insert(event_loop.handle())
            .unwrap();

        for seat in &globals.seats {
            let mut index = 0;
            let mut ch = None;
            if keyboard && (seat.capabilities & Capability::Keyboard == Capability::Keyboard) {
                let _ = map_keyboard_repeat(
                    event_loop.handle(),
                    &Attached::from(seat.seat.clone()),
                    None,
                    RepeatKind::System,
                    move |ev, _, mut inner| match ev {
                        keyboard::Event::Modifiers {
                            modifiers
                        } => {
                            if let Some(application) = inner.get::<Application>() {
                                application.inner[index].dispatch(Dispatch::Keyboard(Key {
                                    utf8: ch.as_ref(),
                                    value: &[],
                                    modifiers: Modifiers::from(modifiers),
                            		pressed: true,
                                }));
                            }
                        }
                        keyboard::Event::Enter {
                            serial: _,
                            surface,
                            rawkeys: _,
                            keysyms: _
                        } => {
                            if let Some(application) = inner.get::<Application>() {
                                index = application.get_index(&surface);
                            }
                        }
                        keyboard::Event::Key {
                            serial: _,
                            time: _,
                            rawkey,
                            keysym: _,
                            state,
                            utf8,
                        } => {
                            ch = utf8;
                            if let Some(application) = inner.get::<Application>() {
                                application.inner[index].dispatch(Dispatch::Keyboard(Key {
                                    utf8: ch.as_ref(),
                                    value: &[rawkey],
                                    modifiers: Modifiers::default(),
                            		pressed: state == keyboard::KeyState::Pressed,
                                }));
                            }
                        }
                        keyboard::Event::Repeat {
                            time: _,
                            rawkey: _,
                            keysym,
                            utf8: _
                        } => {
                            if let Some(application) = inner.get::<Application>() {
                                application.inner[index].dispatch(Dispatch::Keyboard(Key {
                                    utf8: ch.as_ref(),
                                    value: &[keysym],
                                    modifiers: Modifiers::default(),
                            		pressed: true,
                                }));
                            }
                        }
                        _ => {}
                    },
                ).unwrap();
            }
            if pointer && seat.capabilities & Capability::Pointer == Capability::Pointer {
                let pointer = seat.seat.get_pointer();
                assign_pointer(&pointer);
            }
        }

        (
            Application {
                display,
                globals: Rc::new(globals),
                global_manager,
                inner: Vec::new(),
                token,
            },
            event_loop,
        )
    }
    pub fn get_outputs(&self) -> Vec<Output> {
        self.globals.outputs.clone()
    }
    pub fn get_seats(&self) -> &[Seat] {
        &self.globals.as_ref().seats
    }
    fn get_index(&self, surface: &WlSurface) -> usize {
        for i in 0..self.inner.len() {
            if self.inner[i].eq(surface) {
                return i;
            }
        }
        0
    }
    fn get_application(&mut self, surface: &WlSurface) -> Option<&mut InnerApplication> {
        for inner in &mut self.inner {
            if inner.eq(surface) {
                return Some(inner)
            }
        }
        None
    }
    pub fn get_global<I>(&self) -> Result<Main<I>, GlobalError>
    where
        I: Interface + AsRef<Proxy<I>> + From<Proxy<I>>,
    {
        self.global_manager.instantiate_range::<I>(0, 1 << 8)
    }
    pub fn create_inner_application_from<Data:'static>(
        &mut self,
        config: ShellConfig,
        widget: impl Widget + 'static,
        handle: LoopHandle<'_, Data>,
        cb: impl FnMut(&mut CoreApplication, Dispatch) + 'static,
    ) {
        let dt = DrawTarget::new(widget.width() as i32, widget.height() as i32);
        let iapp = InnerApplication::new(
            widget,
            Backend::Raqote(dt),
            config,
            self.globals.clone(),
            cb,
        );
        self.inner.push(iapp);
        handle.update(&self.token).unwrap();
    }
    pub fn create_inner_application<Data:'static>(
        &mut self,
        widget: impl Widget + 'static,
        handle: LoopHandle<'_, Data>,
        cb: impl FnMut(&mut CoreApplication, Dispatch) + 'static,
    ) {
        let dt = DrawTarget::new(widget.width() as i32, widget.height() as i32);
        let iapp = InnerApplication::default(widget, Backend::Raqote(dt), self.globals.clone(), cb);
        self.inner.push(iapp);
        handle.update(&self.token).unwrap();
    }
    pub fn run(
        mut self,
        event_loop: &mut EventLoop<'static, Application>,
    ) {
        loop {
            self.display.flush().unwrap();
            event_loop.dispatch(None, &mut self).unwrap();
        }
    }
}

impl Deref for InnerApplication {
    type Target = CoreApplication;
    fn deref(&self) -> &Self::Target {
        &self.core
    }
}

impl DerefMut for InnerApplication {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.core
    }
}

impl CoreApplication {
    pub fn render(&mut self) {
        self.widget.draw(&mut self.ctx, 0., 0.);
        if let Some(pool) = self.mempool.pool() {
            if let Some(surface) = &mut self.surface {
                if let Ok((buffer, wl_buffer)) = Buffer::new(pool, &mut self.ctx) {
                    buffer.merge();
                    surface.attach_buffer(wl_buffer, 0, 0);
                }
                surface.damage(self.ctx.report_damage());
            }
        }
    }
    pub fn is_running(&self) -> bool {
        self.ctx.running
    }
    pub fn destroy(&mut self) {
        self.ctx.running = false;
        self.surface.as_mut().unwrap().destroy();
    }
    pub fn quick_roundtrip(&mut self, ev: Dispatch) {
        self.widget.roundtrip(0., 0., &mut self.ctx, &ev);
    }
    pub fn get_layer_surface(&self) -> ZwlrLayerSurfaceV1 {
        match &self.surface.as_ref().unwrap().shell {
            Shell::LayerShell { config: _, surface } => surface.detach(),
        }
    }
    pub fn replace_surface(&mut self) {
        if let Some(surface) = self.surface.as_mut() {
            surface.destroy();
            match &surface.shell {
                Shell::LayerShell { config, surface: _ } => {
                    self.surface = self.globals.as_ref().create_shell_surface_from(
                        self.widget.deref(),
                        config.clone(),
                        Some(surface.clone()),
                    );
                }
            }
        }
    }
}

impl Deref for CoreApplication {
    type Target = Context;
    fn deref(&self) -> &Self::Target {
        &self.ctx
    }
}

impl DerefMut for CoreApplication {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ctx
    }
}

impl InnerApplication {
    fn default(
        widget: impl Widget + 'static,
        backend: Backend,
        globals: Rc<Globals>,
        cb: impl FnMut(&mut CoreApplication, Dispatch) + 'static,
    ) -> Self {
        InnerApplication {
            core: CoreApplication {
                ctx: Context::new(backend),
                surface: globals.as_ref().create_shell_surface_from(
                    &widget,
                    ShellConfig::default_layer_shell(),
                    None,
                ),
                widget: Box::new(widget),
                mempool: globals.as_ref().create_mempool(),
                globals,
            },
            cb: Box::new(cb),
        }
    }
    fn new(
        widget: impl Widget + 'static,
        backend: Backend,
        config: ShellConfig,
        globals: Rc<Globals>,
        cb: impl FnMut(&mut CoreApplication, Dispatch) + 'static,
    ) -> Self {
        InnerApplication {
            core: CoreApplication {
                ctx: Context::new(backend),
                surface: globals
                    .as_ref()
                    .create_shell_surface_from(&widget, config, None),
                widget: Box::new(widget),
                mempool: globals.as_ref().create_mempool(),
                globals,
            },
            cb: Box::new(cb),
        }
    }
    fn eq(&self, wl_surface: &WlSurface) -> bool {
        if let Some(surface) = &self.surface {
            return surface.surface.detach().eq(wl_surface);
        }
        false
    }
    pub fn dispatch(&mut self, ev: Dispatch) {
        let (w, h) = (self.widget.width(), self.widget.height());
        self.core.widget.roundtrip(0., 0., &mut self.core.ctx, &ev);

        let mut show = true;
        match self.core.ctx.damage_type() {
            DamageType::Full => {
                if self.widget.width() != w || self.widget.height() != h {
                    self.core.ctx.resize(self.widget.width() as i32, self.widget.height() as i32);
                    if let Some(surface) = &self.surface {
                        surface.set_size(self.widget.width() as u32, self.widget.height() as u32);
                    }
                }
                self.core.widget.draw(&mut self.core.ctx, 0., 0.);
                self.core
                    .surface
                    .as_ref()
                    .unwrap()
                    .region
                    .subtract(0, 0, 1 << 30, 1 << 30);
            }
            DamageType::Partial => {
                if !self.core.ctx.is_damaged() {
                    show = false; }
            }
            DamageType::Resize => {
                self.core.ctx.resize(self.widget.width() as i32, self.widget.height() as i32);
                if let Some(surface) = &self.surface {
                    surface.set_size(self.widget.width() as u32, self.widget.height() as u32);
                    self.core.widget.draw(&mut self.core.ctx, 0., 0.);
                }
            }
        }

        (self.cb)(&mut self.core, ev);

        if self.core.ctx.running && show {
            if let Some(pool) = self.core.mempool.pool() {
                if let Some(surface) = &mut self.core.surface {
                    // surface.add_input(self.core.ctx.report_input());
                    if let Ok((buffer, wl_buffer)) = Buffer::new(pool, &mut self.core.ctx) {
                        buffer.merge();
                        surface.attach_buffer(wl_buffer, 0, 0);
                    }
                    surface.damage(self.core.ctx.report_damage());
                    surface.commit();
                }
            }
        } else if !self.core.ctx.running {
            if let Some(surface) = &mut self.core.surface {
                surface.destroy();
            }
        }

        self.ctx.flush();
    }
}


impl Modifiers {
    fn from(modifer_state: ModifiersState) -> Modifiers {
        Modifiers {
            ctrl: modifer_state.ctrl,
            alt: modifer_state.alt,
            shift: modifer_state.shift,
            caps_lock: modifer_state.caps_lock,
            logo: modifer_state.logo,
            num_lock: modifer_state.num_lock,
        }
    }
}

/* TO-DO
fn assign_keyboard(keyboard: &Main<WlKeyboard>) {
    let mut index = 0;
    let mut kb_key: Option<Key> = None;
    keyboard.quick_assign(move |_, event, mut inner| match event {
        wl_keyboard::Event::Leave { serial, surface } => {
        }
        wl_keyboard::Event::Modifiers {
            serial,
            mods_depressed,
            mods_latched,
            mods_locked,
            group,
        } => {}
        wl_keyboard::Event::Enter {
            serial,
            surface,
            keys,
        } => {
            if let Some(inner) = inner.get::<Vec<InnerApplication>>() {
                index = Application::get_index(inner, &surface);
            }
        }
        wl_keyboard::Event::Key {
            serial: _,
            time,
            key,
            state,
        } => {
            if let Some(inner) = inner.get::<Vec<InnerApplication>>() {
            }
        }
        wl_keyboard::Event::RepeatInfo { rate, delay } => {}
        _ => {}
    });
}
*/

fn assign_pointer(pointer: &Main<WlPointer>) {
    let mut index = 0;
    let mut input = Pointer::Enter;
    let (mut x, mut y) = (0., 0.);
    pointer.quick_assign(move |_, event, mut inner| match event {
        wl_pointer::Event::Leave { serial: _, surface } => {
            input = Pointer::Leave;
            if let Some(application) = inner.get::<Application>() {
                if let Some(inner) = application.get_application(&surface) {
                    inner.dispatch(Dispatch::Pointer(x as f32, y as f32, input));
                }
            }
        }
        wl_pointer::Event::Button {
            serial: _,
            time,
            button,
            state,
        } => {
            input = Pointer::MouseClick {
                time,
                button: MouseButton::new(button),
                pressed: state == wl_pointer::ButtonState::Pressed,
            };
        }
        wl_pointer::Event::Frame => {
            if let Some(application) = inner.get::<Application>() {
                application.inner[index].dispatch(Dispatch::Pointer(x as f32, y as f32, input));
            }
        }
        wl_pointer::Event::Enter {
            serial: _,
            surface,
            surface_x,
            surface_y,
        } => {
            if let Some(application) = inner.get::<Application>() {
                x = surface_x;
                y = surface_y;
                index = application.get_index(&surface);
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
        _ => {}
    });
}

fn assign_surface(shell: &Main<ZwlrLayerSurfaceV1>) {
    shell.quick_assign(|shell, event, mut inner| match event {
        zwlr_layer_surface_v1::Event::Configure {
            serial,
            width: _,
            height: _,
        } => {
            shell.ack_configure(serial);
            if let Some(application) = inner.get::<Application>() {
                for a in &mut application.inner {
                    if let Some(surface) = &a.surface {
                        match &surface.shell {
                            Shell::LayerShell { config: _, surface } => {
                                if shell.eq(surface) {
                                    a.full_damage();
                                    a.dispatch(Dispatch::Commit);
                                }
                            }
                        }
                    }
                }
            }
        }
        _ => unreachable!(),
    });
}
