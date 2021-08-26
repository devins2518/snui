pub mod app;

use crate::widgets::render;
use crate::*;
use smithay_client_toolkit::shm::{AutoMemPool, Format};
use wayland_client::protocol::{wl_buffer::WlBuffer, wl_surface::WlSurface};

//TO-DO Use MemPool instead of AutoMemPool
pub struct Buffer<'b> {
    width: u32,
    height: u32,
    wl_buffer: WlBuffer,
    canvas: &'b mut [u8],
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

impl<'b> Canvas for Buffer<'b> {
    fn get_buf(&self) -> &[u8] {
        &self.canvas
    }
    fn get_mut_buf(&mut self) -> &mut [u8] {
        &mut self.canvas
    }
    fn composite(&mut self, surface: &(impl Canvas + Geometry), x: u32, y: u32) {
        let width = self.get_width();
        render(self.get_mut_buf(), surface, width as usize, x, y);
    }
    fn size(&self) -> usize {
        (self.width * self.height * 4) as usize
    }
}

impl<'b> Buffer<'b> {
    pub fn new<'a>(width: i32, height: i32, stride: i32, mempool: &'a mut AutoMemPool) -> Buffer {
        let format = Format::Argb8888;
        let buffer = mempool.buffer(width, height, stride, format).unwrap();
        Buffer {
            width: width as u32,
            height: height as u32,
            wl_buffer: buffer.1,
            canvas: buffer.0,
        }
    }
    pub fn attach(&mut self, surface: &WlSurface, x: i32, y: i32) {
        surface.attach(Some(&self.wl_buffer), x, y);
    }
    pub fn get_wl_buffer(&self) -> Option<WlBuffer> {
        Some(self.wl_buffer.clone())
    }
}
