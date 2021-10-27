pub mod app;

use crate::context::Context;
use crate::*;
use smithay_client_toolkit::shm::{Format, MemPool};
use std::io::Write;
use wayland_client::protocol::wl_buffer::WlBuffer;

const FORMAT: Format = Format::Argb8888;

pub struct Buffer<'b> {
    mmap: &'b mut [u8],
    ctx: &'b mut Context,
}

impl<'b> Buffer<'b> {
    fn new(mempool: &'b mut MemPool, ctx: &'b mut Context) -> Result<(Self, WlBuffer), ()> {
        let width = ctx.width() as i32;
        let height = ctx.height() as i32;
        let stride = width * 4;
        if mempool.resize((stride * height) as usize).is_ok() {
            let wlbuf = mempool.buffer(0, width, height as i32, stride, FORMAT);
            Ok((
                Self {
                    mmap: mempool.mmap(),
                    ctx,
                },
                wlbuf,
            ))
        } else {
            Err(())
        }
    }
    pub fn ctx(&mut self) -> &mut Context {
        self.ctx
    }
    pub fn merge(mut self) {
        self.mmap.write_all(&self.ctx).unwrap();
        self.mmap.flush().unwrap();
        self.ctx.flush();
    }
}
