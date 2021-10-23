pub mod app;

use crate::canvas::Canvas;
use crate::*;
use smithay_client_toolkit::shm::{Format, MemPool};
use std::io::Write;
use wayland_client::protocol::wl_buffer::WlBuffer;

const FORMAT: Format = Format::Argb8888;

pub struct Buffer<'b> {
    mmap: &'b mut [u8],
    canvas: &'b mut Canvas,
}

impl<'b> Buffer<'b> {
    fn new(mempool: &'b mut MemPool, canvas: &'b mut Canvas) -> Result<(Self, WlBuffer), ()> {
        let width = canvas.width() as i32;
        let height = canvas.height() as i32;
        let stride = width * 4;
        if mempool.resize((stride * height) as usize).is_ok() {
            let wlbuf = mempool.buffer(0, width, height as i32, stride, FORMAT);
            Ok((
                Self {
                    mmap: mempool.mmap(),
                    canvas,
                },
                wlbuf,
            ))
        } else {
            Err(())
        }
    }
    pub fn canvas(&mut self) -> &mut Canvas {
        self.canvas
    }
    pub fn merge(mut self) {
        self.mmap.write_all(&self.canvas).unwrap();
        self.mmap.flush().unwrap();
        self.canvas.flush_damage();
    }
}
