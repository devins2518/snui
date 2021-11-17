use crate::context::Backend;
use crate::context::DrawContext;
use crate::scene::*;
use crate::widgets::font::FontCache;
use crate::wayland::Buffer;
use crate::*;
use raqote::*;
use smithay_client_toolkit::reexports::calloop::{EventLoop, LoopHandle, RegistrationToken};
use smithay_client_toolkit::seat::keyboard::{
    self, map_keyboard_repeat, ModifiersState, RepeatKind,
};
use smithay_client_toolkit::shm::DoubleMemPool;
use smithay_client_toolkit::WaylandSource;

use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use wayland_client::protocol::wl_buffer::WlBuffer;
use wayland_client::protocol::wl_compositor::WlCompositor;

use wayland_client::protocol::wl_output::{self, WlOutput};
use wayland_client::protocol::wl_pointer::{self, WlPointer};
use wayland_client::protocol::wl_region::WlRegion;
use wayland_client::protocol::wl_seat::{self, Capability, WlSeat};
use wayland_client::protocol::wl_shm::WlShm;
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_client::{Attached, Display, GlobalError, GlobalManager, Interface, Main, Proxy};
use wayland_protocols::wlr::unstable::layer_shell::v1::client::{
    zwlr_layer_shell_v1::Layer, zwlr_layer_shell_v1::ZwlrLayerShellV1, zwlr_layer_surface_v1,
    zwlr_layer_surface_v1::Anchor, zwlr_layer_surface_v1::KeyboardInteractivity,
    zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
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
    alive: bool,
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

struct Context {
    draw_target: DrawTarget,
    render_node: Option<RenderNode>,
    font_cache: crate::widgets::font::FontCache,
}

impl Context {
    fn sync_ctx(&mut self) -> SyncContext<'_> {
        SyncContext::new(self.render_node.as_mut(), &mut self.font_cache)
    }
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
    cb: Box<dyn FnMut(&mut CoreApplication, Event)>,
}

impl Surface {
    fn new(
        surface: Main<WlSurface>,
        shell: Shell,
        region: Main<WlRegion>,
        previous: Option<Surface>,
    ) -> Self {
        Surface {
            alive: true,
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
        self.alive = false;
        self.surface.destroy();
        self.region.destroy();
        self.shell.destroy();
        self.buffer = None;
    }
    fn destroy_previous(&mut self) {
        if let Some(surface) = self.previous.as_mut() {
            surface.destroy();
        }
        self.previous = None;
    }
    fn set_size(&self, width: u32, height: u32) {
        self.shell.set_size(width, height);
    }
    // fn add_input(&self, _report: &[Region]) {
    // if !report.is_empty() {
    //     for r in report {
    //         self.region
    //             .add(r.x as i32, r.y as i32, r.width as i32, r.height as i32);
    //     }
    //     self.surface.set_input_region(Some(&self.region));
    // } else {
    //     self.surface.set_input_region(Some(&self.region));
    // }
    // }
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
    fn is_alive(&self) -> bool {
        self.alive
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
    pub fn new(pointer: bool) -> (Self, EventLoop<'static, Application>) {
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
            if false && (seat.capabilities & Capability::Keyboard == Capability::Keyboard) {
                let _ = map_keyboard_repeat(
                    event_loop.handle(),
                    &Attached::from(seat.seat.clone()),
                    None,
                    RepeatKind::System,
                    move |ev, _, mut inner| match ev {
                        keyboard::Event::Modifiers { modifiers } => {
                            if let Some(application) = inner.get::<Application>() {
                                application.inner[index].dispatch(Event::Keyboard(Key {
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
                            keysyms: _,
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
                                application.inner[index].dispatch(Event::Keyboard(Key {
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
                            utf8: _,
                        } => {
                            if let Some(application) = inner.get::<Application>() {
                                application.inner[index].dispatch(Event::Keyboard(Key {
                                    utf8: ch.as_ref(),
                                    value: &[keysym],
                                    modifiers: Modifiers::default(),
                                    pressed: true,
                                }));
                            }
                        }
                        _ => {}
                    },
                )
                .unwrap();
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
                return Some(inner);
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
    pub fn create_empty_inner_application<Data: 'static>(
        &mut self,
        widget: impl Widget + 'static,
        handle: LoopHandle<'_, Data>,
        cb: impl FnMut(&mut CoreApplication, Event) + 'static,
    ) {
        let iapp = InnerApplication::empty(widget, self.globals.clone(), cb);
        self.inner.push(iapp);
        handle.update(&self.token).unwrap();
    }
    pub fn create_inner_application_from<Data: 'static>(
        &mut self,
        config: ShellConfig,
        widget: impl Widget + 'static,
        handle: LoopHandle<'_, Data>,
        cb: impl FnMut(&mut CoreApplication, Event) + 'static,
    ) {
        let iapp = InnerApplication::new(
            widget,
            config,
            self.globals.clone(),
            cb,
        );
        self.inner.push(iapp);
        handle.update(&self.token).unwrap();
    }
    pub fn create_inner_application<Data: 'static>(
        &mut self,
        widget: impl Widget + 'static,
        handle: LoopHandle<'_, Data>,
        cb: impl FnMut(&mut CoreApplication, Event) + 'static,
    ) {
        let iapp = InnerApplication::default(widget, self.globals.clone(), cb);
        self.inner.push(iapp);
        handle.update(&self.token).unwrap();
    }
    pub fn run(mut self, event_loop: &mut EventLoop<'static, Application>) {
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
    // pub fn render(&mut self) {
    //     if let Some(pool) = self.mempool.pool() {
    //         if let Some(surface) = &mut self.surface {
    //             if let Ok((buffer, wl_buffer)) = Buffer::new(pool, &mut self.ctx) {
    //                 buffer.merge();
    //                 surface.attach_buffer(wl_buffer, 0, 0);
    //             }
    //             surface.damage(self.ctx.report_damage());
    //         }
    //     }
    // }
    pub fn destroy(&mut self) {
        if let Some(surface) = self.surface.as_mut() {
            surface.destroy();
        }
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
        } else {
            self.surface = self.globals.as_ref().create_shell_surface_from(
                self.widget.deref(),
                ShellConfig::default_layer_shell(),
                None,
            );
        }
    }
    pub fn load_font(&mut self, name: &str, path: &std::path::Path) {
        self.ctx.font_cache.load_font(name, path);
    }
    pub fn replace_surface_by(&mut self, config: ShellConfig) {
        if let Some(surface) = self.surface.as_mut() {
            surface.destroy();
            self.surface = self.globals.as_ref().create_shell_surface_from(
                self.widget.deref(),
                config,
                Some(surface.clone()),
            );
        } else {
            self.surface =
                self.globals
                    .as_ref()
                    .create_shell_surface_from(self.widget.deref(), config, None);
        }
    }
}

impl Geometry for CoreApplication {
    fn width(&self) -> f32 {
        self.widget.width()
    }
    fn height(&self) -> f32 {
        self.widget.height()
    }
    fn set_size(&mut self, width: f32, height: f32) -> Result<(), (f32, f32)> {
        if let Some(surface) = self.surface.as_ref() {
            surface.set_size(width as u32, height as u32);
        }
        self.ctx.draw_target = DrawTarget::new(width as i32, height as i32);
        Ok(())
    }
}

impl InnerApplication {
    fn empty(
        widget: impl Widget + 'static,
        globals: Rc<Globals>,
        cb: impl FnMut(&mut CoreApplication, Event) + 'static,
    ) -> Self {
        InnerApplication {
            core: CoreApplication {
                ctx: Context {
                    draw_target: DrawTarget::new(widget.width() as i32, widget.height() as i32),
                    font_cache: FontCache::new(),
                    render_node: None,
                },
                surface: None,
                widget: Box::new(widget),
                mempool: globals.as_ref().create_mempool(),
                globals,
            },
            cb: Box::new(cb),
        }
    }
    fn default(
        widget: impl Widget + 'static,
        globals: Rc<Globals>,
        cb: impl FnMut(&mut CoreApplication, Event) + 'static,
    ) -> Self {
        InnerApplication {
            core: CoreApplication {
                ctx: Context {
                    draw_target: DrawTarget::new(widget.width() as i32, widget.height() as i32),
                    font_cache: FontCache::new(),
                    render_node: None,
                },
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
        config: ShellConfig,
        globals: Rc<Globals>,
        cb: impl FnMut(&mut CoreApplication, Event) + 'static,
    ) -> Self {
        InnerApplication {
            core: CoreApplication {
                ctx: Context {
                    draw_target: DrawTarget::new(widget.width() as i32, widget.height() as i32),
                    font_cache: FontCache::new(),
                    render_node: None,
                },
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
    pub fn dispatch(&mut self, ev: Event) {
        let render;

        // Sending the event to the widget tree
        self.core.widget.sync(&mut self.core.ctx.sync_ctx(), ev);

        // Creating the render node
        let recent_node = self.core.widget.create_node(0., 0.);

        // Getting the new size of the widget
        let width = self.width();
        let height = self.height();
        let mut v = Vec::new();

		// Resizing the surface in case the widget changed size
        if self.core.ctx.draw_target.width() != width as i32
        || self.core.ctx.draw_target.height() != height as i32 {
            self.core.set_size(width, height).unwrap();
        }

        if let Some(render_node) = &self.core.ctx.render_node {
            render_node.find_diff(
                &recent_node,
                &mut DrawContext::new(
                    Backend::Raqote(&mut self.core.ctx.draw_target),
                    &mut self.core.ctx.font_cache,
                    &mut v
                ),
                &Background::Transparent
            );
            render = false;
        } else {
            recent_node.render(
            &mut DrawContext::new(
                    Backend::Raqote(&mut self.core.ctx.draw_target),
                    &mut self.core.ctx.font_cache,
                    &mut v
                )
            );
            self.core.ctx.render_node = Some(recent_node);
            render = true;
        }

        // Calling the application´s closure
        (self.cb)(&mut self.core, ev);

        if render && self.surface.is_some() {
            if let Some(pool) = self.core.mempool.pool() {
                if let Some(surface) = &mut self.core.surface {
                    if let Ok((buffer, wl_buffer)) = Buffer::new(
                        pool,
                        DrawContext::new(
                            Backend::Raqote(&mut self.core.ctx.draw_target),
                            &mut self.core.ctx.font_cache,
                            &mut v
                        )
                    ) {
                        buffer.merge();
                        surface.attach_buffer(wl_buffer, 0, 0);
                        surface.damage(&v);
                        surface.commit();
                    }
                }
            }
        }
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

fn assign_pointer(pointer: &Main<WlPointer>) {
    let mut index = 0;
    let mut input = Pointer::Enter;
    let (mut x, mut y) = (0., 0.);
    pointer.quick_assign(move |_, event, mut inner| match event {
        wl_pointer::Event::Leave { serial: _, surface } => {
            input = Pointer::Leave;
            if let Some(application) = inner.get::<Application>() {
                if let Some(inner) = application.get_application(&surface) {
                    inner.dispatch(Event::Pointer(x as f32, y as f32, input));
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
                application.inner[index].dispatch(Event::Pointer(x as f32, y as f32, input));
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
                    if let Some(app_surface) = a.surface.as_mut() {
                        match &app_surface.shell {
                            Shell::LayerShell { config: _, surface } => {
                                if shell.eq(surface) {
                                    app_surface.destroy_previous();
                                    a.dispatch(Event::Commit);
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
