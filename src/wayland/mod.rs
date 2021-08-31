pub mod app;

use crate::*;
use smithay_client_toolkit::shm::{MemPool, Format};
use wayland_client::protocol::{wl_buffer::WlBuffer, wl_surface::WlSurface};

const FORMAT:Format = Format::Argb8888;

pub struct Buffer<'b> {
    width: u32,
    height: u32,
    wlbuffer: WlBuffer,
    pub canvas: Canvas<'b>,
}

impl<'b> Geometry for Buffer<'b> {
    fn get_width(&self) -> u32 {
        self.width
    }
    fn get_height(&self) -> u32 {
        self.height
    }
    fn resize(&mut self, _width: u32, _height: u32) -> Result<(), Error> {
        Err(Error::Dimension(
            "\"buffer\" cannot be resized",
            self.get_width(),
            self.get_height(),
        ))
    }
}

impl<'b> Buffer<'b> {
    pub fn new<'a>(width: u32, height: u32, mempool: &'a mut MemPool) -> Result<Buffer, ()> {
        let stride = width * 4;
        if mempool.resize((stride * height) as usize).is_ok() {
            let buffer = mempool.buffer(0, width as i32, height as i32, stride as i32, FORMAT);
            Ok(
                Buffer {
                    width: width,
                    height: height,
                    wlbuffer: buffer,
                    canvas: Canvas::new(mempool.mmap(), width as u32, height as u32),
                }
            )
        } else {
            Err(())
        }
    }
    pub fn attach(&mut self, surface: &WlSurface, x: i32, y: i32) {
        surface.attach(Some(&self.wlbuffer), x, y);
    }
    pub fn get(&self) -> Option<WlBuffer> {
        Some(self.wlbuffer.clone())
    }
}
