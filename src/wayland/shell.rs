use crate::context::DrawContext;
use crate::controller::{Controller, TryFromArg};
use crate::font::FontCache;
use crate::scene::*;
use crate::wayland::{GlobalManager, Output, Seat, Shell, Surface};
use crate::widgets::window::WindowMessage;
use crate::*;
// use smithay_client_toolkit::reexports::calloop::{EventLoop, LoopHandle, RegistrationToken};
// use smithay_client_toolkit::seat::keyboard::ModifiersState;
// use smithay_client_toolkit::shm::AutoMemPool;
// use smithay_client_toolkit::WaylandSource;

use std::cell::RefCell;
use std::mem::take;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use wayland_client::{
    protocol::wl_buffer,
    protocol::wl_callback,
    protocol::wl_compositor,
    protocol::wl_output,
    protocol::wl_pointer::{self, Axis, ButtonState},
    protocol::wl_region,
    protocol::wl_registry,
    protocol::wl_seat::{self, Capability},
    protocol::wl_shm,
    protocol::wl_subcompositor,
    protocol::wl_subsurface,
    protocol::wl_surface,
    Connection, ConnectionHandle, Dispatch, QueueHandle, WEnum,
};
use wayland_protocols::{
    wlr::unstable::layer_shell::v1::client::{zwlr_layer_shell_v1, zwlr_layer_surface_v1},
    xdg_shell::client::{xdg_surface, xdg_toplevel, xdg_wm_base},
};

pub struct WaylandClient<M, C>
where
    M: TryInto<WindowMessage> + 'static,
    C: Controller<M> + Clone + 'static,
{
    focus: Option<usize>,
    font_cache: FontCache,
    connection: Connection,
    event_buffer: Event<'static, M>,
    globals: Rc<RefCell<GlobalManager>>,
    applications: Vec<Application<M, C>>,
}

impl<M, C> WaylandClient<M, C>
where
    M: TryInto<WindowMessage> + 'static,
    C: Controller<M> + Clone,
{
    fn flush_ev_buffer(&mut self) {
        if let Some(i) = self.focus {
            let ev = take(&mut self.event_buffer);
            // Forward the event to the Application
            todo!();
            if !self.applications[i].state.configured {
                self.applications.remove(i);
            }
        }
    }
    fn fwd_event(&mut self, event: Event<M>) {
        if let Some(i) = self.focus {
            todo!();
            if !self.applications[i].state.configured {
                self.applications.remove(i);
            }
        }
    }
}

pub struct Application<M, C>
where
    C: Controller<M>,
{
    state: State,
    controller: C,
    // mempool: AutoMemPool,
    surface: Option<Surface>,
    widget: Box<dyn Widget<M>>,
    globals: Rc<RefCell<GlobalManager>>,
}

struct State {
    time: u32,
    configured: bool,
    pending_cb: bool,
    render_node: RenderNode,
}

impl Default for State {
    fn default() -> Self {
        Self {
            time: 0,
            configured: false,
            pending_cb: false,
            render_node: RenderNode::None,
        }
    }
}

impl<M, C> Application<M, C>
where
    C: Controller<M>,
{
    fn eq_surface(&self, surface: &wl_surface::WlSurface) -> bool {
        match self.surface.as_ref() {
            Some(s) => s.wl_surface.eq(surface),
            _ => false,
        }
    }
    fn set_size(&mut self, conn: &mut ConnectionHandle, width: f32, height: f32) {
        let (width, height) = Geometry::set_size(self, width, height)
            .err()
            .unwrap_or((width, height));

        if let Some(s) = self.surface.as_ref() {
            s.set_size(conn, width as u32, height as u32)
        }
    }
    unsafe fn globals(&self) -> Rc<GlobalManager> {
        let ptr = self.globals.as_ref().as_ptr();
        Rc::increment_strong_count(ptr);
        Rc::from_raw(ptr)
    }
}

impl<M, C> Application<M, C>
where
    M: TryInto<WindowMessage>,
    C: Controller<M> + std::clone::Clone,
{
    fn sync(&mut self, conn: &mut ConnectionHandle, fc: &mut FontCache, event: Event<M>) -> Damage {
        let mut damage = Damage::None;
        let mut ctx = SyncContext::new(&mut self.controller, fc);
        damage = damage.max(self.widget.sync(&mut ctx, event));

        while let Ok(msg) = ctx.sync() {
            damage = damage.max(self.widget.sync(&mut ctx, Event::Message(&msg)));
            if let Some(s) = self.surface.as_mut() {
                if let Ok(wm) = msg.try_into() {
                    match wm {
                        WindowMessage::Close => {
                            // The WaylandClient will check if it's configured
                            // and remove the Application if it's not.
                            self.state.configured = false;
                            s.destroy(conn);
                            self.surface = None;
                        }
                        WindowMessage::Move => match &s.shell {
                            Shell::Xdg { toplevel, .. } => {
                                todo!()
                            }
                            _ => {}
                        },
                        WindowMessage::Minimize => match &s.shell {
                            Shell::Xdg { toplevel, .. } => {
                                toplevel.set_minimized(conn);
                            }
                            _ => {}
                        },
                        WindowMessage::Maximize => {
                            match &s.shell {
                                Shell::Xdg { toplevel, .. } => {
                                    toplevel.set_maximized(conn);
                                }
                                Shell::LayerShell { layer_surface, .. } => {
                                    // The following Configure event should be adjusted to
                                    // the size of the output
                                    layer_surface.set_size(conn, 1 << 31, 1 << 31);
                                }
                            }
                        }
                        WindowMessage::Title(title) => match &s.shell {
                            Shell::Xdg { toplevel, .. } => {
                                toplevel.set_title(conn, title);
                            }
                            _ => {}
                        },
                    }
                    return Damage::None;
                }
            }
        }

        damage
    }
    fn update_scene(
        &mut self,
        conn: &mut ConnectionHandle,
        qh: &QueueHandle<WaylandClient<M, C>>,
        fc: &mut FontCache,
        event: Event<M>,
    ) {
        if !self.state.pending_cb {
            match self.sync(conn, fc, event) {
                Damage::Partial => todo!(),
                Damage::Frame => {
                    if let Some(s) = self.surface.as_ref() {
                        if s.wl_surface.frame(conn, qh, ()).is_ok() {
                            self.state.pending_cb = true;
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

impl<M, C> Geometry for Application<M, C>
where
    C: Controller<M>,
{
    fn height(&self) -> f32 {
        self.widget.height()
    }
    fn width(&self) -> f32 {
        self.widget.width()
    }
    fn set_width(&mut self, width: f32) -> Result<(), f32> {
        self.widget.set_width(width)
    }
    fn set_height(&mut self, height: f32) -> Result<(), f32> {
        self.widget.set_height(height)
    }
}

impl GlobalManager {
    fn create_xdg_surface<M, C>(
        &self,
        conn: &mut ConnectionHandle,
        qh: &QueueHandle<WaylandClient<M, C>>,
    ) -> Option<Surface>
    where
        M: TryInto<WindowMessage>,
        C: Controller<M> + Clone,
    {
        let wm_base = self.wm_base.as_ref()?;
        let compositor = self.compositor.as_ref()?;

        let wl_surface = compositor.create_surface(conn, qh, ()).ok()?;
        let wl_region = compositor.create_region(conn, qh, ()).ok()?;
        let xdg_surface = wm_base.get_xdg_surface(conn, &wl_surface, qh, ()).ok()?;
        let toplevel = xdg_surface.get_toplevel(conn, qh, ()).ok()?;

        wl_surface.commit(conn);

        Some(Surface {
            wl_surface,
            wl_region,
            wl_buffer: None,
            previous: None,
            wl_output: None,
            shell: Shell::Xdg {
                xdg_surface,
                toplevel,
            },
        })
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
            wl_output: None,
            previous: if let Some(surface) = previous {
                Some(Box::new(surface))
            } else {
                None
            },
            wl_buffer: None,
        }
    }
    fn commit(&mut self, ch: &mut ConnectionHandle) {
        self.wl_surface.commit(ch);
        std::mem::drop(&mut self.previous);
        self.previous = None;
    }
    fn destroy(&mut self, ch: &mut ConnectionHandle) {
        self.wl_surface.destroy(ch);
        self.wl_region.destroy(ch);
        self.shell.destroy(ch);
        if let Some(buffer) = self.wl_buffer.as_ref() {
            buffer.destroy(ch);
        }
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
    fn damage(&self, ch: &mut ConnectionHandle, report: &[Region]) {
        for d in report {
            self.wl_surface
                .damage(ch, d.x as i32, d.y as i32, d.width as i32, d.height as i32);
        }
    }
    fn attach_buffer(&mut self, ch: &mut ConnectionHandle, wl_buffer: wl_buffer::WlBuffer) {
        self.wl_buffer = Some(wl_buffer);
        self.wl_surface.attach(ch, self.wl_buffer.as_ref(), 0, 0);
    }
}

impl<M, C> Dispatch<wl_registry::WlRegistry> for WaylandClient<M, C>
where
    M: TryInto<WindowMessage> + 'static,
    C: Controller<M> + Clone + 'static,
{
    type UserData = ();

    fn event(
        &mut self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &Self::UserData,
        conn: &mut ConnectionHandle,
        qh: &wayland_client::QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name, interface, ..
        } = event
        {
            match &interface[..] {
                "wl_compositor" => {
                    self.globals.borrow_mut().compositor = registry
                        .bind::<wl_compositor::WlCompositor, _>(conn, name, 1, qh, ())
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
                }
                "wl_seat" => {
                    registry
                        .bind::<wl_seat::WlSeat, _>(conn, name, 1, qh, ())
                        .unwrap();
                }
                "wl_output" => {
                    registry
                        .bind::<wl_output::WlOutput, _>(conn, name, 1, qh, ())
                        .unwrap();
                }
                "zwlr_layer_shell" => {
                    self.globals.borrow_mut().layer_shell = registry
                        .bind::<zwlr_layer_shell_v1::ZwlrLayerShellV1, _>(conn, name, 1, qh, ())
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

impl<M, C> Dispatch<wl_compositor::WlCompositor> for WaylandClient<M, C>
where
    M: TryInto<WindowMessage> + 'static,
    C: Controller<M> + Clone + 'static,
{
    type UserData = ();

    fn event(
        &mut self,
        _: &wl_compositor::WlCompositor,
        _: wl_compositor::Event,
        _: &Self::UserData,
        _: &mut ConnectionHandle,
        _: &wayland_client::QueueHandle<Self>,
    ) {
        // wl_compositor has no event
    }
}

impl<M, C> Dispatch<wl_subcompositor::WlSubcompositor> for WaylandClient<M, C>
where
    M: TryInto<WindowMessage> + 'static,
    C: Controller<M> + Clone + 'static,
{
    type UserData = ();

    fn event(
        &mut self,
        _: &wl_subcompositor::WlSubcompositor,
        _: wl_subcompositor::Event,
        _: &Self::UserData,
        _: &mut ConnectionHandle,
        _: &wayland_client::QueueHandle<Self>,
    ) {
        // wl_compositor has no event
    }
}

impl<M, C> Dispatch<wl_surface::WlSurface> for WaylandClient<M, C>
where
    M: TryInto<WindowMessage> + 'static,
    C: Controller<M> + Clone + 'static,
{
    type UserData = ();

    fn event(
        &mut self,
        surface: &wl_surface::WlSurface,
        event: wl_surface::Event,
        _: &Self::UserData,
        conn: &mut ConnectionHandle,
        _: &wayland_client::QueueHandle<Self>,
    ) {
        if let wl_surface::Event::Enter { output } = event {
            if let Some(o) = self
                .globals
                .borrow()
                .outputs
                .iter()
                .find(|o| o.output == output)
            {
                surface.set_buffer_scale(conn, o.scale);
                if let Some(application) = self
                    .applications
                    .iter_mut()
                    .find(|a| a.eq_surface(&surface))
                {
                    application.surface.as_mut().unwrap().wl_output = Some(output);
                }
            }
        }
    }
}

impl<M, C> Dispatch<wl_callback::WlCallback> for WaylandClient<M, C>
where
    M: TryInto<WindowMessage> + 'static,
    C: Controller<M> + Clone + 'static,
{
    type UserData = ();

    fn event(
        &mut self,
        callback: &wl_callback::WlCallback,
        event: wl_callback::Event,
        _: &Self::UserData,
        conn: &mut ConnectionHandle,
        qh: &wayland_client::QueueHandle<Self>,
    ) {
        if let wl_callback::Event::Done { callback_data } = event {
            for application in self.applications.iter_mut() {
                if application.state.pending_cb {
                    application.state.pending_cb = false;
                    // The application is rendered prior and the changes are commited here
                    application.surface.as_mut().unwrap().commit(conn);
                    let frame_time = (callback_data - application.state.time).min(60);
                    // Send a callback event with the timeout the application
                    application.update_scene(
                        conn,
                        qh,
                        &mut self.font_cache,
                        Event::Callback(frame_time),
                    );
                }
            }
        }
    }
}

impl<M, C> Dispatch<wl_subsurface::WlSubsurface> for WaylandClient<M, C>
where
    M: TryInto<WindowMessage> + 'static,
    C: Controller<M> + Clone + 'static,
{
    type UserData = ();

    fn event(
        &mut self,
        _: &wl_subsurface::WlSubsurface,
        _: wl_subsurface::Event,
        _: &Self::UserData,
        _: &mut ConnectionHandle,
        _: &wayland_client::QueueHandle<Self>,
    ) {
        // wl_subsurface has no event
    }
}

impl<M, C> Dispatch<wl_region::WlRegion> for WaylandClient<M, C>
where
    M: TryInto<WindowMessage> + 'static,
    C: Controller<M> + Clone + 'static,
{
    type UserData = ();

    fn event(
        &mut self,
        _: &wl_region::WlRegion,
        _: wl_region::Event,
        _: &Self::UserData,
        _: &mut ConnectionHandle,
        _: &wayland_client::QueueHandle<Self>,
    ) {
        // wl_region has no event
    }
}

impl<M, C> Dispatch<xdg_toplevel::XdgToplevel> for WaylandClient<M, C>
where
    M: TryInto<WindowMessage> + 'static,
    C: Controller<M> + Clone + 'static,
{
    type UserData = ();

    fn event(
        &mut self,
        toplevel: &xdg_toplevel::XdgToplevel,
        event: xdg_toplevel::Event,
        _: &Self::UserData,
        conn: &mut ConnectionHandle,
        _: &wayland_client::QueueHandle<Self>,
    ) {
        if let Some(application) = self.applications.iter_mut().find(|a| {
            if let Some(s) = a.surface.as_ref() {
                let t_toplevel = toplevel;
                match &s.shell {
                    wayland::Shell::Xdg { toplevel, .. } => toplevel.eq(t_toplevel),
                    _ => false,
                }
            } else {
                false
            }
        }) {
            match event {
                xdg_toplevel::Event::Configure { width, height, .. } => {
                    application.set_size(conn, width as f32, height as f32);
                    todo!()
                }
                xdg_toplevel::Event::Close => {
                    application.surface.as_mut().unwrap().destroy(conn);
                }
                _ => {}
            }
        }
        // wl_region has no event
    }
}

impl<M, C> Dispatch<wl_shm::WlShm> for WaylandClient<M, C>
where
    M: TryInto<WindowMessage> + 'static,
    C: Controller<M> + Clone + 'static,
{
    type UserData = ();

    fn event(
        &mut self,
        _: &wl_shm::WlShm,
        _: wl_shm::Event,
        _: &Self::UserData,
        _: &mut ConnectionHandle,
        _: &wayland_client::QueueHandle<Self>,
    ) {
        // wl_shm has no event
    }
}

impl<M, C> Dispatch<xdg_wm_base::XdgWmBase> for WaylandClient<M, C>
where
    M: TryInto<WindowMessage> + 'static,
    C: Controller<M> + Clone + 'static,
{
    type UserData = ();

    fn event(
        &mut self,
        wm_base: &xdg_wm_base::XdgWmBase,
        event: xdg_wm_base::Event,
        _: &Self::UserData,
        conn: &mut ConnectionHandle,
        _: &wayland_client::QueueHandle<Self>,
    ) {
        if let xdg_wm_base::Event::Ping { serial } = event {
            wm_base.pong(conn, serial);
        }
    }
}

impl<M, C> Dispatch<xdg_surface::XdgSurface> for WaylandClient<M, C>
where
    M: TryInto<WindowMessage> + 'static,
    C: Controller<M> + Clone + 'static,
{
    type UserData = ();

    fn event(
        &mut self,
        xdg_surface: &xdg_surface::XdgSurface,
        event: xdg_surface::Event,
        _: &Self::UserData,
        conn: &mut ConnectionHandle,
        _: &wayland_client::QueueHandle<Self>,
    ) {
        if let xdg_surface::Event::Configure { serial, .. } = event {
            xdg_surface.ack_configure(conn, serial);
            if let Some(application) = self.applications.iter_mut().find(|a| {
                if let Some(s) = a.surface.as_ref() {
                    let t_xdg_surface = xdg_surface;
                    match &s.shell {
                        wayland::Shell::Xdg { xdg_surface, .. } => xdg_surface.eq(t_xdg_surface),
                        _ => false,
                    }
                } else {
                    false
                }
            }) {
                application.state.configured = true;
                todo!()
            }
        }
    }
}

impl<M, C> Dispatch<zwlr_layer_shell_v1::ZwlrLayerShellV1> for WaylandClient<M, C>
where
    M: TryInto<WindowMessage> + 'static,
    C: Controller<M> + Clone + 'static,
{
    type UserData = ();

    fn event(
        &mut self,
        _: &zwlr_layer_shell_v1::ZwlrLayerShellV1,
        _: zwlr_layer_shell_v1::Event,
        _: &Self::UserData,
        _: &mut ConnectionHandle,
        _: &wayland_client::QueueHandle<Self>,
    ) {
        // layer_shell has no event
    }
}

impl<M, C> Dispatch<wl_seat::WlSeat> for WaylandClient<M, C>
where
    M: TryInto<WindowMessage> + 'static,
    C: Controller<M> + Clone + 'static,
{
    type UserData = ();

    fn event(
        &mut self,
        seat: &wl_seat::WlSeat,
        event: wl_seat::Event,
        data: &Self::UserData,
        conn: &mut ConnectionHandle,
        qh: &wayland_client::QueueHandle<Self>,
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

impl<M, C> Dispatch<wl_pointer::WlPointer> for WaylandClient<M, C>
where
    M: TryInto<WindowMessage> + 'static,
    C: Controller<M> + Clone + 'static,
{
    type UserData = ();

    fn event(
        &mut self,
        pointer: &wl_pointer::WlPointer,
        event: wl_pointer::Event,
        data: &Self::UserData,
        conn: &mut ConnectionHandle,
        qh: &wayland_client::QueueHandle<Self>,
    ) {
        if self.focus.is_some() {
            match event {
                wl_pointer::Event::Button {
                    serial: _,
                    time,
                    button,
                    state,
                } => {
                    if let Event::Pointer(_, _, p) = &mut self.event_buffer {
                        *p = Pointer::MouseClick {
                            time,
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
                    self.flush_ev_buffer();
                }
                _ => {}
            }
        } else {
            match event {
                wl_pointer::Event::Enter { ref surface, .. } => {
                    self.focus = (0..self.applications.len())
                        .find(|i| self.applications[*i].eq_surface(surface));
                    self.event(pointer, event, data, conn, qh);
                }
                wl_pointer::Event::Leave { .. } => {
                    self.focus = None;
                    // Call dispatch method of the Application
                    todo!()
                }
                _ => {}
            }
        }
    }
}

impl<M, C> Dispatch<wl_output::WlOutput> for WaylandClient<M, C>
where
    M: TryInto<WindowMessage> + 'static,
    C: Controller<M> + Clone + 'static,
{
    type UserData = ();

    fn event(
        &mut self,
        output: &wl_output::WlOutput,
        event: wl_output::Event,
        data: &Self::UserData,
        conn: &mut ConnectionHandle,
        qh: &wayland_client::QueueHandle<Self>,
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
