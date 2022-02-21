use crate::cache::*;
use crate::context::DrawContext;
use crate::mail::Data;
use crate::scene::*;
use crate::wayland::{buffer, GlobalManager, LayerShellConfig, Output, Seat, Shell, Surface};
use crate::widgets::shapes::Rectangle;
use crate::*;

use smithay_client_toolkit::reexports::client::Proxy;

use tiny_skia::Transform;

use std::cell::RefCell;
use std::mem::take;
use std::ops::Deref;
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
    Connection, ConnectionHandle, DelegateDispatch, DelegateDispatchBase, Dispatch, EventQueue,
    QueueHandle, WEnum,
};
use smithay_client_toolkit::reexports::protocols::{
    wlr::unstable::layer_shell::v1::client::{zwlr_layer_shell_v1, zwlr_layer_surface_v1},
    xdg_shell::client::{xdg_surface, xdg_toplevel, xdg_wm_base},
};
use smithay_client_toolkit::shm::pool::multi::MultiPool;
use smithay_client_toolkit::shm::pool::PoolHandle;
use smithay_client_toolkit::shm::{ShmHandler, ShmState};
use wayland_cursor::CursorTheme;

pub struct WaylandClient<T>
where
    T: Data + Clone + 'static,
{
    cache: Cache,
    current: Option<usize>,
    connection: Connection,
    event: Event<'static>,
    globals: Rc<RefCell<GlobalManager>>,
    views: Vec<View<T>>,
    pool: Option<MultiPool<wl_surface::WlSurface>>,
}

impl<T> AsMut<Cache> for WaylandClient<T>
where
    T: Data + Clone,
{
    fn as_mut(&mut self) -> &mut Cache {
        &mut self.cache
    }
}

impl<T> WaylandClient<T>
where
    T: Data + Clone,
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
            event: Event::default(),
            views: Vec::new(),
        };

        for _ in 0..2 {
            event_queue.blocking_dispatch(&mut wl_client).unwrap();
        }

        Some((wl_client, event_queue))
    }
    fn flush_event(&mut self, conn: &mut ConnectionHandle, qh: &QueueHandle<Self>) {
        if let Some(i) = self.current {
            // Forward the event to the View
            self.views[i].update_scene(
                self.pool.as_mut().unwrap(),
                &mut self.cache,
                self.event,
                conn,
                qh,
            );
            if !self.views[i].state.configured {
                self.current = None;
                self.event = Event::default();
                let view = self.views.remove(i);
                self.pool
                    .as_mut()
                    .unwrap()
                    .remove(&view.surface.wl_surface, conn);
            }
        }
    }
    pub fn send_event(&mut self, event: Event, qh: &QueueHandle<Self>) {
        if let Some(i) = self.current {
            self.views[i].update_scene(
                self.pool.as_mut().unwrap(),
                &mut self.cache,
                event,
                &mut self.connection.handle(),
                qh,
            );
            if !self.views[i].state.configured {
                self.current = None;
                let view = self.views.remove(i);
                self.pool
                    .as_mut()
                    .unwrap()
                    .remove(&view.surface.wl_surface, &mut self.connection.handle());
            }
        }
    }
    pub fn new_window(
        &mut self,
        data: T,
        widget: impl Widget<T> + 'static,
        qh: &QueueHandle<Self>,
    ) {
        let mut conn = self.connection.handle();
        let surface = self.globals.borrow().create_xdg_surface(&mut conn, qh);
        let view = View {
            state: State::default(),
            data,
            globals: self.globals.clone(),
            widget: Box::new(widget),
            clipmask: Some(ClipMask::new()),
            surface: surface.expect("Failed to create an XdgSurface"),
        };

        self.views.push(view);
    }
    pub fn new_widget(
        &mut self,
        data: T,
        widget: impl Widget<T> + 'static,
        config: LayerShellConfig,
        qh: &QueueHandle<Self>,
    ) {
        let mut conn = self.connection.handle();
        let surface = self
            .globals
            .borrow()
            .create_layer_surface(&mut conn, config, qh);
        let view = View {
            state: State::default(),
            data,
            globals: self.globals.clone(),
            clipmask: Some(ClipMask::new()),
            widget: Box::new(widget),
            surface: surface.expect("Failed to create a LayerSurface"),
        };

        self.views.push(view);
    }
    pub fn has_view(&self) -> bool {
        !self.views.is_empty()
    }
    pub fn cache(&mut self) -> &mut Cache {
        &mut self.cache
    }
}

impl<T> Drop for WaylandClient<T>
where
    T: Data + Clone,
{
    fn drop(&mut self) {
        self.globals.borrow().destroy(&mut self.connection.handle())
    }
}

pub struct View<T>
where
    T: Data,
{
    data: T,
    state: State,
    surface: Surface,
    clipmask: Option<ClipMask>,
    widget: Box<dyn Widget<T>>,
    globals: Rc<RefCell<GlobalManager>>,
}

struct State {
    time: u32,
    offset: usize,
    configured: bool,
    pending_cb: bool,
    enter_serial: u32,
    constraint: BoxConstraints,
    window_state: Vec<WindowState>,
    render_node: RenderNode,
}

impl Default for State {
    fn default() -> Self {
        Self {
            time: 0,
            offset: 0,
            enter_serial: 0,
            configured: false,
            pending_cb: false,
            constraint: BoxConstraints::default(),
            window_state: Vec::new(),
            render_node: RenderNode::None,
        }
    }
}

impl<T> View<T>
where
    T: Data + Clone,
{
    fn eq_surface(&self, surface: &wl_surface::WlSurface) -> bool {
        self.surface.wl_surface.eq(surface)
    }
    fn handle<'t, 'v, 'c>(
        &'t mut self,
        conn: &'v mut ConnectionHandle<'c>,
        qh: &'v QueueHandle<WaylandClient<T>>,
    ) -> ViewHandle<'v, 'c, T>
    where
        't: 'v,
        'c: 'v,
    {
        ViewHandle {
            conn,
            globals: self.globals.borrow_mut(),
            state: &mut self.state,
            qh,
            surface: &mut self.surface,
        }
    }
    pub unsafe fn globals(&self) -> Rc<GlobalManager> {
        let ptr = self.globals.as_ptr();
        Rc::increment_strong_count(ptr);
        Rc::from_raw(ptr)
    }
}

use crate::context::WindowHandle;
use std::cell::RefMut;

struct ViewHandle<'v, 'c, T: 'static + Data + Clone> {
    qh: &'v QueueHandle<WaylandClient<T>>,
    globals: RefMut<'v, GlobalManager>,
    state: &'v mut State,
    conn: &'v mut ConnectionHandle<'c>,
    surface: &'v mut Surface,
}

impl<'v, 'c, T: Data + Clone> WindowHandle for ViewHandle<'v, 'c, T> {
    fn close(&mut self) {
        // The WaylandClient will check if it's configured
        // and remove the View if it's not.
        self.state.configured = false;
        self.surface.destroy(self.conn);
    }
    fn _move(&mut self, serial: u32) {
        match &self.surface.shell {
            Shell::Xdg { toplevel, .. } => {
                for seat in &self.globals.seats {
                    toplevel._move(self.conn, &seat.seat, serial);
                }
            }
            _ => {}
        }
    }
    fn maximize(&mut self) {
        match &self.surface.shell {
            Shell::Xdg { toplevel, .. } => {
                if let Some(_) = self
                    .state
                    .window_state
                    .iter()
                    .find(|s| WindowState::Maximized.eq(s))
                {
                    toplevel.unset_maximized(self.conn);
                } else {
                    toplevel.set_maximized(self.conn);
                }
            }
            Shell::LayerShell { layer_surface, .. } => {
                // The following Configure event should be adjusted to
                // the size of the output
                layer_surface.set_size(self.conn, 1 << 31, 1 << 31);
            }
        }
    }
    fn menu(&mut self, x: f32, y: f32, serial: u32) {
        match &self.surface.shell {
            Shell::Xdg { toplevel, .. } => {
                for seat in &self.globals.seats {
                    toplevel.show_window_menu(self.conn, &seat.seat, serial, x as i32, y as i32);
                }
            }
            _ => {}
        }
    }
    fn minimize(&mut self) {
        match &self.surface.shell {
            Shell::Xdg { toplevel, .. } => {
                toplevel.set_minimized(self.conn);
            }
            _ => {}
        }
    }
    fn set_title(&mut self, title: String) {
        match self.surface.shell {
            Shell::Xdg { ref toplevel, .. } => {
                toplevel.set_title(self.conn, title);
            }
            Shell::LayerShell { ref mut config, .. } => {
                config.namespace = title;
            }
        }
    }
    fn set_cursor(&mut self, cursor: Cursor) {
        let scale = self
            .surface
            .output
            .as_ref()
            .map(|output| output.scale)
            .unwrap_or(1);
        let globals = &mut *self.globals;
        let surface = if let Some(surface) = globals.pointer_surface.as_ref() {
            surface
        } else {
            globals.pointer_surface = Some(
                globals
                    .create_surface(self.conn, self.qh)
                    .expect("Failed to create cursor surface"),
            );
            globals.pointer_surface.as_ref().unwrap()
        };
        let seats = &globals.seats;
        let cursor_size = 24 * scale as u32;
        let cursor_theme = globals.cursor_theme.get_mut(&cursor_size);
        if let Some(cursor_theme) = cursor_theme {
            for seat in seats {
                if let Some(cursor) = cursor_theme.get_cursor(self.conn, cursor.as_str()) {
                    let buffer = &cursor[0];
                    let (hotspot_x, hotspot_y) = buffer.hotspot();
                    surface.set_buffer_scale(self.conn, scale);
                    surface.attach(self.conn, Some(&buffer), 0, 0);
                    surface.commit(self.conn);
                    seat.pointer
                        .as_ref()
                        .expect("Failed to retreive the pointer")
                        .set_cursor(
                            self.conn,
                            self.state.enter_serial,
                            Some(&surface),
                            hotspot_x as i32,
                            hotspot_y as i32,
                        );
                }
            }
        } else {
            if let Ok(cursor_theme) =
                CursorTheme::load(self.conn, globals.shm.clone().unwrap(), 24 * scale as u32)
            {
                globals.cursor_theme.insert(cursor_size, cursor_theme);
                std::mem::drop(globals);
                self.set_cursor(cursor);
            } else {
                surface.destroy(self.conn);
            }
        }
    }
    fn get_state(&self) -> &[WindowState] {
        self.state.window_state.as_slice()
    }
}

impl<T> View<T>
where
    T: Data + std::clone::Clone,
{
    fn sync(
        &mut self,
        cache: &mut Cache,
        event: Event,
        conn: &mut ConnectionHandle,
        qh: &QueueHandle<WaylandClient<T>>,
    ) -> Damage {
        let mut damage = match event {
            Event::Draw | Event::Configure => Damage::Partial,
            _ => Damage::None,
        };
        let mut handle = ViewHandle {
            conn,
            globals: self.globals.borrow_mut(),
            state: &mut self.state,
            qh,
            surface: &mut self.surface,
        };
        let mut ctx = SyncContext::new(&mut self.data, cache, &mut handle);
        damage = damage.max(self.widget.sync(&mut ctx, event));

        while ctx.sync() {
            damage = damage.max(self.widget.sync(&mut ctx, Event::Sync));
        }

        damage
    }
    fn update_scene(
        &mut self,
        pool: &mut MultiPool<wl_surface::WlSurface>,
        cache: &mut Cache,
        event: Event,
        conn: &mut ConnectionHandle,
        qh: &QueueHandle<WaylandClient<T>>,
    ) {
        match self.sync(cache, event, conn, qh) {
            Damage::Partial => {
                if !self.state.pending_cb {
                    self.render(pool, cache, Damage::Partial, conn, qh);
                }
            }
            Damage::Frame => {
                if !self.state.pending_cb {
                    if self.surface.frame(conn, qh, ()).is_ok() {
                        self.state.pending_cb = true;
                        self.render(pool, cache, Damage::Frame, conn, qh);
                    }
                }
            }
            _ => {}
        }
    }
    fn quick_render(
        &mut self,
        pool: &mut MultiPool<wl_surface::WlSurface>,
        cache: &mut Cache,
        conn: &mut ConnectionHandle,
        qh: &QueueHandle<WaylandClient<T>>,
    ) {
        let scale = self
            .surface
            .output
            .as_ref()
            .map(|output| output.scale)
            .unwrap_or(1);
        let mut layout = LayoutCtx::new(cache);
        let Size { width, height } = self.widget.layout(&mut layout, &self.state.constraint);
        let surface = &mut self.surface;

        if let Some((offset, wl_buffer, backend)) = buffer(
            pool,
            width as u32 * scale as u32,
            height as u32 * scale as u32,
            &surface.wl_surface,
            (),
            conn,
            qh,
        ) {
            let transform = Transform::from_scale(scale as f32, scale as f32);
            surface.replace_buffer(wl_buffer);
            let region = Rectangle::new(width, height);
            let mut ctx = DrawContext::new(backend, cache)
                .with_transform(transform)
                .with_clipmask(self.clipmask.as_mut());

            self.state.offset = offset;
            ctx.clear(
                &Background::new(&region),
                Region::new(0., 0., width, height),
            );
            self.state.render_node.render(&mut ctx, transform, None);

            surface.damage(conn, ctx.damage_queue());
            surface.commit(conn);
            self.clipmask.as_mut().unwrap().clear();
        }
    }
    fn render(
        &mut self,
        pool: &mut MultiPool<wl_surface::WlSurface>,
        cache: &mut Cache,
        damage: Damage,
        conn: &mut ConnectionHandle,
        qh: &QueueHandle<WaylandClient<T>>,
    ) {
        let scale = self
            .surface
            .output
            .as_ref()
            .map(|output| output.scale)
            .unwrap_or(1);
        let mut layout = LayoutCtx::new(cache);
        let Size { width, height } = self.widget.layout(&mut layout, &self.state.constraint);
        let surface = &mut self.surface;

        self.clipmask.as_mut().unwrap().clear();
        if let Some((offset, wl_buffer, backend)) = buffer(
            pool,
            width as u32 * scale as u32,
            height as u32 * scale as u32,
            &surface.wl_surface,
            (),
            conn,
            qh,
        ) {
            let transform = Transform::from_scale(scale as f32, scale as f32);
            surface.replace_buffer(wl_buffer);
            let region = Rectangle::new(width, height);
            let mut ctx = DrawContext::new(backend, cache)
                .with_transform(transform)
                .with_clipmask(self.clipmask.as_mut());

            if offset != self.state.offset {
                self.state.offset = offset;
                ctx.clear(
                    &Background::new(&region),
                    Region::new(0., 0., width, height),
                );
                self.widget
                    .draw_scene(Scene::new(&mut self.state.render_node, &mut ctx, &region));
                self.state.render_node.render(&mut ctx, transform, None);
            } else {
                self.widget
                    .draw_scene(Scene::new(&mut self.state.render_node, &mut ctx, &region));
            }

            surface.damage(conn, ctx.damage_queue());
            surface.commit(conn);
        } else if !self.state.pending_cb && surface.frame(conn, qh, ()).is_ok() {
            surface.wl_surface.commit(conn);
            self.state.pending_cb = true;
        // If this is the case it means the callback failed so pending_cb callback should be reset
        } else if let Damage::Frame = damage {
            self.state.pending_cb = false;
        }
    }
}

impl GlobalManager {
    fn create_xdg_surface<T>(
        &self,
        conn: &mut ConnectionHandle,
        qh: &QueueHandle<WaylandClient<T>>,
    ) -> Option<Surface>
    where
        T: Data + Clone,
    {
        let wm_base = self.wm_base.as_ref()?;
        let compositor = self.compositor.as_ref()?;

        let wl_surface = compositor.create_surface(conn, qh, ()).ok()?;
        let wl_region = compositor.create_region(conn, qh, ()).ok()?;
        let xdg_surface = wm_base.get_xdg_surface(conn, &wl_surface, qh, ()).ok()?;
        let toplevel = xdg_surface.get_toplevel(conn, qh, ()).ok()?;

        toplevel.set_app_id(conn, "snui".to_string());
        toplevel.set_title(conn, "default".to_string());

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
    fn create_surface<T>(
        &self,
        conn: &mut ConnectionHandle,
        qh: &QueueHandle<WaylandClient<T>>,
    ) -> Option<wl_surface::WlSurface>
    where
        T: Data + Clone,
    {
        let compositor = self.compositor.as_ref()?;
        compositor.create_surface(conn, qh, ()).ok()
    }
    fn create_layer_surface<T>(
        &self,
        conn: &mut ConnectionHandle,
        config: LayerShellConfig,
        qh: &QueueHandle<WaylandClient<T>>,
    ) -> Option<Surface>
    where
        T: Data + Clone,
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
    fn damage(&self, ch: &mut ConnectionHandle, report: &[Region]) {
        for d in report {
            self.wl_surface
                .damage(ch, d.x as i32, d.y as i32, d.width as i32, d.height as i32);
        }
    }
    fn replace_buffer(&mut self, wl_buffer: wl_buffer::WlBuffer) {
        self.wl_buffer = Some(wl_buffer);
    }
}

impl Deref for Surface {
    type Target = wl_surface::WlSurface;
    fn deref(&self) -> &Self::Target {
        &self.wl_surface
    }
}

use smithay_client_toolkit::registry::{ProvidesRegistryState, RegistryHandler};

impl<T> Dispatch<wl_registry::WlRegistry> for WaylandClient<T>
where
    T: Data + Clone + 'static,
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
                    ShmState::new_global(self, conn, qh, name, &interface[..], 1);
                    let mut globals = self.globals.borrow_mut();
                    globals.shm = registry
                        .bind::<wl_shm::WlShm, _>(conn, name, 1, qh, ())
                        .ok();
                    self.pool = globals
                        .shm_state
                        .new_raw_pool(1 << 10, conn, qh, ())
                        .ok()
                        .map(|raw| raw.into());
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
                        .bind::<xdg_wm_base::XdgWmBase, _>(conn, name, 2, qh, ())
                        .ok();
                }
                _ => {}
            }
        }
    }
}

impl<'p, T: Data + Clone> From<&'p mut WaylandClient<T>>
    for PoolHandle<'p, MultiPool<wl_surface::WlSurface>>
{
    fn from(this: &'p mut WaylandClient<T>) -> Self {
        PoolHandle::Ref(this.pool.as_mut().unwrap())
    }
}

impl<T: Data + Clone> ShmHandler for WaylandClient<T> {
    fn shm_state(&mut self) -> &mut ShmState {
        unsafe { &mut (*self.globals.as_ptr()).shm_state }
    }
}

impl<T: Data + Clone> ProvidesRegistryState for WaylandClient<T> {
    fn registry(&mut self) -> &mut smithay_client_toolkit::registry::RegistryState {
        unsafe { &mut (*self.globals.as_ptr()).registry }
    }
}

impl<T> Dispatch<wl_buffer::WlBuffer> for WaylandClient<T>
where
    T: Data + Clone + 'static,
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

impl<T> Dispatch<wl_compositor::WlCompositor> for WaylandClient<T>
where
    T: Data + Clone + 'static,
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

impl<T> Dispatch<wl_shm_pool::WlShmPool> for WaylandClient<T>
where
    T: Data + Clone + 'static,
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

impl<T> Dispatch<wl_subcompositor::WlSubcompositor> for WaylandClient<T>
where
    T: Data + Clone + 'static,
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

impl<T> Dispatch<wl_surface::WlSurface> for WaylandClient<T>
where
    T: Data + Clone + 'static,
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
            let globals = self.globals.borrow();
            if let Some(output) = globals.outputs.iter().find(|o| o.output == output) {
                surface.set_buffer_scale(conn, output.scale);
                if let Some((i, view)) = self
                    .views
                    .iter_mut()
                    .enumerate()
                    .find(|a| a.1.eq_surface(&surface))
                {
                    self.current = Some(i);
                    if let Some(c_output) = view.surface.output.as_ref() {
                        if c_output.scale != output.scale {
                            view.surface.output = Some(output.clone());
                            std::mem::drop(output);
                            std::mem::drop(globals);
                            view.quick_render(
                                self.pool.as_mut().unwrap(),
                                &mut self.cache,
                                conn,
                                qh,
                            )
                        }
                    } else {
                        view.surface.output = Some(output.clone());
                        std::mem::drop(output);
                        std::mem::drop(globals);
                        view.quick_render(self.pool.as_mut().unwrap(), &mut self.cache, conn, qh)
                    }
                }
            }
        }
    }
}

impl<T> Dispatch<wl_callback::WlCallback> for WaylandClient<T>
where
    T: Data + Clone + 'static,
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
            for (i, view) in self.views.iter_mut().enumerate() {
                if view.state.pending_cb {
                    view.state.pending_cb = false;
                    // The view is rendered prior and the changes are commited here
                    let frame_time = (callback_data - view.state.time).min(50);
                    view.state.time = callback_data;
                    // Send a callback event with the timeout the view
                    match view.sync(cache, Event::Callback(frame_time), conn, qh) {
                        Damage::Partial => {
                            view.render(pool, cache, Damage::Partial, conn, qh);
                        }
                        Damage::Frame => {
                            cb = Some(i);
                            view.state.pending_cb = true;
                            view.render(pool, cache, Damage::Partial, conn, qh);
                        }
                        Damage::None => {}
                    }
                }
            }
            if let Some(i) = cb {
                if self.views[i].surface.frame(conn, qh, ()).is_ok() {
                    self.views[i].surface.deref().commit(conn);
                }
            }
        }
    }
}

impl<T> Dispatch<wl_subsurface::WlSubsurface> for WaylandClient<T>
where
    T: Data + Clone + 'static,
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

impl<T> Dispatch<wl_region::WlRegion> for WaylandClient<T>
where
    T: Data + Clone + 'static,
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

impl<T> Dispatch<xdg_toplevel::XdgToplevel> for WaylandClient<T>
where
    T: Data + Clone + 'static,
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
        if let Some((i, view)) = self.views.iter_mut().enumerate().find(|(_, a)| {
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
                    view.state.window_state = list_states(states);
                    if width > 0 && height > 0 {
                        view.state.constraint = BoxConstraints::new(
                            (width as f32, height as f32),
                            (width as f32, height as f32),
                        );
                    } else if view.state.constraint.is_default() {
                        let mut ctx = LayoutCtx::new(&mut self.cache);
                        let (r_width, r_height) = view
                            .widget
                            .layout(&mut ctx, &BoxConstraints::default())
                            .into();
                        view.state.constraint =
                            BoxConstraints::new((r_width, r_height), (r_width, r_height));
                        view.widget.layout(&mut ctx, &view.state.constraint);
                        toplevel.set_min_size(conn, r_width as i32, r_height as i32)
                    }
                }
                xdg_toplevel::Event::Close => {
                    view.surface.destroy(conn);
                    self.current = None;
                    let view = self.views.remove(i);
                    self.pool.as_mut().unwrap().remove(&view.surface, conn);
                }
                _ => {}
            }
        }
    }
}

fn list_states(states: Vec<u8>) -> Vec<WindowState> {
    states
        .chunks(4)
        .filter_map(|endian| {
            xdg_toplevel::State::try_from(u32::from_ne_bytes([
                endian[0], endian[1], endian[2], endian[3],
            ]))
            .ok()
        })
        .map(|state| WindowState::from(state))
        .collect()
}

impl<T> Dispatch<wl_shm::WlShm> for WaylandClient<T>
where
    T: Data + Clone + 'static,
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

impl<T> Dispatch<xdg_wm_base::XdgWmBase> for WaylandClient<T>
where
    T: Data + Clone + 'static,
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

impl<T> Dispatch<xdg_surface::XdgSurface> for WaylandClient<T>
where
    T: Data + Clone + 'static,
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
            if let Some(view) = self.views.iter_mut().find(|a| {
                let t_xdg_surface = xdg_surface;
                match &a.surface.shell {
                    wayland::Shell::Xdg { xdg_surface, .. } => xdg_surface.eq(t_xdg_surface),
                    _ => false,
                }
            }) {
                view.state.configured = true;
                view.update_scene(
                    self.pool.as_mut().unwrap(),
                    &mut self.cache,
                    Event::Configure,
                    conn,
                    qh,
                )
            }
        }
    }
}

impl<T> Dispatch<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1> for WaylandClient<T>
where
    T: Data + Clone + 'static,
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
            if let Some(view) = self.views.iter_mut().find(|a| {
                let t_layer_surface = layer_surface;
                match &a.surface.shell {
                    wayland::Shell::LayerShell { layer_surface, .. } => {
                        layer_surface.eq(t_layer_surface)
                    }
                    _ => false,
                }
            }) {
                if let Shell::LayerShell { config, .. } = &view.surface.shell {
                    if config.exclusive {
                        use zwlr_layer_surface_v1::Anchor;
                        if let Some(anchor) = config.anchor {
                            match anchor {
                                Anchor::Left | Anchor::Right => {
                                    let width = view.state.constraint.minimum_width();
                                    layer_surface.set_exclusive_zone(conn, width as i32)
                                }
                                Anchor::Top | Anchor::Bottom => {
                                    let height = view.state.constraint.minimum_height();
                                    layer_surface.set_exclusive_zone(conn, height as i32)
                                }
                                _ => {}
                            }
                        }
                    }
                }
                view.state.configured = true;
                view.state.constraint = BoxConstraints::new(
                    (width as f32, height as f32),
                    (width as f32, height as f32),
                );
                view.update_scene(
                    self.pool.as_mut().unwrap(),
                    &mut self.cache,
                    Event::Configure,
                    conn,
                    qh,
                )
            }
        }
    }
}

impl<T> Dispatch<zwlr_layer_shell_v1::ZwlrLayerShellV1> for WaylandClient<T>
where
    T: Data + Clone + 'static,
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

impl<T> Dispatch<wl_seat::WlSeat> for WaylandClient<T>
where
    T: Data + Clone + 'static,
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
                            seat.pointer = seat.seat.get_pointer(conn, qh, ()).ok();
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

impl<T> Dispatch<wl_pointer::WlPointer> for WaylandClient<T>
where
    T: Data + Clone + 'static,
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
        if let Some(index) = self.current {
            match event {
                wl_pointer::Event::Button {
                    serial,
                    time: _,
                    button,
                    state,
                } => {
                    if let Event::Pointer(_, _, p) = &mut self.event {
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
                    if let Event::Pointer(_, _, p) = &mut self.event {
                        if let WEnum::Value(axis) = axis {
                            *p = Pointer::Scroll {
                                orientation: match axis {
                                    Axis::VerticalScroll => Orientation::Vertical,
                                    Axis::HorizontalScroll => Orientation::Horizontal,
                                    _ => unreachable!(),
                                },
                                step: Step::Value(value as f32),
                            };
                        }
                    }
                }
                wl_pointer::Event::AxisDiscrete { axis, discrete } => {
                    if let Event::Pointer(_, _, p) = &mut self.event {
                        if let WEnum::Value(axis) = axis {
                            *p = Pointer::Scroll {
                                orientation: match axis {
                                    Axis::VerticalScroll => Orientation::Vertical,
                                    Axis::HorizontalScroll => Orientation::Horizontal,
                                    _ => unreachable!(),
                                },
                                step: Step::Increment(discrete),
                            };
                        }
                    }
                }
                wl_pointer::Event::Motion {
                    time: _,
                    surface_x,
                    surface_y,
                } => {
                    self.event = Event::Pointer(surface_x as f32, surface_y as f32, Pointer::Hover);
                }
                wl_pointer::Event::Frame => {
                    self.flush_event(conn, qh);
                }
                wl_pointer::Event::Leave { .. } => {
                    if let Event::Pointer(_, _, p) = &mut self.event {
                        *p = Pointer::Leave;
                    }
                    self.flush_event(conn, qh);
                    self.current = None;
                }
                wl_pointer::Event::Enter {
                    serial,
                    surface_x,
                    surface_y,
                    ..
                } => {
                    self.views[index].handle(conn, qh).set_cursor(Cursor::Arrow);
                    self.views[index].state.enter_serial = serial;
                    self.event = Event::Pointer(surface_x as f32, surface_y as f32, Pointer::Enter);
                    self.flush_event(conn, qh);
                }
                _ => {}
            }
        } else {
            match event {
                wl_pointer::Event::Enter { ref surface, .. } => {
                    self.current =
                        (0..self.views.len()).find(|i| self.views[*i].eq_surface(surface));
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

impl<T> Dispatch<wl_output::WlOutput> for WaylandClient<T>
where
    T: Data + Clone + 'static,
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

impl<T, I> DelegateDispatchBase<I> for WaylandClient<T>
where
    I: Proxy,
    T: Data + Clone,
{
    type UserData = ();
}

impl<T, I> DelegateDispatch<I, WaylandClient<T>> for WaylandClient<T>
where
    I: Proxy,
    T: Data + Clone,
    Self: Dispatch<I, UserData = Self::UserData>,
{
    fn event(
        data: &mut WaylandClient<T>,
        proxy: &I,
        event: <I as Proxy>::Event,
        udata: &Self::UserData,
        connhandle: &mut ConnectionHandle,
        qhandle: &QueueHandle<WaylandClient<T>>,
    ) {
        Dispatch::event(data, proxy, event, udata, connhandle, qhandle)
    }
}
