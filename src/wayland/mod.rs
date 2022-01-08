pub mod shell;

use tiny_skia::*;

use super::widgets::Alignment;
use super::Orientation;
pub use wayland_client::{
    protocol::wl_buffer::WlBuffer,
    protocol::wl_compositor::WlCompositor,
    protocol::wl_output::WlOutput,
    protocol::wl_region::WlRegion,
    protocol::wl_registry::WlRegistry,
    protocol::wl_seat::{Capability, WlSeat},
    protocol::wl_shm::{Format, WlShm},
    protocol::wl_shm_pool::WlShmPool,
    protocol::wl_subcompositor::WlSubcompositor,
    protocol::wl_surface::WlSurface,
    ConnectionHandle, WEnum,
};
use wayland_protocols::{
    wlr::unstable::layer_shell::v1::client::{
        zwlr_layer_shell_v1::{Layer, ZwlrLayerShellV1},
        zwlr_layer_surface_v1::{Anchor, KeyboardInteractivity, ZwlrLayerSurfaceV1},
    },
    xdg_shell::client::{xdg_surface, xdg_toplevel, xdg_wm_base},
};

use crate::context::Backend;

const FORMAT: Format = Format::Argb8888;
// fn buffer<'b>(mempool: &'b mut AutoMemPool, width: i32, height: i32) -> Result<(Backend, WlBuffer), ()> {
//     let stride = width * 4;
//     if mempool.resize((stride * height) as usize).is_ok() {
//         if let Ok((buf, wlbuf)) = mempool.buffer(width, height as i32, stride, FORMAT) {
//             if let Some(pixmap) = PixmapMut::from_bytes(buf, width as u32, height as u32) {
//                 return Ok((
//                     Self {
//                         backend: Backend::Pixmap(pixmap),
//                     },
//                     wlbuf,
//                 ));
//             }
//         }
//     }
//     Err(())
// }

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

pub enum ShellConfig {
    LayerShell(LayerShellConfig),
}

#[derive(Debug, Clone)]
pub struct LayerShellConfig {
    pub layer: Layer,
    pub anchor: Option<Anchor>,
    pub namespace: String,
    pub exclusive: i32,
    pub interactivity: KeyboardInteractivity,
    pub margin: [i32; 4],
}

impl Default for LayerShellConfig {
    fn default() -> Self {
        Self {
            layer: Layer::Top,
            anchor: None,
            exclusive: 0,
            interactivity: KeyboardInteractivity::None,
            namespace: "".to_string(),
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
    wl_output: Option<WlOutput>,
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
    pub capabilities: WEnum<Capability>,
}

impl Seat {
    fn new(seat: WlSeat) -> Self {
        Self {
            seat,
            name: String::new(),
            capabilities: WEnum::Unknown(0),
        }
    }
}

pub struct GlobalManager {
    pub outputs: Vec<Output>,
    pub seats: Vec<Seat>,
    pub shm: Option<WlShm>,
    pub registry: WlRegistry,
    pub compositor: Option<WlCompositor>,
    pub subcompositor: Option<WlSubcompositor>,
    pub wm_base: Option<xdg_wm_base::XdgWmBase>,
    pub layer_shell: Option<ZwlrLayerShellV1>,
}
