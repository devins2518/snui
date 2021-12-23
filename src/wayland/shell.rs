use crate::context::DrawContext;
use crate::data::Controller;
use crate::font::FontCache;
use crate::scene::*;
use crate::wayland::*;
use crate::*;
use smithay_client_toolkit::reexports::calloop::{EventLoop, LoopHandle, RegistrationToken};
use smithay_client_toolkit::seat::keyboard::{
    ModifiersState,
};
use smithay_client_toolkit::shm::AutoMemPool;
use smithay_client_toolkit::WaylandSource;

use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use wayland_client::protocol::wl_buffer::WlBuffer;
use wayland_client::protocol::wl_compositor::WlCompositor;

use wayland_client::protocol::wl_callback;
use wayland_client::protocol::wl_output::{self, WlOutput};
use wayland_client::protocol::wl_pointer::{self, WlPointer};
use wayland_client::protocol::wl_region::WlRegion;
use wayland_client::protocol::wl_seat::{self, Capability, WlSeat};
use wayland_client::protocol::wl_shm::WlShm;
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_client::{Attached, Display, GlobalError, GlobalManager, Interface, Main, Proxy};
use wayland_protocols::wlr::unstable::layer_shell::v1::client::{
    zwlr_layer_shell_v1::Layer, zwlr_layer_shell_v1::ZwlrLayerShellV1, zwlr_layer_surface_v1,
    zwlr_layer_surface_v1::Anchor, zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
};

pub struct Application<C: Controller + Clone + 'static> {
    display: Display,
    globals: Rc<Globals>,
    global_manager: GlobalManager,
    pub inner: Vec<InnerApplication<C>>,
    token: RegistrationToken,
}

struct Context {
    pending_cb: bool,
    time: Option<u32>,
    render_node: Option<RenderNode>,
    font_cache: FontCache,
}

pub struct CoreApplication<C: Controller + Clone> {
    pub controller: C,
    ctx: Context,
    globals: Rc<Globals>,
    mempool: AutoMemPool,
    widget: Box<dyn Widget>,
    surface: Option<Surface>,
}

pub struct InnerApplication<C: Controller + Clone> {
    core: CoreApplication<C>,
    cb: Box<dyn FnMut(&mut CoreApplication<C>, Event)>,
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
        self.buffer = None;
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
    fn damage(&self, report: &[Region]) {
        self.surface.attach(self.buffer.as_ref(), 0, 0);
        for d in report {
            self.surface
                .damage(d.x as i32, d.y as i32, d.width as i32, d.height as i32);
        }
    }
    fn attach_buffer(&mut self, buffer: WlBuffer) {
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
    pub fn create_shell_surface<C: Controller + Clone + 'static>(
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
            assign_surface::<C>(&shell);
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
    pub fn create_shell_surface_from<C: Controller + Clone + 'static>(
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
            assign_surface::<C>(&shell);
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
    pub fn create_mempool(&self) -> AutoMemPool {
        let attached = Attached::from(self.shm.clone().unwrap());
        AutoMemPool::new(attached).unwrap()
    }
    pub fn get_outputs(&self) -> Vec<Output> {
        self.outputs.clone()
    }
    pub fn get_seats(&self) -> &[Seat] {
        &self.seats
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

impl<C: Controller + Clone + 'static> Application<C> {
    pub fn new(pointer: bool) -> (Self, EventLoop<'static, Self>) {
        let display = Display::connect_to_env().unwrap();
        let event_queue = display.create_event_queue();
        let attached_display = (*display).clone().attach(event_queue.token());

		let display_handle = display.clone();

        let globals = Globals::new();

        let global_manager = GlobalManager::new_with_cb(
            &attached_display,
            wayland_client::global_filter!(
                [
                    ZwlrLayerShellV1,
                    1,
                    |shell: Main<ZwlrLayerShellV1>, mut application: DispatchData| {
                        if let Some(application) = application.get::<Application<C>>() {
                            Rc::get_mut(&mut application.globals).unwrap().shell = Some(shell);
                        }
                    }
                ],
                [WlShm, 1, |shm: Main<WlShm>, mut application: DispatchData| {
                    shm.quick_assign(|_, _, _| {});
                    if let Some(application) = application.get::<Application<C>>() {
                        Rc::get_mut(&mut application.globals).unwrap().shm = Some(shm);
                    }
                }],
                [
                    WlCompositor,
                    4,
                    |compositor: Main<WlCompositor>, mut application: DispatchData| {
                        if let Some(application) = application.get::<Application<C>>() {
                            Rc::get_mut(&mut application.globals).unwrap().compositor = Some(compositor);
                        }
                    }
                ],
                [WlSeat, 7, move |seat: Main<WlSeat>, _: DispatchData| {
                    seat.quick_assign(move |seat, event, mut application| match event {
                        wl_seat::Event::Capabilities { capabilities } => {
                            if let Some(application) = application.get::<Application<C>>() {
                                if pointer && capabilities & Capability::Pointer == Capability::Pointer {
                                    let pointer = seat.get_pointer();
                                    assign_pointer::<C>(&pointer);
                                }
                                Rc::get_mut(&mut application.globals).unwrap().seats.push(Seat { capabilities, seat });
                            }
                        }
                        _ => {}
                    });
                }],
                [
                    WlOutput,
                    3,
                    |output: Main<WlOutput>, _application: DispatchData| {
                        output.quick_assign(move |wl_output, event, mut application| match event {
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
                                if let Some(application) = application.get::<Application<C>>() {
                                    if let Some(output) = Rc::get_mut(&mut application.globals).unwrap().outputs.last_mut() {
                                        if wl_output == output.output {
                                            output.name = make;
                                        } else {
                                            let mut output = Output::new(wl_output);
                                            output.name = make;
                                            Rc::get_mut(&mut application.globals).unwrap().outputs.push(output);
                                        }
                                    } else {
                                        let mut output = Output::new(wl_output);
                                        output.name = make;
                                        Rc::get_mut(&mut application.globals).unwrap().outputs.push(output);
                                    }
                                }
                            }
                            wl_output::Event::Mode {
                                flags: _,
                                width,
                                height,
                                refresh: _,
                            } => {
                                if let Some(application) = application.get::<Application<C>>() {
                                    if let Some(output) = Rc::get_mut(&mut application.globals).unwrap().outputs.last_mut() {
                                        if wl_output == output.output {
                                            output.width = width;
                                            output.height = height;
                                        } else {
                                            let mut output = Output::new(wl_output);
                                            output.width = width;
                                            output.height = height;
                                            Rc::get_mut(&mut application.globals).unwrap().outputs.push(output);
                                        }
                                    } else {
                                        let mut output = Output::new(wl_output);
                                        output.width = width;
                                        output.height = height;
                                        Rc::get_mut(&mut application.globals).unwrap().outputs.push(output);
                                    }
                                }
                            }
                            wl_output::Event::Scale { factor } => {
                                if let Some(application) = application.get::<Application<C>>() {
                                    if let Some(output) = Rc::get_mut(&mut application.globals).unwrap().outputs.last_mut() {
                                        if wl_output == output.output {
                                            output.scale = factor;
                                        } else {
                                            let mut output = Output::new(wl_output);
                                            output.scale = factor;
                                            Rc::get_mut(&mut application.globals).unwrap().outputs.push(output);
                                        }
                                    } else {
                                        let mut output = Output::new(wl_output);
                                        output.scale = factor;
                                        Rc::get_mut(&mut application.globals).unwrap().outputs.push(output);
                                    }
                                }
                            }
                            wl_output::Event::Done => {}
                            _ => {}
                        });
                    }
                ]
            ),
        );

        let event_loop = EventLoop::try_new().expect("Failed to initialize the event loop!");
        let token = WaylandSource::new(event_queue)
            .quick_insert(event_loop.handle())
            .unwrap();

        let (mut application, mut event_loop)= (
            Application {
                display,
                globals: Rc::new(globals),
                global_manager,
                inner: Vec::new(),
                token,
            },
            event_loop,
        );

		for _ in 0..2 {
            display_handle.flush().unwrap();
            event_loop.dispatch(None, &mut application).unwrap();
		}

        (application, event_loop)
    }
    fn get_index(&self, surface: &WlSurface) -> usize {
        for i in 0..self.inner.len() {
            if self.inner[i].eq(surface) {
                return i;
            }
        }
        0
    }
    fn get_application(&mut self, surface: &WlSurface) -> Option<&mut InnerApplication<C>> {
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
        controller: C,
        widget: impl Widget + 'static,
        handle: LoopHandle<'_, Data>,
        cb: impl FnMut(&mut CoreApplication<C>, Event) + 'static,
    ) {
        let inner_application = InnerApplication::empty(controller, widget, self.globals.clone(), cb);
        self.inner.push(inner_application);
        handle.update(&self.token).unwrap();
    }
    pub fn create_inner_application_from<Data: 'static>(
        &mut self,
        controller: C,
        config: ShellConfig,
        widget: impl Widget + 'static,
        handle: LoopHandle<'_, Data>,
        cb: impl FnMut(&mut CoreApplication<C>, Event) + 'static,
    ) {
        let inner_application = InnerApplication::new(controller, widget, config, self.globals.clone(), cb);
        self.inner.push(inner_application);
        handle.update(&self.token).unwrap();
    }
    pub fn create_inner_application<Data: 'static>(
        &mut self,
        controller: C,
        widget: impl Widget + 'static,
        handle: LoopHandle<'_, Data>,
        cb: impl FnMut(&mut CoreApplication<C>, Event) + 'static,
    ) {
        let inner_application = InnerApplication::default(controller, widget, self.globals.clone(), cb);
        self.inner.push(inner_application);
        handle.update(&self.token).unwrap();
    }
    pub fn run(mut self, event_loop: &mut EventLoop<'static, Self>) {
        loop {
            self.display.flush().unwrap();
            event_loop.dispatch(None, &mut self).unwrap();
        }
    }
}

impl<C: Controller + Clone> Deref for InnerApplication<C> {
    type Target = CoreApplication<C>;
    fn deref(&self) -> &Self::Target {
        &self.core
    }
}

impl<C: Controller + Clone> DerefMut for InnerApplication<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.core
    }
}

impl<C: Controller + Clone + 'static> CoreApplication<C> {
    fn sync(&mut self, ev: Event) -> bool {
        let mut sync_ctx = SyncContext::new(&mut self.controller, &mut self.ctx.font_cache);
        let mut damage = self.widget.sync(&mut sync_ctx, ev);
        while let Ok(signal) = sync_ctx.sync() {
            damage = damage.max(self.widget.sync(&mut sync_ctx, Event::Message(signal)));
        }
        if damage == Damage::Frame {
            if self.ctx.time.is_none() {
                self.ctx.time = Some(0);
            }
        }
        damage.is_some() && !self.ctx.pending_cb
    }
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
                    self.surface = self.globals.as_ref().create_shell_surface_from::<C>(
                        self.widget.deref(),
                        config.clone(),
                        Some(surface.clone()),
                    );
                }
            }
        } else {
            self.surface = self.globals.as_ref().create_shell_surface_from::<C>(
                self.widget.deref(),
                ShellConfig::default_layer_shell(),
                None,
            );
        }
    }
    pub fn replace_surface_by(&mut self, config: ShellConfig) {
        if let Some(surface) = self.surface.as_mut() {
            surface.destroy();
            self.surface = self.globals.as_ref().create_shell_surface_from::<C>(
                self.widget.deref(),
                config,
                Some(surface.clone()),
            );
        } else {
            self.surface = self.globals.as_ref().create_shell_surface_from::<C>(
                self.widget.deref(),
                config,
                None,
            );
        }
    }
}

impl<C: Controller + Clone> Geometry for CoreApplication<C> {
    fn width(&self) -> f32 {
        self.widget.width()
    }
    fn height(&self) -> f32 {
        self.widget.height()
    }
    fn set_size(&mut self, width: f32, height: f32) -> Result<(), (f32, f32)> {
        if let Some(surface) = self.surface.as_ref() {
            surface.set_size(width as u32, height as u32);
            surface.surface.commit();
        }
        Ok(())
    }
}

impl<C: Controller + Clone + 'static> InnerApplication<C> {
    fn empty(
        controller: C,
        widget: impl Widget + 'static,
        globals: Rc<Globals>,
        cb: impl FnMut(&mut CoreApplication<C>, Event) + 'static,
    ) -> Self {
        InnerApplication {
            core: CoreApplication {
                controller,
                ctx: Context {
                    pending_cb: false,
                    time: None,
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
        controller: C,
        widget: impl Widget + 'static,
        globals: Rc<Globals>,
        cb: impl FnMut(&mut CoreApplication<C>, Event) + 'static,
    ) -> Self {
        let mut default = InnerApplication {
            core: CoreApplication {
                controller,
                ctx: Context {
                    pending_cb: false,
                    time: None,
                    font_cache: FontCache::new(),
                    render_node: None,
                },
                surface: None,
                widget: Box::new(widget),
                mempool: globals.as_ref().create_mempool(),
                globals,
            },
            cb: Box::new(cb),
        };
        default.sync(Event::Prepare);
        default.replace_surface();
        default
    }
    fn new(
        controller: C,
        widget: impl Widget + 'static,
        config: ShellConfig,
        globals: Rc<Globals>,
        cb: impl FnMut(&mut CoreApplication<C>, Event) + 'static,
    ) -> Self {
        let mut new = InnerApplication {
            core: CoreApplication {
                controller,
                ctx: Context {
                    pending_cb: false,
                    time: None,
                    font_cache: FontCache::new(),
                    render_node: None,
                },
                surface: None,
                widget: Box::new(widget),
                mempool: globals.as_ref().create_mempool(),
                globals,
            },
            cb: Box::new(cb),
        };
        new.sync(Event::Prepare);
        new.replace_surface_by(config);
        new
    }
    fn eq(&self, wl_surface: &WlSurface) -> bool {
        if let Some(surface) = &self.surface {
            return surface.surface.detach().eq(wl_surface);
        }
        false
    }
    pub fn roundtrip(&mut self, ev: Event) -> Result<RenderNode, ()> {
        let width = self.width();
        let height = self.height();

        // Sending the event to the widget tree
        if self.sync(ev) || ev == Event::Frame {
            // Calling the application´s closure
            (self.cb)(&mut self.core, ev);

            let current_width = self.width();
            let current_height = self.height();

            // Resizing the surface in case the widget changed size
            if ev == Event::Frame {
                self.ctx.render_node = None;
            } else if width != current_width || height != current_height {
                let _ = self.set_size(current_width, current_height);
                return Err(());
            }

            // Creating the render node
            let render_node = self.core.widget.create_node(0., 0.);

            self.ctx.pending_cb = true;

            return Ok(render_node);
        } else {
            // Calling the application´s closure
            (self.cb)(&mut self.core, ev);
        }

        Err(())
    }
    fn render(&mut self, time: u32, recent_node: RenderNode) {
        let width = recent_node.width();
        let height = recent_node.height();
        if let Ok((buffer, wl_buffer)) =
            Buffer::new(&mut self.core.mempool, width as i32, height as i32)
        {
            let mut v = Vec::new();
            let mut ctx = DrawContext::new(buffer.backend, &mut self.core.ctx.font_cache, &mut v);
            if let Some(render_node) = self.core.ctx.render_node.as_mut() {
                if let Err(region) = render_node.draw_merge(
                    recent_node,
                    &mut ctx,
                    &Instruction::empty(0., 0., width, height),
                    None,
                ) {
                    ctx.damage_region(&Background::Transparent, region, false);
                }
            } else {
                ctx.damage_region(
                    &Background::Transparent,
                    Region::new(0., 0., width, height),
                    false,
                );
                recent_node.render(&mut ctx, None);
                self.core.ctx.render_node = Some(recent_node);
            }
            self.core.ctx.pending_cb = false;
            if let Some(surface) = self.core.surface.as_mut() {
                surface.attach_buffer(wl_buffer);
                surface.damage(&v);
                surface.commit();
                if let Some(_) = self.core.ctx.time {
                    self.core.ctx.time = Some(time);
                    frame_callback::<C>(time, surface.surface.clone());
                }
            }
        }
    }
    pub fn callback(&mut self, ev: Event) {
        if self.ctx.time.is_none() || ev.is_cb() {
            if let Ok(render_node) = self.roundtrip(ev) {
                draw_callback::<C>(&self.surface.as_ref().unwrap().surface, render_node);
            }
        } else {
            self.sync(ev);
        }
    }
}

fn frame_callback<C: Controller + Clone + 'static>(time: u32, surface: Main<WlSurface>) {
    let h = surface.detach();
    surface
        .frame()
        .quick_assign(move |_, event, mut application| match event {
            wl_callback::Event::Done { callback_data } => {
                let timeout = (callback_data - time).min(50);
                if let Some(application) = application.get::<Application<C>>() {
                    let inner_application = application.get_application(&h).unwrap();
                    inner_application.ctx.time = None;
                    inner_application.callback(Event::Callback(timeout));
                }
            }
            _ => {}
        });
    surface.commit();
}

fn draw_callback<C: Controller + Clone + 'static>(
    surface: &Main<WlSurface>,
    mut recent_node: RenderNode,
) {
    let h = surface.detach();
    surface
        .frame()
        .quick_assign(move |_, event, mut application| match event {
            wl_callback::Event::Done { callback_data } => {
                if let Some(application) = application.get::<Application<C>>() {
                    let inner_application = application.get_application(&h).unwrap();
                    inner_application.render(callback_data, std::mem::take(&mut recent_node));
                }
            }
            _ => {}
        });
    surface.commit();
}

impl From<ModifiersState> for Modifiers {
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

fn assign_pointer<C: Controller + Clone + 'static>(pointer: &Main<WlPointer>) {
    let mut index = 0;
    let mut input = Pointer::Enter;
    let (mut x, mut y) = (0., 0.);
    pointer.quick_assign(move |_, event, mut inner| match event {
        wl_pointer::Event::Leave { serial: _, surface } => {
            input = Pointer::Leave;
            if let Some(application) = inner.get::<Application<C>>() {
                if let Some(inner_application) = application.get_application(&surface) {
                    inner_application.callback(Event::Pointer(x as f32, y as f32, input));
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
            if let Some(application) = inner.get::<Application<C>>() {
                let inner_application = application.inner.get_mut(index).unwrap();
                inner_application.callback(Event::Pointer(x as f32, y as f32, input));
            }
        }
        wl_pointer::Event::Axis {
            time: _,
            axis,
            value,
        } => {
            input = Pointer::Scroll {
                orientation: match axis {
                    wl_pointer::Axis::VerticalScroll => Orientation::Vertical,
                    wl_pointer::Axis::HorizontalScroll => Orientation::Horizontal,
                    _ => Orientation::Vertical,
                },
                value: value as f32,
            }
        }
        wl_pointer::Event::Enter {
            serial: _,
            surface,
            surface_x,
            surface_y,
        } => {
            if let Some(application) = inner.get::<Application<C>>() {
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

fn assign_surface<C: Controller + Clone + 'static>(shell: &Main<ZwlrLayerSurfaceV1>) {
    shell.quick_assign(|shell, event, mut inner| match event {
        zwlr_layer_surface_v1::Event::Configure {
            serial,
            width,
            height,
        } => {
            shell.ack_configure(serial);
            if let Some(application) = inner.get::<Application<C>>() {
                for inner_application in &mut application.inner {
                    if let Some(app_surface) = inner_application.surface.as_mut() {
                        match &app_surface.shell {
                            Shell::LayerShell { config: _, surface } => {
                                if shell.eq(surface) {
                                    app_surface.destroy_previous();
                                    let _ = inner_application.widget.set_size(width as f32, height as f32);
                                    if inner_application.ctx.pending_cb {
                                        if let Ok(render_node) = inner_application.roundtrip(Event::Frame) {
                                            draw_callback::<C>(
                                                &inner_application.surface.as_ref().unwrap().surface,
                                                render_node,
                                            );
                                        }
                                    } else {
                                        if let Ok(render_node) = inner_application.roundtrip(Event::Frame) {
                                            inner_application.render(0, render_node);
                                        }
                                    }
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
