use crate::snui::*;
use smithay_client_toolkit::shm::{AutoMemPool, Format};
use std::convert::TryInto;
use wayland_client::protocol::{wl_buffer::WlBuffer, wl_surface::WlSurface};
use std::io::{Write, Seek, SeekFrom, BufWriter};
use smithay_client_toolkit::shm::MemPool;
use wayland_client::Main;

const TRANSPARENT: u32 = 0x00_00_00_00;

// A Pool is attached to whatever client with a surface
// The client requests a buffer and is given the type Buffer
// which holds a WlBuffer which references the Buffer in the MemPool
// along with it's offset and size.
//
// When a buffer has to be drawn in the MemPool, a Buffer is passed.
// The Pool checks if the Buffer fits and then draws the buf if it is.

// Points to where a buffer is set in the MemPool
#[derive(Clone, Debug)]
pub struct Buffer {
    pub wl_buffer: WlBuffer,
    offset: i32,
    width: i32,
    height: i32,
}

impl Buffer {
    fn new(offset: i32, width: i32, height: i32, wl_buffer: WlBuffer) -> Self {
        Buffer {
            wl_buffer,
            offset,
            width,
            height
        }
    }
    fn contains(&self, index: i32) -> bool {
        index < self.width * self.height
    }
    pub fn size(&self) -> i32 {
        self.width * self.height
    }
    pub fn attach(&self, surface: &Main<WlSurface>) {
        surface.attach(Some(&self.wl_buffer), 0, 0)
    }
}

pub struct Pool {
    width: i32,
    height: i32,
    stride: i32,
    format: Format,
    buffers: Vec<Buffer>,
    pub mempool: MemPool,
}

fn set_pixel(buffer: &mut [u8], mut start: u32, pixel: u32) {
    for byte in &pixel.to_ne_bytes() {
        buffer[start as usize] = *byte;
        start += 1;
    }
}

impl Geometry for Pool {
    fn get_width(&self) -> u32 {
        self.width as u32
    }
    fn get_height(&self) -> u32 {
        self.height as u32
    }
    fn contains(
        &mut self,
        _widget_x: u32,
        _widget_y: u32,
        _x: u32,
        _y: u32,
        _event: Input,
    ) -> Damage {
        Damage::None
    }
}

impl Pool {
    pub fn new(width: i32, height: i32, stride: i32, mut mempool: MemPool) -> Pool {
        mempool.resize((width*height*stride) as usize);
        Pool {
            format: Format::Argb8888,
            stride,
            width,
            buffers: Vec::new(),
            height,
            mempool,
        }
    }
    pub fn size(&self) -> i32 {
        self.width * self.height * self.stride
    }
    // Create a new Buffer from the MemPool
    pub fn new_buffer(&mut self, width: i32, height: i32) -> Buffer {
        let offset = if let Some(buf) = self.buffers.last() {
            buf.offset + buf.size()
        } else {
            0
        };
        let buffer = self.mempool.buffer(offset, width.abs(), height.abs(), width*4, self.format);
        Buffer::new(offset, width, height, buffer)
    }
    pub fn composite(&mut self, x: i32, y: i32, buffer: &Buffer, buf: &[u8]) -> Result<(), Error> {
        if buffer.size() <= self.width * self.height {
            let mut i = 0;
            let width = self.width * 4;
            let mut index = buffer.offset + (x + (y * self.width)) * 4;
            if buffer.contains(index) {
                let mut writer = BufWriter::new(&mut self.mempool);
                while (i+width as usize) < buf.len() {
                    writer.seek(SeekFrom::Start(index as u64)).unwrap();
                    writer.write_all(&buf[i..(i+width as usize) as usize]).unwrap();
                    writer.flush().unwrap();
                    index += width;
                    i += width as usize;
                }
            }
            Ok(())
        } else {
            Err(Error::Dimension("buffer", buffer.width as u32, buffer.height as u32))
        }
    }
    pub fn fill(&mut self) {
        let mut writer = BufWriter::new(&mut self.mempool);
        let pixel: u32 = 0xFF_D0_00_00;
        for _ in 0..(self.width*self.height) {
            writer.write_all(&pixel.to_ne_bytes()).unwrap();
        }
        writer.flush().unwrap();
    }
    pub fn debug(&self) {
        print!("width: {}\n", self.width);
        print!("height: {}\n", self.height);
    }
    pub fn resize(&mut self, width: i32, height: i32) {
        self.width = width;
        self.height = width;
        self.mempool.resize((width*height*4) as usize);
    }
}
