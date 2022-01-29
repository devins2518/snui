use crate::cache::*;
use crate::context::DrawContext;
use crate::data::Data;
use crate::scene::*;
use crate::wayland::{buffer, GlobalManager, LayerShellConfig, Output, Seat, Shell, Surface};
use crate::widgets::window::WindowRequest;
use crate::*;
use tiny_skia::Transform;
// use smithay_client_toolkit::reexports::calloop::{EventLoop, LoopHandle, RegistrationToken};
// use smithay_client_toolkit::seat::keyboard::ModifiersState;
// use smithay_client_toolkit::shm::AutoMemPool;
// use smithay_client_toolkit::WaylandSource;

use std::cell::RefCell;
use std::mem::take;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use smithay_client_toolkit::reexports::client::{
    protocol::wl_buffer,
    protocol::wl_callback,
    protocol::wl_compositor,
    protocol::wl_output,
    protocol::wl_pointer::{self, Axis, ButtonState},
    protocol::wl_region,
    protocol::wl_registry,
    protocol::wl_seat::{self, Capability},
    protocol::wl_shm,
    protocol::wl_shm_pool,
    protocol::wl_subcompositor,
    protocol::wl_subsurface,
    protocol::wl_surface,
    Connection, ConnectionHandle, DelegateDispatch, Dispatch, EventQueue, QueueHandle, WEnum,
};
use smithay_client_toolkit::reexports::protocols::{
    wlr::unstable::layer_shell::v1::client::{zwlr_layer_shell_v1, zwlr_layer_surface_v1},
    xdg_shell::client::{xdg_surface, xdg_toplevel, xdg_wm_base},
};
use smithay_client_toolkit::shm::pool::multi::MultiPool;
use smithay_client_toolkit::shm::pool::raw::RawPool;
use smithay_client_toolkit::shm::pool::{AsPool, PoolHandle};

pub struct WaylandClient<D>
where
    D: Data + Clone + 'static,
{
    cache: Cache,
    current: Option<usize>,
    connection: Connection,
    event_buffer: Event<'static>,
    globals: Rc<RefCell<GlobalManager>>,
    applications: Vec<Application<D>>,
    pool: Option<MultiPool<wl_surface::WlSurface>>,
}

impl<D> WaylandClient<D>
where
    D: Data + Clone,
{
    pub fn new() -> Option<(Self, EventQueue<Self>)> {
        let conn = Connection::connect_to_env().ok()?;

        let mut event_queue = conn.new_event_queue();
        let qhandle = event_queue.handle();

        let display = conn.handle().display();
        let registry = display
            .get_registry(&mut conn.handle(), &qhandle, ())
            .ok()?;

        let mut wl_client = Self {
            current: None,
            pool: None,
            cache: Cache::default(),
            globals: Rc::new(RefCell::new(GlobalManager::new(registry))),
            connection: conn,
            event_buffer: Event::default(),
            applications: Vec::new(),
        };

        for _ in 0..2 {
            event_queue.blocking_dispatch(&mut wl_client).unwrap();
        }

        Some((wl_client, event_queue))
    }
    fn flush_ev_buffer(&mut self, conn: &mut ConnectionHandle, qh: &QueueHandle<Self>) {
        if let Some(i) = self.current {
            // Forward the event to the Application
            self.applications[i].update_scene(
                self.pool.as_mut().unwrap(),
                &mut self.cache,
                self.event_buffer,
                conn,
                qh,
            );
            if !self.applications[i].state.configured {
                self.current = None;
                self.event_buffer = Event::default();
                let application = self.applications.remove(i);
                self.pool
                    .as_mut()
                    .unwrap()
                    .remove(&application.surface.wl_surface);
            }
        }
    }
    pub fn fwd_event(&mut self, event: Event, qh: &QueueHandle<Self>) {
        if let Some(i) = self.current {
            self.applications[i].update_scene(
                self.pool.as_mut().unwrap(),
                &mut self.cache,
                event,
                &mut self.connection.handle(),
                qh,
            );
            if !self.applications[i].state.configured {
                self.current = None;
                let application = self.applications.remove(i);
                self.pool
                    .as_mut()
                    .unwrap()
                    .remove(&application.surface.wl_surface);
            }
        }
    }
    pub fn new_window(
        &mut self,
        data: D,
        widget: impl Widget<D> + 'static,
        qh: &QueueHandle<Self>,
    ) {
        let mut conn = self.connection.handle();
        let surface = self.globals.borrow().create_xdg_surface(&mut conn, qh);
        let mut application = Application {
            state: State::default(),
            data,
            globals: self.globals.clone(),
            widget: Box::new(widget),
            clipmask: ClipMask::new(),
            surface: surface.unwrap(),
        };

        application.sync(&mut conn, &mut self.cache, Event::Prepare);

        self.applications.push(application);
    }
    pub fn new_widget(
        &mut self,
        data: D,
        widget: impl Widget<D> + 'static,
        config: LayerShellConfig,
        qh: &QueueHandle<Self>,
    ) {
        let mut conn = self.connection.handle();
        let surface = self
            .globals
            .borrow()
            .create_layer_surface(&mut conn, config, qh);
        let mut application = Application {
            state: State::default(),
            data,
            globals: self.globals.clone(),
            clipmask: ClipMask::new(),
            widget: Box::new(widget),
            surface: surface.unwrap(),
        };

        application.sync(&mut conn, &mut self.cache, Event::Prepare);

        self.applications.push(application);
    }
    pub fn has_application(&self) -> bool {
        !self.applications.is_empty()
    }
    pub fn cache(&mut self) -> &mut Cache {
        &mut self.cache
    }
}

impl<D> Drop for WaylandClient<D>
where
    D: Data + Clone,
{
    fn drop(&mut self) {
        self.globals.borrow().destroy(&mut self.connection.handle())
    }
}

pub struct Application<D>
where
    D: Data,
{
    state: State,
    data: D,
    surface: Surface,
    widget: Box<dyn Widget<D>>,
    clipmask: ClipMask,
    globals: Rc<RefCell<GlobalManager>>,
}

struct State {
    time: u32,
    offset: usize,
    configured: bool,
    pending_cb: bool,
    window_state: Vec<WindowState>,
    render_node: RenderNode,
}

impl Default for State {
    fn default() -> Self {
        Self {
            time: 0,
            offset: 0,
            configured: false,
            pending_cb: false,
            window_state: Vec::new(),
            render_node: RenderNode::None,
        }
    }
}

impl<D> Application<D>
where
    D: Data,
{
    fn eq_surface(&self, surface: &wl_surface::WlSurface) -> bool {
        self.surface.wl_surface.eq(surface)
    }
    fn set_size(&mut self, conn: &mut ConnectionHandle, width: f32, height: f32) {
        let (width, height) = Geometry::set_size(self, width, height)
            .err()
            .unwrap_or((width, height));

        self.surface.set_size(conn, width as u32, height as u32)
    }
    pub unsafe fn globals(&self) -> Rc<GlobalManager> {
        let ptr = self.globals.as_ptr();
        Rc::increment_strong_count(ptr);
        Rc::from_raw(ptr)
    }
}

impl<D> Application<D>
where
    D: Data + std::clone::Clone,
{
    fn sync(&mut self, conn: &mut ConnectionHandle, cache: &mut Cache, event: Event) -> Damage {
        let mut damage = if event.is_configure() {
            Damage::Partial
        } else {
            Damage::None
        };
        let mut ctx = SyncContext::new(&mut self.data, cache);
        damage = damage.max(self.widget.sync(&mut ctx, event));

        while ctx.sync() {
            damage = damage.max(self.widget.sync(&mut ctx, Event::Sync));
        }

        if let Some(request) = take(&mut ctx.window_request) {
            match request {
                WindowRequest::Close => {
                    // The WaylandClient will check if it's configured
                    // and remove the Application if it's not.
                    self.state.configured = false;
                    self.surface.destroy(conn);
                    return Damage::None;
                }
                WindowRequest::Move(serial) => match &self.surface.shell {
                    Shell::Xdg { toplevel, .. } => {
                        for seat in &self.globals.borrow().seats {
                            toplevel._move(conn, &seat.seat, serial);
                        }
                    }
                    _ => {}
                },
                WindowRequest::Menu(x, y, serial) => match &self.surface.shell {
                    Shell::Xdg { toplevel, .. } => {
                        for seat in &self.globals.borrow().seats {
                            toplevel.show_window_menu(conn, &seat.seat, serial, x as i32, y as i32);
                        }
                    }
                    _ => {}
                },
                WindowRequest::Minimize => match &self.surface.shell {
                    Shell::Xdg { toplevel, .. } => {
                        toplevel.set_minimized(conn);
                    }
                    _ => {}
                },
                WindowRequest::Maximize => {
                    match &self.surface.shell {
                        Shell::Xdg { toplevel, .. } => {
                            if let Some(_) = self
                                .state
                                .window_state
                                .iter()
                                .find(|s| WindowState::Maximized.eq(s))
                            {
                                toplevel.unset_maximized(conn);
                            } else {
                                toplevel.set_maximized(conn);
                            }
                        }
                        Shell::LayerShell { layer_surface, .. } => {
                            // The following Configure event should be adjusted to
                            // the size of the output
                            layer_surface.set_size(conn, 1 << 31, 1 << 31);
                        }
                    }
                }
                WindowRequest::Title(title) => match &self.surface.shell {
                    Shell::Xdg { toplevel, .. } => {
                        toplevel.set_title(conn, title);
                    }
                    _ => {}
                },
            }
        }

        damage
    }
    fn update_scene(
        &mut self,
        pool: &mut MultiPool<wl_surface::WlSurface>,
        cache: &mut Cache,
        event: Event,
        conn: &mut ConnectionHandle,
        qh: &QueueHandle<WaylandClient<D>>,
    ) {
        let scale = if let Some(output) = &self.surface.output {
            output.scale
        } else {
            1
        };
        let width = self.state.render_node.width() / scale as f32;
        let height = self.state.render_node.height() / scale as f32;
        match self.sync(conn, cache, event) {
            Damage::Partial => {
                if !self.state.pending_cb {
                    if width != self.width() * scale as f32 || height != self.height() {
                        self.sync(conn, cache, Event::Prepare);
                    }
                    let render_node = self
                        .widget
                        .create_node(Transform::from_scale(scale as f32, scale as f32));
                    self.render(pool, cache, render_node, scale, Damage::Partial, conn, qh);
                }
            }
            Damage::Frame => {
                if !self.state.pending_cb {
                    if self.surface.frame(conn, qh, ()).is_ok() {
                        if width != self.width() || height != self.height() {
                            self.sync(conn, cache, Event::Prepare);
                        }
                        let render_node = self
                            .widget
                            .create_node(Transform::from_scale(scale as f32, scale as f32));
                        self.state.pending_cb = true;
                        self.render(pool, cache, render_node, scale, Damage::Frame, conn, qh);
                    }
                }
            }
            _ => {}
        }
    }
    fn render(
        &mut self,
        pool: &mut MultiPool<wl_surface::WlSurface>,
        cache: &mut Cache,
        render_node: RenderNode,
        scale: i32,
        damage: Damage,
        conn: &mut ConnectionHandle,
        qh: &QueueHandle<WaylandClient<D>>,
    ) {
        let surface = &mut self.surface;
        let width = self.widget.width() * scale as f32;
        let height = self.widget.height() * scale as f32;
        if let Some((offset, wl_buffer, backend)) = buffer(
            pool,
            width as u32,
            height as u32,
            &surface.wl_surface,
            (),
            conn,
            qh,
        ) {
            surface.replace_buffer(wl_buffer);
            let mut v = Vec::new();
            let region = Region::new(0., 0., width, height);
            let mut ctx = DrawContext::new(backend, cache, &mut v);

            if offset != self.state.offset {
                self.state.offset = offset;
                self.state.render_node.merge(render_node);
                ctx.damage_region(&Texture::Transparent, region, false);
                self.state
                    .render_node
                    .render(&mut ctx, &mut ClipRegion::new(region, None));
            } else {
                if let Err(region) = self.state.render_node.draw_merge(
                    render_node,
                    &mut ctx,
                    &Instruction::empty(0., 0., width, height),
                    &mut ClipRegion::new(region, Some(&mut self.clipmask)),
                ) {
                    ctx.damage_region(&Texture::Transparent, region, false);
                    self.state
                        .render_node
                        .render(&mut ctx, &mut ClipRegion::new(region, None));
                }
            }

            surface.damage(conn, &v, scale);
            surface.commit(conn);
        } else if !self.state.pending_cb && surface.frame(conn, qh, ()).is_ok() {
            surface.wl_surface.commit(conn);
            self.state.pending_cb = true;
        // If this is a case it means the callback failed so pending_cb callback should be reset
        } else if let Damage::Frame = damage {
            self.state.pending_cb = false;
        }
    }
}

impl<D> Geometry for Application<D>
where
    D: Data,
{
    fn height(&self) -> f32 {
        self.widget.height()
    }
    fn width(&self) -> f32 {
        self.widget.width()
    }
    fn set_width(&mut self, width: f32) -> Result<(), f32> {
        if width > 0. {
            self.widget.set_width(width)
        } else {
            Err(self.width())
        }
    }
    fn set_height(&mut self, height: f32) -> Result<(), f32> {
        if height > 0. {
            return self.widget.set_height(height);
        } else {
            Err(self.width())
        }
    }
}

impl GlobalManager {
    fn create_xdg_surface<D>(
        &self,
        conn: &mut ConnectionHandle,
        qh: &QueueHandle<WaylandClient<D>>,
    ) -> Option<Surface>
    where
        D: Data + Clone,
    {
        let wm_base = self.wm_base.as_ref()?;
        let compositor = self.compositor.as_ref()?;

        let wl_surface = compositor.create_surface(conn, qh, ()).ok()?;
        let wl_region = compositor.create_region(conn, qh, ()).ok()?;
        let xdg_surface = wm_base.get_xdg_surface(conn, &wl_surface, qh, ()).ok()?;
        let toplevel = xdg_surface.get_toplevel(conn, qh, ()).ok()?;

        toplevel.set_app_id(conn, "snui".to_string());
        toplevel.set_title(conn, "test window".to_string());

        wl_surface.commit(conn);

        Some(Surface::new(
            wl_surface,
            wl_region,
            Shell::Xdg {
                xdg_surface,
                toplevel,
            },
            None,
        ))
    }
    fn create_layer_surface<D>(
        &self,
        conn: &mut ConnectionHandle,
        config: LayerShellConfig,
        qh: &QueueHandle<WaylandClient<D>>,
    ) -> Option<Surface>
    where
        D: Data + Clone,
    {
        let layer_shell = self.layer_shell.as_ref()?;
        let compositor = self.compositor.as_ref()?;

        let wl_surface = compositor.create_surface(conn, qh, ()).ok()?;
        let wl_region = compositor.create_region(conn, qh, ()).ok()?;
        let layer_surface = layer_shell
            .get_layer_surface(
                conn,
                &wl_surface,
                None,
                config.layer,
                config.namespace.clone(),
                qh,
                (),
            )
            .ok()?;
        if let Some(anchor) = config.anchor {
            layer_surface.set_anchor(conn, anchor);
        }
        wl_surface.commit(conn);
        layer_surface.set_margin(
            conn,
            config.margin[0],
            config.margin[1],
            config.margin[2],
            config.margin[3],
        );
        layer_surface.set_keyboard_interactivity(conn, config.interactivity);
        Some(Surface::new(
            wl_surface,
            wl_region,
            Shell::LayerShell {
                config,
                layer_surface,
            },
            None,
        ))
    }
}

impl Surface {
    fn new(
        wl_surface: wl_surface::WlSurface,
        wl_region: wl_region::WlRegion,
        shell: Shell,
        previous: Option<Surface>,
    ) -> Self {
        Surface {
            wl_surface,
            shell,
            wl_region,
            output: None,
            previous: if let Some(surface) = previous {
                Some(Box::new(surface))
            } else {
                None
            },
            wl_buffer: None,
        }
    }
    fn commit(&mut self, ch: &mut ConnectionHandle) {
        self.wl_surface.attach(ch, self.wl_buffer.as_ref(), 0, 0);
        self.wl_surface.commit(ch);
        std::mem::drop(&mut self.previous);
        self.previous = None;
        self.wl_buffer = None;
    }
    fn destroy(&mut self, ch: &mut ConnectionHandle) {
        self.shell.destroy(ch);
        self.wl_surface.destroy(ch);
        self.wl_region.destroy(ch);
        self.wl_buffer = None;
        self.destroy_previous(ch);
    }
    fn destroy_previous(&mut self, ch: &mut ConnectionHandle) {
        if let Some(mut surface) = take(&mut self.previous) {
            surface.destroy(ch);
        }
    }
    fn set_size(&self, ch: &mut ConnectionHandle, width: u32, height: u32) {
        self.shell.set_size(ch, width, height);
    }
    fn damage(&self, ch: &mut ConnectionHandle, report: &[Region], scale: i32) {
        for d in report {
            self.wl_surface.damage(
                ch,
                d.x as i32 / scale,
                d.y as i32 / scale,
                d.width as i32 / scale,
                d.height as i32 / scale,
            );
        }
    }
    fn replace_buffer(&mut self, wl_buffer: wl_buffer::WlBuffer) -> Option<()> {
        self.wl_buffer = Some(wl_buffer);
        return Some(());
    }
}

impl Deref for Surface {
    type Target = wl_surface::WlSurface;
    fn deref(&self) -> &Self::Target {
        &self.wl_surface
    }
}

impl<D> Dispatch<wl_registry::WlRegistry> for WaylandClient<D>
where
    D: Data + Clone + 'static,
{
    type UserData = ();

    fn event(
        &mut self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &Self::UserData,
        conn: &mut ConnectionHandle,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name, interface, ..
        } = event
        {
            match &interface[..] {
                "wl_compositor" => {
                    self.globals.borrow_mut().compositor = registry
                        .bind::<wl_compositor::WlCompositor, _>(conn, name, 4, qh, ())
                        .ok();
                }
                "wl_subcompositor" => {
                    self.globals.borrow_mut().subcompositor = registry
                        .bind::<wl_subcompositor::WlSubcompositor, _>(conn, name, 1, qh, ())
                        .ok();
                }
                "wl_shm" => {
                    self.globals.borrow_mut().shm = registry
                        .bind::<wl_shm::WlShm, _>(conn, name, 1, qh, ())
                        .ok();
                    if let Some(ref shm) = self.globals.borrow().shm {
                        self.pool = Some(RawPool::new(1 << 10, shm, conn, qh, ()).unwrap().into());
                    }
                }
                "wl_seat" => {
                    registry
                        .bind::<wl_seat::WlSeat, _>(conn, name, 5, qh, ())
                        .unwrap();
                }
                "wl_output" => {
                    registry
                        .bind::<wl_output::WlOutput, _>(conn, name, 2, qh, ())
                        .unwrap();
                }
                "zwlr_layer_shell_v1" => {
                    self.globals.borrow_mut().layer_shell = registry
                        .bind::<zwlr_layer_shell_v1::ZwlrLayerShellV1, _>(conn, name, 4, qh, ())
                        .ok();
                }
                "xdg_wm_base" => {
                    self.globals.borrow_mut().wm_base = registry
                        .bind::<xdg_wm_base::XdgWmBase, _>(conn, name, 1, qh, ())
                        .ok();
                }
                _ => {}
            }
        }
    }
}

impl<D> AsPool<MultiPool<wl_surface::WlSurface>> for WaylandClient<D>
where
    D: Data + Clone + 'static,
{
    fn pool_handle(&self) -> PoolHandle<MultiPool<wl_surface::WlSurface>> {
        PoolHandle::Ref(self.pool.as_ref().unwrap())
    }
}

impl<D> Dispatch<wl_buffer::WlBuffer> for WaylandClient<D>
where
    D: Data + Clone + 'static,
{
    type UserData = ();

    fn event(
        &mut self,
        proxy: &wl_buffer::WlBuffer,
        event: wl_buffer::Event,
        data: &Self::UserData,
        conn: &mut ConnectionHandle,
        qh: &QueueHandle<Self>,
    ) {
        <MultiPool<wl_surface::WlSurface> as DelegateDispatch<wl_buffer::WlBuffer, Self>>::event(
            self, proxy, event, data, conn, qh,
        );
    }
}

impl<D> Dispatch<wl_compositor::WlCompositor> for WaylandClient<D>
where
    D: Data + Clone + 'static,
{
    type UserData = ();

    fn event(
        &mut self,
        _: &wl_compositor::WlCompositor,
        _: wl_compositor::Event,
        _: &Self::UserData,
        _: &mut ConnectionHandle,
        _: &QueueHandle<Self>,
    ) {
        // wl_compositor has no event
    }
}

impl<D> Dispatch<wl_shm_pool::WlShmPool> for WaylandClient<D>
where
    D: Data + Clone + 'static,
{
    type UserData = ();

    fn event(
        &mut self,
        _: &wl_shm_pool::WlShmPool,
        _: wl_shm_pool::Event,
        _: &Self::UserData,
        _: &mut ConnectionHandle,
        _: &QueueHandle<Self>,
    ) {
        // wl_shm_pool has no event
    }
}

impl<D> Dispatch<wl_subcompositor::WlSubcompositor> for WaylandClient<D>
where
    D: Data + Clone + 'static,
{
    type UserData = ();

    fn event(
        &mut self,
        _: &wl_subcompositor::WlSubcompositor,
        _: wl_subcompositor::Event,
        _: &Self::UserData,
        _: &mut ConnectionHandle,
        _: &QueueHandle<Self>,
    ) {
        // wl_compositor has no event
    }
}

impl<D> Dispatch<wl_surface::WlSurface> for WaylandClient<D>
where
    D: Data + Clone + 'static,
{
    type UserData = ();

    fn event(
        &mut self,
        surface: &wl_surface::WlSurface,
        event: wl_surface::Event,
        _: &Self::UserData,
        conn: &mut ConnectionHandle,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_surface::Event::Enter { output } = event {
            if let Some(output) = self
                .globals
                .borrow()
                .outputs
                .iter()
                .find(|o| o.output == output)
            {
                // Scaling is currently unsupported
                surface.set_buffer_scale(conn, output.scale);
                if let Some((i, application)) = self
                    .applications
                    .iter_mut()
                    .enumerate()
                    .find(|a| a.1.eq_surface(&surface))
                {
                    self.current = Some(i);
                    if let Some(c_output) = application.surface.output.as_ref() {
                        if c_output.scale != output.scale {
                            application.state.render_node = RenderNode::None;
                            application.update_scene(
                                self.pool.as_mut().unwrap(),
                                &mut self.cache,
                                Event::Prepare,
                                conn,
                                qh,
                            );
                        }
                    }
                    application.surface.output = Some(output.clone());
                }
            }
        }
    }
}

impl<D> Dispatch<wl_callback::WlCallback> for WaylandClient<D>
where
    D: Data + Clone + 'static,
{
    type UserData = ();

    fn event(
        &mut self,
        _: &wl_callback::WlCallback,
        event: wl_callback::Event,
        _: &Self::UserData,
        conn: &mut ConnectionHandle,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_callback::Event::Done { callback_data } = event {
            let mut cb = None;
            let cache = &mut self.cache;
            let pool = self.pool.as_mut().unwrap();
            for (i, application) in self.applications.iter_mut().enumerate() {
                if application.state.pending_cb {
                    application.state.pending_cb = false;
                    // The application is rendered prior and the changes are commited here
                    let frame_time = (callback_data - application.state.time).min(20);
                    application.state.time = callback_data;
                    let scale = if let Some(output) = &application.surface.output {
                        output.scale
                    } else {
                        1
                    };
                    // Send a callback event with the timeout the application
                    let width = application.state.render_node.width() / scale as f32;
                    let height = application.state.render_node.height() / scale as f32;
                    match application.sync(conn, cache, Event::Callback(frame_time)) {
                        Damage::Partial => {
                            if width != application.width() || height != application.height() {
                                application.sync(conn, cache, Event::Prepare);
                            }
                            let render_node = application
                                .widget
                                .create_node(Transform::from_scale(scale as f32, scale as f32));
                            application.render(
                                pool,
                                cache,
                                render_node,
                                scale,
                                Damage::Partial,
                                conn,
                                qh,
                            );
                        }
                        Damage::Frame => {
                            cb = Some(i);
                            if width != application.width() || height != application.height() {
                                application.sync(conn, cache, Event::Prepare);
                            }
                            let render_node = application
                                .widget
                                .create_node(Transform::from_scale(scale as f32, scale as f32));
                            application.state.pending_cb = true;
                            application.render(
                                pool,
                                cache,
                                render_node,
                                scale,
                                Damage::Partial,
                                conn,
                                qh,
                            );
                        }
                        Damage::None => {}
                    }
                }
            }
            if let Some(i) = cb {
                if self.applications[i].surface.frame(conn, qh, ()).is_ok() {
                    self.applications[i].surface.deref().commit(conn);
                }
            }
        }
    }
}

impl<D> Dispatch<wl_subsurface::WlSubsurface> for WaylandClient<D>
where
    D: Data + Clone + 'static,
{
    type UserData = ();

    fn event(
        &mut self,
        _: &wl_subsurface::WlSubsurface,
        _: wl_subsurface::Event,
        _: &Self::UserData,
        _: &mut ConnectionHandle,
        _: &QueueHandle<Self>,
    ) {
        // wl_subsurface has no event
    }
}

impl<D> Dispatch<wl_region::WlRegion> for WaylandClient<D>
where
    D: Data + Clone + 'static,
{
    type UserData = ();

    fn event(
        &mut self,
        _: &wl_region::WlRegion,
        _: wl_region::Event,
        _: &Self::UserData,
        _: &mut ConnectionHandle,
        _: &QueueHandle<Self>,
    ) {
        // wl_region has no event
    }
}

impl From<xdg_toplevel::State> for WindowState {
    fn from(state: xdg_toplevel::State) -> Self {
        match state {
            xdg_toplevel::State::Activated => WindowState::Activated,
            xdg_toplevel::State::Fullscreen => WindowState::Fullscreen,
            xdg_toplevel::State::TiledLeft => WindowState::TiledLeft,
            xdg_toplevel::State::TiledRight => WindowState::TiledRight,
            xdg_toplevel::State::TiledTop => WindowState::TiledTop,
            xdg_toplevel::State::TiledBottom => WindowState::TiledBottom,
            xdg_toplevel::State::Resizing => WindowState::Resizing,
            xdg_toplevel::State::Maximized => WindowState::Maximized,
            _ => unreachable!(),
        }
    }
}

impl<D> Dispatch<xdg_toplevel::XdgToplevel> for WaylandClient<D>
where
    D: Data + Clone + 'static,
{
    type UserData = ();

    fn event(
        &mut self,
        toplevel: &xdg_toplevel::XdgToplevel,
        event: xdg_toplevel::Event,
        _: &Self::UserData,
        conn: &mut ConnectionHandle,
        _: &QueueHandle<Self>,
    ) {
        if let Some((i, application)) = self.applications.iter_mut().enumerate().find(|(_, a)| {
            let t_toplevel = toplevel;
            match &a.surface.shell {
                wayland::Shell::Xdg { toplevel, .. } => toplevel.eq(t_toplevel),
                _ => false,
            }
        }) {
            match event {
                xdg_toplevel::Event::Configure {
                    width,
                    height,
                    states,
                } => {
                    application.state.window_state = list_states(states);
                    let r_width = application.widget.set_width(width as f32);
                    let r_height = application.widget.set_height(height as f32);
                    if r_width.is_err() && r_height.is_err() {
                        if let Shell::Xdg { toplevel, .. } = &application.surface.shell {
                            let r_width = r_width.unwrap_err();
                            let r_height = r_height.unwrap_err();
                            if width < r_width as i32 && height < r_height as i32 {
                                toplevel.set_min_size(conn, r_width as i32, r_height as i32);
                            } else if width > r_width as i32 && height > r_height as i32 {
                                toplevel.set_max_size(conn, r_width as i32, r_height as i32);
                            }
                        }
                    }
                    let mut ctx = SyncContext::new(&mut application.data, &mut self.cache);
                    // TO-DO
                    // Convert xdg_toplevel.state to WindowState
                    application
                        .widget
                        .sync(&mut ctx, Event::Configure(&application.state.window_state));
                }
                xdg_toplevel::Event::Close => {
                    application.surface.destroy(conn);
                    self.current = None;
                    let application = self.applications.remove(i);
                    self.pool
                        .as_mut()
                        .unwrap()
                        .remove(&application.surface.wl_surface);
                }
                _ => {}
            }
        }
        // wl_region has no event
    }
}

fn list_states(states: Vec<u8>) -> Vec<WindowState> {
    states
        .chunks(4)
        .filter_map(|endian| {
            if endian.len() == 4 {
                if let Ok(state) = xdg_toplevel::State::try_from(u32::from_ne_bytes([
                    endian[0], endian[1], endian[2], endian[3],
                ])) {
                    Some(WindowState::from(state))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect()
}

impl<D> Dispatch<wl_shm::WlShm> for WaylandClient<D>
where
    D: Data + Clone + 'static,
{
    type UserData = ();

    fn event(
        &mut self,
        _: &wl_shm::WlShm,
        _: wl_shm::Event,
        _: &Self::UserData,
        _: &mut ConnectionHandle,
        _: &QueueHandle<Self>,
    ) {
        // wl_shm has no event
    }
}

impl<D> Dispatch<xdg_wm_base::XdgWmBase> for WaylandClient<D>
where
    D: Data + Clone + 'static,
{
    type UserData = ();

    fn event(
        &mut self,
        wm_base: &xdg_wm_base::XdgWmBase,
        event: xdg_wm_base::Event,
        _: &Self::UserData,
        conn: &mut ConnectionHandle,
        _: &QueueHandle<Self>,
    ) {
        if let xdg_wm_base::Event::Ping { serial } = event {
            wm_base.pong(conn, serial);
        }
    }
}

impl<D> Dispatch<xdg_surface::XdgSurface> for WaylandClient<D>
where
    D: Data + Clone + 'static,
{
    type UserData = ();

    fn event(
        &mut self,
        xdg_surface: &xdg_surface::XdgSurface,
        event: xdg_surface::Event,
        _: &Self::UserData,
        conn: &mut ConnectionHandle,
        qh: &QueueHandle<Self>,
    ) {
        if let xdg_surface::Event::Configure { serial, .. } = event {
            xdg_surface.ack_configure(conn, serial);
            if let Some(application) = self.applications.iter_mut().find(|a| {
                let t_xdg_surface = xdg_surface;
                match &a.surface.shell {
                    wayland::Shell::Xdg { xdg_surface, .. } => xdg_surface.eq(t_xdg_surface),
                    _ => false,
                }
            }) {
                application.state.configured = true;
                application.update_scene(
                    self.pool.as_mut().unwrap(),
                    &mut self.cache,
                    Event::Prepare,
                    conn,
                    qh,
                )
            }
        }
    }
}

impl<D> Dispatch<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1> for WaylandClient<D>
where
    D: Data + Clone + 'static,
{
    type UserData = ();

    fn event(
        &mut self,
        layer_surface: &zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
        event: zwlr_layer_surface_v1::Event,
        _: &Self::UserData,
        conn: &mut ConnectionHandle,
        qh: &QueueHandle<Self>,
    ) {
        if let zwlr_layer_surface_v1::Event::Configure {
            serial,
            width,
            height,
        } = event
        {
            layer_surface.ack_configure(conn, serial);
            if let Some(application) = self.applications.iter_mut().find(|a| {
                let t_layer_surface = layer_surface;
                match &a.surface.shell {
                    wayland::Shell::LayerShell { layer_surface, .. } => {
                        layer_surface.eq(t_layer_surface)
                    }
                    _ => false,
                }
            }) {
                if let Shell::LayerShell { config, .. } = &application.surface.shell {
                    if config.exclusive {
                        use zwlr_layer_surface_v1::Anchor;
                        if let Some(anchor) = config.anchor {
                            match anchor {
                                Anchor::Left | Anchor::Right => layer_surface
                                    .set_exclusive_zone(conn, application.width() as i32),
                                Anchor::Top | Anchor::Bottom => layer_surface
                                    .set_exclusive_zone(conn, application.height() as i32),
                                _ => {}
                            }
                        }
                    }
                }
                application.set_size(conn, width as f32, height as f32);
                application.state.configured = true;
                application.update_scene(
                    self.pool.as_mut().unwrap(),
                    &mut self.cache,
                    Event::Configure(&[WindowState::Activated]),
                    conn,
                    qh,
                )
            }
        }
    }
}

impl<D> Dispatch<zwlr_layer_shell_v1::ZwlrLayerShellV1> for WaylandClient<D>
where
    D: Data + Clone + 'static,
{
    type UserData = ();

    fn event(
        &mut self,
        _: &zwlr_layer_shell_v1::ZwlrLayerShellV1,
        _: zwlr_layer_shell_v1::Event,
        _: &Self::UserData,
        _: &mut ConnectionHandle,
        _: &QueueHandle<Self>,
    ) {
        // layer_shell has no event
    }
}

impl<D> Dispatch<wl_seat::WlSeat> for WaylandClient<D>
where
    D: Data + Clone + 'static,
{
    type UserData = ();

    fn event(
        &mut self,
        seat: &wl_seat::WlSeat,
        event: wl_seat::Event,
        data: &Self::UserData,
        conn: &mut ConnectionHandle,
        qh: &QueueHandle<Self>,
    ) {
        if let Some(seat) = self
            .globals
            .borrow_mut()
            .seats
            .iter_mut()
            .find(|s| s.seat.eq(seat))
        {
            match event {
                wl_seat::Event::Name { ref name } => {
                    seat.name = name.clone();
                }
                wl_seat::Event::Capabilities { capabilities } => {
                    seat.capabilities = capabilities;
                    if let WEnum::Value(capabilities) = capabilities {
                        if capabilities & Capability::Pointer == Capability::Pointer {
                            seat.seat.get_pointer(conn, qh, ()).unwrap();
                        }
                    }
                }
                _ => {}
            }
            return;
        }
        self.globals
            .borrow_mut()
            .seats
            .push(Seat::new(seat.clone()));
        self.event(seat, event, data, conn, qh);
    }
}

impl<D> Dispatch<wl_pointer::WlPointer> for WaylandClient<D>
where
    D: Data + Clone + 'static,
{
    type UserData = ();

    fn event(
        &mut self,
        pointer: &wl_pointer::WlPointer,
        event: wl_pointer::Event,
        data: &Self::UserData,
        conn: &mut ConnectionHandle,
        qh: &QueueHandle<Self>,
    ) {
        if self.current.is_some() {
            match event {
                wl_pointer::Event::Button {
                    serial,
                    time: _,
                    button,
                    state,
                } => {
                    if let Event::Pointer(_, _, p) = &mut self.event_buffer {
                        *p = Pointer::MouseClick {
                            serial,
                            button: MouseButton::new(button),
                            pressed: if let WEnum::Value(state) = state {
                                state == ButtonState::Pressed
                            } else {
                                false
                            },
                        };
                    }
                }
                wl_pointer::Event::Axis {
                    time: _,
                    axis,
                    value,
                } => {
                    if let Event::Pointer(_, _, p) = &mut self.event_buffer {
                        if let WEnum::Value(axis) = axis {
                            *p = Pointer::Scroll {
                                orientation: match axis {
                                    Axis::VerticalScroll => Orientation::Vertical,
                                    Axis::HorizontalScroll => Orientation::Horizontal,
                                    _ => unreachable!(),
                                },
                                value: Move::Value(value as f32),
                            };
                        }
                    }
                }
                wl_pointer::Event::AxisDiscrete { axis, discrete } => {
                    if let Event::Pointer(_, _, p) = &mut self.event_buffer {
                        if let WEnum::Value(axis) = axis {
                            *p = Pointer::Scroll {
                                orientation: match axis {
                                    Axis::VerticalScroll => Orientation::Vertical,
                                    Axis::HorizontalScroll => Orientation::Horizontal,
                                    _ => unreachable!(),
                                },
                                value: Move::Step(discrete),
                            };
                        }
                    }
                }
                wl_pointer::Event::Motion {
                    time: _,
                    surface_x,
                    surface_y,
                } => {
                    self.event_buffer =
                        Event::Pointer(surface_x as f32, surface_y as f32, Pointer::Hover);
                }
                wl_pointer::Event::Frame => {
                    // Call dispatch method of the Application
                    self.flush_ev_buffer(conn, qh);
                }
                wl_pointer::Event::Leave { .. } => {
                    if let Event::Pointer(_, _, p) = &mut self.event_buffer {
                        *p = Pointer::Leave;
                    }
                    self.flush_ev_buffer(conn, qh);
                    self.current = None;
                }
                _ => {}
            }
        } else {
            match event {
                wl_pointer::Event::Enter { ref surface, .. } => {
                    self.current = (0..self.applications.len())
                        .find(|i| self.applications[*i].eq_surface(surface));
                    self.event(pointer, event, data, conn, qh);
                }
                wl_pointer::Event::Leave { .. } => {
                    self.current = None;
                }
                _ => {}
            }
        }
    }
}

impl<D> Dispatch<wl_output::WlOutput> for WaylandClient<D>
where
    D: Data + Clone + 'static,
{
    type UserData = ();

    fn event(
        &mut self,
        output: &wl_output::WlOutput,
        event: wl_output::Event,
        data: &Self::UserData,
        conn: &mut ConnectionHandle,
        qh: &QueueHandle<Self>,
    ) {
        if let Some(output) = self
            .globals
            .borrow_mut()
            .outputs
            .iter_mut()
            .find(|o| o.output.eq(output))
        {
            match event {
                wl_output::Event::Geometry {
                    physical_width,
                    physical_height,
                    ..
                } => {
                    output.physical_width = physical_width;
                    output.physical_height = physical_height;
                }
                wl_output::Event::Mode {
                    flags: _,
                    width,
                    height,
                    refresh,
                } => {
                    output.width = width;
                    output.height = height;
                    output.refresh = refresh;
                }
                wl_output::Event::Name { ref name } => {
                    output.name = name.clone();
                }
                wl_output::Event::Scale { factor } => {
                    output.scale = factor;
                }
                _ => {}
            }
            return;
        }
        self.globals
            .borrow_mut()
            .outputs
            .push(Output::new(output.clone()));
        self.event(output, event, data, conn, qh);
    }
}
