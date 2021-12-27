pub mod shell;

use tiny_skia::*;

use super::Orientation;
use super::widgets::Alignment;
pub use smithay_client_toolkit;
use smithay_client_toolkit::shm::AutoMemPool;
pub use smithay_client_toolkit::reexports::client::{
    Main,
	protocol::wl_region::WlRegion,
	protocol::wl_seat::{Capability, WlSeat},
	protocol::wl_shm::WlShm,
	protocol::wl_surface::WlSurface,
    protocol::wl_buffer::WlBuffer,
	protocol::wl_compositor::WlCompositor,
	protocol::wl_output::WlOutput,
};
pub use smithay_client_toolkit::reexports::protocols::wlr::unstable::layer_shell::v1::client::{
    zwlr_layer_shell_v1::Layer, zwlr_layer_shell_v1::ZwlrLayerShellV1,
    zwlr_layer_surface_v1::Anchor, zwlr_layer_surface_v1::KeyboardInteractivity,
    zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
};

use crate::context::Backend;
use smithay_client_toolkit::shm::Format;

const FORMAT: Format = Format::Argb8888;

pub enum DisplayEvent {
    Output
}

pub struct Buffer<'b> {
    pub backend: Backend<'b>,
}

impl<'b> Buffer<'b> {
    fn new(mempool: &'b mut AutoMemPool, width: i32, height: i32) -> Result<(Self, WlBuffer), ()> {
        let stride = width * 4;
        if mempool.resize((stride * height) as usize).is_ok() {
            if let Ok((buf, wlbuf)) = mempool.buffer(width, height as i32, stride, FORMAT) {
                if let Some(pixmap) = PixmapMut::from_bytes(buf, width as u32, height as u32) {
                    return Ok((
                        Self {
                            backend: Backend::Pixmap(pixmap),
                        },
                        wlbuf,
                    ));
                }
            }
        }
        Err(())
    }
}

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
    pub fn anchor(mut self, x: Alignment, y: Alignment) -> Self {
        let mut anchor = Anchor::empty();
        match x {
            Alignment::Start => anchor.insert(Anchor::Left),
            Alignment::End => anchor.insert(Anchor::Right),
            _ => {}
        }
        match y {
            Alignment::Start => anchor.insert(Anchor::Top),
            Alignment::End => anchor.insert(Anchor::Top),
            _ => {}
        }
        self.anchor = Some(anchor);
        self
    }
    pub fn anchor_side(mut self, alignment: Alignment, orientation: Orientation) -> Self {
        let mut anchor = Anchor::empty();
        match orientation {
            Orientation::Horizontal => match alignment {
                Alignment::Start => {
                    anchor.insert(Anchor::Top);
                    anchor.insert(Anchor::Left);
                    anchor.insert(Anchor::Bottom);
                }
                Alignment::End => {
                    anchor.insert(Anchor::Top);
                    anchor.insert(Anchor::Right);
                    anchor.insert(Anchor::Bottom);
                }
                _ => {}
            }
            Orientation::Vertical => match alignment {
                Alignment::Start => {
                    anchor.insert(Anchor::Top);
                    anchor.insert(Anchor::Left);
                    anchor.insert(Anchor::Right);
                }
                Alignment::End => {
                    anchor.insert(Anchor::Bottom);
                    anchor.insert(Anchor::Left);
                    anchor.insert(Anchor::Right);
                }
                _ => {}
            }
        }
        self.anchor = Some(anchor);
        self
    }
    pub fn output(mut self, output: WlOutput) -> Self {
        self.output = Some(output);
        self
    }
    pub fn background(mut self) -> Self {
        self.layer = Layer::Background;
        self
    }
    pub fn bottom(mut self) -> Self {
        self.layer = Layer::Bottom;
        self
    }
    pub fn overlay(mut self) -> Self {
        self.layer = Layer::Overlay;
        self
    }
    pub fn margin(mut self, top: i32, right: i32, bottom: i32, left: i32) -> Self {
        self.margin = [top, right, bottom, left];
        self
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
    pub fn destroy(&self) {
        match self {
            Shell::LayerShell { config: _, surface } => {
                surface.destroy();
            }
        }
    }
    pub fn set_size(&self, width: u32, height: u32) {
        match self {
            Shell::LayerShell { config: _, surface } => {
                surface.set_size(width, height);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Surface {
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

pub enum MultiGlobal {
    Output(Output),
}

pub struct Globals {
    pub outputs: Vec<Output>,
    pub seats: Vec<Seat>,
    pub shm: Option<Main<WlShm>>,
    pub compositor: Option<Main<WlCompositor>>,
    pub shell: Option<Main<ZwlrLayerShellV1>>,
}
