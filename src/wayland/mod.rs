pub mod app;

use crate::*;
use std::io::Write;
use crate::Canvas;
use smithay_client_toolkit::shm::{Format, MemPool};
use wayland_client::protocol::wl_buffer::WlBuffer;

const FORMAT: Format = Format::Argb8888;

pub struct Buffer<'b> {
    mmap: &'b mut [u8],
    canvas: &'b mut Canvas,
}

impl<'b> Buffer<'b> {
    fn new(mmap: &'b mut [u8], canvas: &'b mut Canvas) -> Self {
        Self { mmap, canvas }
    }
    pub fn canvas(&mut self) -> &mut Canvas {
        &mut self.canvas
    }
    pub fn merge(mut self) {
        self.mmap.write_all(&self.canvas).unwrap();
        self.mmap.flush().unwrap();
        self.canvas.clear();
    }
}

pub fn buffer<'b>(
    canvas: &'b mut Canvas,
    mempool: &'b mut MemPool,
) -> Result<(Buffer<'b>, WlBuffer), ()> {
    let width = canvas.width() as i32;
    let height = canvas.height() as i32;
    let stride = width * 4;
    if mempool.resize((stride * height) as usize).is_ok() {
        let wlbuf = mempool.buffer(0, width, height as i32, stride, FORMAT);
        Ok((Buffer::new(mempool.mmap(), canvas), wlbuf))
    } else {
        Err(())
    }
}
