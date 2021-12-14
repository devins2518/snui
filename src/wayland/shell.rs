use crate::context::DrawContext;
use crate::data::Controller;
use crate::font::FontCache;
use crate::scene::*;
use crate::wayland::*;
use crate::*;
use smithay_client_toolkit::reexports::calloop::{EventLoop, LoopHandle, RegistrationToken};
use smithay_client_toolkit::seat::keyboard::{
    self, map_keyboard_repeat, ModifiersState, RepeatKind,
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
        let mut event_queue = display.create_event_queue();
        let attached_display = (*display).clone().attach(event_queue.token());

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
                    move |ev, _, mut applications| match ev {
                        keyboard::Event::Modifiers { modifiers } => {
                            if let Some(applications) = applications.get::<Application<C>>() {
                                let inner_application = applications.inner.get_mut(index).unwrap();
                                if let Ok(render_node) =
                                    inner_application.roundtrip(Event::Keyboard(Key {
                                        utf8: ch.as_ref(),
                                        value: &[],
                                        modifiers: Modifiers::from(modifiers),
                                        pressed: true,
                                    }))
                                {
                                    draw_callback::<C>(
                                        &inner_application.surface.as_ref().unwrap().surface,
                                        render_node,
                                    );
                                }
                            }
                        }
                        keyboard::Event::Enter {
                            serial: _,
                            surface,
                            rawkeys: _,
                            keysyms: _,
                        } => {
                            if let Some(applications) = applications.get::<Application<C>>() {
                                index = applications.get_index(&surface);
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
                            if let Some(applications) = applications.get::<Application<C>>() {
                                let inner_application = applications.inner.get_mut(index).unwrap();
                                if let Ok(render_node) =
                                    inner_application.roundtrip(Event::Keyboard(Key {
                                        utf8: ch.as_ref(),
                                        value: &[rawkey],
                                        modifiers: Modifiers::default(),
                                        pressed: state == keyboard::KeyState::Pressed,
                                    }))
                                {
                                    draw_callback::<C>(
                                        &inner_application.surface.as_ref().unwrap().surface,
                                        render_node,
                                    );
                                }
                            }
                        }
                        keyboard::Event::Repeat {
                            time: _,
                            rawkey: _,
                            keysym,
                            utf8: _,
                        } => {
                            if let Some(applications) = applications.get::<Application<C>>() {
                                let inner_application = applications.inner.get_mut(index).unwrap();
                                if let Ok(render_node) =
                                    inner_application.roundtrip(Event::Keyboard(Key {
                                        utf8: ch.as_ref(),
                                        value: &[keysym],
                                        modifiers: Modifiers::default(),
                                        pressed: true,
                                    }))
                                {
                                    draw_callback::<C>(
                                        &inner_application.surface.as_ref().unwrap().surface,
                                        render_node,
                                    );
                                }
                            }
                        }
                        _ => {}
                    },
                )
                .unwrap();
            }
            if pointer && seat.capabilities & Capability::Pointer == Capability::Pointer {
                let pointer = seat.seat.get_pointer();
                assign_pointer::<C>(&pointer);
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
        let i = InnerApplication::empty(controller, widget, self.globals.clone(), cb);
        self.inner.push(i);
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
        let i = InnerApplication::new(controller, widget, config, self.globals.clone(), cb);
        self.inner.push(i);
        handle.update(&self.token).unwrap();
    }
    pub fn create_inner_application<Data: 'static>(
        &mut self,
        controller: C,
        widget: impl Widget + 'static,
        handle: LoopHandle<'_, Data>,
        cb: impl FnMut(&mut CoreApplication<C>, Event) + 'static,
    ) {
        let i = InnerApplication::default(controller, widget, self.globals.clone(), cb);
        self.inner.push(i);
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
        self.widget.sync(&mut sync_ctx, ev);
        while let Ok(signal) = sync_ctx.sync() {
            self.widget.sync(&mut sync_ctx, Event::Message(signal));
        }
        sync_ctx.damage() && !self.ctx.pending_cb
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
        InnerApplication {
            core: CoreApplication {
                controller,
                ctx: Context {
                    pending_cb: false,
                    font_cache: FontCache::new(),
                    render_node: None,
                },
                surface: globals.as_ref().create_shell_surface_from::<C>(
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
        controller: C,
        widget: impl Widget + 'static,
        config: ShellConfig,
        globals: Rc<Globals>,
        cb: impl FnMut(&mut CoreApplication<C>, Event) + 'static,
    ) -> Self {
        InnerApplication {
            core: CoreApplication {
                controller,
                ctx: Context {
                    pending_cb: false,
                    font_cache: FontCache::new(),
                    render_node: None,
                },
                surface: globals
                    .as_ref()
                    .create_shell_surface_from::<C>(&widget, config, None),
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
    pub fn roundtrip(&mut self, ev: Event) -> Result<RenderNode, ()> {
        let width = self.width();
        let height = self.height();

        // Sending the event to the widget tree
        if self.sync(ev) || ev == Event::Commit {
            // Calling the application´s closure
            (self.cb)(&mut self.core, ev);

            // Creating the render node
            let recent_node = self.core.widget.create_node(0., 0.);

            // Resizing the surface in case the widget changed size
            if width != self.width() || height != self.height() {
                let _ = self.set_size(recent_node.width(), recent_node.height());
            }

            self.ctx.pending_cb = true;

            return Ok(recent_node);
        } else {
            // Calling the application´s closure
            (self.cb)(&mut self.core, ev);
        }

        Err(())
    }
    fn render(&mut self, recent_node: RenderNode) {
        let width = recent_node.width();
        let height = recent_node.height();
        if let Ok((buffer, wl_buffer)) =
            Buffer::new(&mut self.core.mempool, width as i32, height as i32)
        {
            let mut v = Vec::new();
            if let Some(render_node) = self.core.ctx.render_node.as_mut() {
                let _ = render_node.merge(
                    recent_node,
                    &mut DrawContext::new(buffer.backend, &mut self.core.ctx.font_cache, &mut v),
                    &Instruction::empty(0., 0., width, height),
                );
            } else {
                recent_node.render(&mut DrawContext::new(
                    buffer.backend,
                    &mut self.core.ctx.font_cache,
                    &mut v,
                ));
                self.core.ctx.render_node = Some(recent_node);
            }
            self.core.ctx.pending_cb = false;
            if let Some(surface) = self.surface.as_mut() {
                surface.attach_buffer(wl_buffer);
                surface.damage(&v);
                surface.commit();
            }
        }
    }
}

fn draw_callback<C: Controller + Clone + 'static>(
    surface: &Main<WlSurface>,
    mut recent_node: RenderNode,
) {
    let h = surface.detach();
    surface
        .frame()
        .quick_assign(move |_, event, mut application| match event {
            wl_callback::Event::Done { callback_data: _ } => {
                if let Some(application) = application.get::<Application<C>>() {
                    let inner_application = application.get_application(&h).unwrap();
                    inner_application.render(std::mem::take(&mut recent_node));
                }
            }
            _ => {}
        });
    surface.commit();
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

fn assign_pointer<C: Controller + Clone + 'static>(pointer: &Main<WlPointer>) {
    let mut index = 0;
    let mut input = Pointer::Enter;
    let (mut x, mut y) = (0., 0.);
    pointer.quick_assign(move |_, event, mut inner| match event {
        wl_pointer::Event::Leave { serial: _, surface } => {
            input = Pointer::Leave;
            if let Some(application) = inner.get::<Application<C>>() {
                if let Some(inner) = application.get_application(&surface) {
                    if let Ok(render_node) =
                        inner.roundtrip(Event::Pointer(x as f32, y as f32, input))
                    {
                        draw_callback::<C>(&inner.surface.as_ref().unwrap().surface, render_node);
                    }
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
                if let Ok(render_node) =
                    inner_application.roundtrip(Event::Pointer(x as f32, y as f32, input))
                {
                    draw_callback::<C>(
                        &inner_application.surface.as_ref().unwrap().surface,
                        render_node,
                    );
                }
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
                for a in &mut application.inner {
                    if let Some(app_surface) = a.surface.as_mut() {
                        match &app_surface.shell {
                            Shell::LayerShell { config: _, surface } => {
                                if shell.eq(surface) {
                                    app_surface.destroy_previous();
<<<<<<< HEAD
                                    if a.ctx.render_node.is_none() {
                                        if let Ok(render_node) = a.roundtrip(Event::Commit) {
                                            a.render(render_node);
                                        }
                                    } else {
=======
                                    if a.ctx.render_node.is_some() {
>>>>>>> rectangle
                                        if let Ok(render_node) = a.roundtrip(Event::Commit) {
                                            draw_callback::<C>(
                                                &a.surface.as_ref().unwrap().surface,
                                                render_node,
                                            );
                                        }
<<<<<<< HEAD
=======
                                    } else {
                                        if let Ok(render_node) = a.roundtrip(Event::Commit) {
                                            a.render(render_node);
                                        }
>>>>>>> rectangle
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
