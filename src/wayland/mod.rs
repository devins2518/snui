pub mod app;

use crate::Canvas;
use smithay_client_toolkit::shm::{Format, MemPool};
use std::io::Write;
use wayland_client::protocol::wl_buffer::WlBuffer;

const FORMAT: Format = Format::Argb8888;

pub struct Buffer<'b> {
    slice: &'b mut [u8],
    canvas: Canvas,
}

impl<'b> Buffer<'b> {
    fn new(slice: &'b mut [u8], width: u32, height: u32) -> Self {
        Self {
            slice,
            canvas: Canvas::new(width, height),
        }
    }
    pub fn canvas(&mut self) -> &mut Canvas {
        &mut self.canvas
    }
    pub fn merge(&mut self) {
        self.slice.write_all(&self.canvas).unwrap();
        self.slice.flush().unwrap();
    }
}

pub fn buffer<'b>(
    width: u32,
    height: u32,
    mempool: &'b mut MemPool,
) -> Result<(Buffer, WlBuffer), ()> {
    let stride = width * 4;
    if mempool.resize((stride * height) as usize).is_ok() {
        let wlbuf = mempool.buffer(0, width as i32, height as i32, stride as i32, FORMAT);
        Ok((
            Buffer::new(mempool.mmap(), width as u32, height as u32),
            wlbuf,
        ))
    } else {
        Err(())
    }
}
