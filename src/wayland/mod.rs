pub mod app;

use crate::Canvas;
use wayland_client::protocol::wl_buffer::WlBuffer;
use smithay_client_toolkit::shm::{Format, MemPool};

const FORMAT: Format = Format::Argb8888;

pub fn buffer<'a>(width: u32, height: u32, mempool: &'a mut MemPool) -> Result<(Canvas, WlBuffer), ()> {
    let stride = width * 4;
    if mempool.resize((stride * height) as usize).is_ok() {
        let buffer = mempool.buffer(0, width as i32, height as i32, stride as i32, FORMAT);
        Ok((Canvas::new(mempool.mmap(), width as u32, height as u32), buffer))
    } else {
        Err(())
    }
}
