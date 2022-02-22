pub mod backend;

pub use smithay_client_toolkit::reexports::client::{
    protocol::wl_buffer::WlBuffer,
    protocol::wl_compositor::WlCompositor,
    protocol::wl_keyboard::WlKeyboard,
    protocol::wl_output::WlOutput,
    protocol::wl_pointer::WlPointer,
    protocol::wl_region::WlRegion,
    protocol::wl_registry::WlRegistry,
    protocol::wl_seat::{Capability, WlSeat},
    protocol::wl_shm::{Format, WlShm},
    protocol::wl_shm_pool::WlShmPool,
    protocol::wl_subcompositor::WlSubcompositor,
    protocol::wl_surface::WlSurface,
    ConnectionHandle, Dispatch, QueueHandle, WEnum,
};
use smithay_client_toolkit::reexports::protocols::{
    wlr::unstable::layer_shell::v1::client::{
        zwlr_layer_shell_v1::{Layer, ZwlrLayerShellV1},
        zwlr_layer_surface_v1::{Anchor, KeyboardInteractivity, ZwlrLayerSurfaceV1},
    },
    xdg_shell::client::{xdg_surface, xdg_toplevel, xdg_wm_base},
};
use smithay_client_toolkit::registry::RegistryState;
use smithay_client_toolkit::shm::ShmState;
use wayland_cursor::CursorTheme;

use crate::context::Backend;
use crate::PixmapMut;
use smithay_client_toolkit::shm::pool::multi::MultiPool;
use std::sync::Arc;
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    sync::atomic::AtomicBool,
};

const FORMAT: Format = Format::Argb8888;

fn buffer<'b, D>(
    pool: &'b mut MultiPool<WlSurface>,
    width: u32,
    height: u32,
    surface: &WlSurface,
    conn: &mut ConnectionHandle,
    qh: &QueueHandle<D>,
) -> Option<(usize, WlBuffer, Backend<'b>)>
where
    D: Dispatch<WlBuffer, UserData = Arc<AtomicBool>> + 'static,
{
    let stride = width * 4;
    if let Some((offset, buffer, slice)) = pool.create_buffer(
        width as i32,
        stride as i32,
        height as i32,
        surface,
        FORMAT,
        conn,
        qh,
    ) {
        if let Some(pixmap) = PixmapMut::from_bytes(slice, width, height) {
            return Some((offset, buffer, Backend::Pixmap(pixmap)));
        }
    }
    None
}

#[derive(Debug, Clone)]
pub enum Shell {
    LayerShell {
        config: LayerShellConfig,
        layer_surface: ZwlrLayerSurfaceV1,
    },
    Xdg {
        xdg_surface: xdg_surface::XdgSurface,
        toplevel: xdg_toplevel::XdgToplevel,
    },
}

#[derive(Debug, Clone)]
pub struct LayerShellConfig {
    pub layer: Layer,
    pub anchor: Option<Anchor>,
    pub namespace: String,
    pub exclusive: bool,
    pub interactivity: KeyboardInteractivity,
    pub margin: [i32; 4],
}

impl Default for LayerShellConfig {
    fn default() -> Self {
        Self {
            layer: Layer::Top,
            anchor: Some(Anchor::all()),
            exclusive: false,
            interactivity: KeyboardInteractivity::None,
            namespace: String::from("snui"),
            margin: [0; 4],
        }
    }
}

impl Shell {
    pub fn destroy(&self, conn: &mut ConnectionHandle) {
        match self {
            Shell::LayerShell {
                config: _,
                layer_surface,
            } => {
                layer_surface.destroy(conn);
            }
            Self::Xdg {
                xdg_surface,
                toplevel,
            } => {
                xdg_surface.destroy(conn);
                toplevel.destroy(conn);
            }
        }
    }
    pub fn set_size(&self, conn: &mut ConnectionHandle, width: u32, height: u32) {
        match self {
            Shell::LayerShell {
                config: _,
                layer_surface,
            } => {
                layer_surface.set_size(conn, width, height);
            }
            _ => {}
        }
    }
}

#[derive(Debug, Clone)]
pub struct Surface {
    shell: Shell,
    wl_region: WlRegion,
    wl_surface: WlSurface,
    output: Option<Output>,
    wl_buffer: Option<WlBuffer>,
    previous: Option<Box<Self>>,
}

#[derive(Debug, Clone)]
pub struct Output {
    pub width: i32,
    pub height: i32,
    pub scale: i32,
    pub physical_width: i32,
    pub physical_height: i32,
    pub name: String,
    pub output: WlOutput,
    pub refresh: i32,
}

impl Output {
    pub fn new(output: WlOutput) -> Self {
        Output {
            output,
            name: String::new(),
            refresh: 60,
            scale: 1,
            height: 1080,
            width: 1920,
            physical_height: 1080,
            physical_width: 1920,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Seat {
    pub seat: WlSeat,
    pub name: String,
    pub pointer: Option<WlPointer>,
    pub keyboard: Option<WlKeyboard>,
    pub capabilities: WEnum<Capability>,
}

impl Seat {
    fn new(seat: WlSeat) -> Self {
        Self {
            seat,
            pointer: None,
            keyboard: None,
            name: String::new(),
            capabilities: WEnum::Unknown(0),
        }
    }
}

pub struct GlobalManager {
    outputs: Vec<Output>,
    seats: Vec<Seat>,
    shm: Option<WlShm>,
    shm_state: ShmState,
    registry: RegistryState,
    pointer_surface: Option<WlSurface>,
    compositor: Option<WlCompositor>,
    cursor_theme: HashMap<u32, CursorTheme>,
    subcompositor: Option<WlSubcompositor>,
    wm_base: Option<xdg_wm_base::XdgWmBase>,
    layer_shell: Option<ZwlrLayerShellV1>,
}

impl GlobalManager {
    pub fn new(registry: WlRegistry) -> Self {
        Self {
            outputs: Vec::new(),
            seats: Vec::new(),
            shm: None,
            shm_state: ShmState::new(),
            registry: RegistryState::new(registry),
            pointer_surface: None,
            cursor_theme: HashMap::new(),
            compositor: None,
            subcompositor: None,
            wm_base: None,
            layer_shell: None,
        }
    }
    pub fn destroy(&self, conn: &mut ConnectionHandle) {
        if let Some(subcompositor) = &self.subcompositor {
            subcompositor.destroy(conn);
        }
        if let Some(wm_base) = &self.wm_base {
            wm_base.destroy(conn);
        }
        if let Some(pointer_surface) = &self.pointer_surface {
            pointer_surface.destroy(conn);
        }
        if let Some(layer_shell) = &self.layer_shell {
            layer_shell.destroy(conn);
        }
        for output in &self.outputs {
            output.output.release(conn);
        }
        for seat in &self.seats {
            seat.seat.release(conn);
        }
    }
}

impl Deref for GlobalManager {
    type Target = RegistryState;
    fn deref(&self) -> &Self::Target {
        &self.registry
    }
}

impl DerefMut for GlobalManager {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.registry
    }
}
