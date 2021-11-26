pub mod shell;

use crate::context::Backend;
use crate::*;
use smithay_client_toolkit::shm::{Format, MemPool};
use std::io::Write;
use wayland_client::protocol::wl_buffer::WlBuffer;

const FORMAT: Format = Format::Argb8888;

pub struct Buffer<'b> {
    mmap: &'b mut [u8],
    backend: Backend<'b>,
}

impl<'b> Buffer<'b> {
    fn new(mempool: &'b mut MemPool, backend: Backend<'b>) -> Result<(Self, WlBuffer), ()> {
        let width = backend.width() as i32;
        let height = backend.height() as i32;
        let stride = width * 4;
        if mempool.resize((stride * height) as usize).is_ok() {
            let wlbuf = mempool.buffer(0, width, height as i32, stride, FORMAT);
            Ok((
                Self {
                    mmap: mempool.mmap(),
                    backend,
                },
                wlbuf,
            ))
        } else {
            Err(())
        }
    }
    pub fn merge(mut self) {
        self.mmap.write_all(&self.backend).unwrap();
        self.mmap.flush().unwrap();
    }
}
