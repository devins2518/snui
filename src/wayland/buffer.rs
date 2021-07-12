use crate::snui::*;
use smithay_client_toolkit::shm::{AutoMemPool, Format};
use std::convert::TryInto;
use std::io::{Write, Seek, SeekFrom, BufWriter};
use wayland_client::protocol::{wl_buffer::WlBuffer, wl_surface::WlSurface};
use wayland_client::Main;

const TRANSPARENT: u32 = 0x00_00_00_00;

pub struct Buffer<'b> {
    width: u32,
    height: u32,
    wl_buffer: WlBuffer,
    canvas: &'b mut [u8],
}

fn set_pixel(buffer: &mut [u8], mut start: u32, pixel: u32) {
    for byte in &pixel.to_ne_bytes() {
        buffer[start as usize] = *byte;
        start += 1;
    }
}

impl<'b> Geometry for Buffer<'b> {
    fn get_width(&self) -> u32 {
        self.width
    }
    fn get_height(&self) -> u32 {
        self.height
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

impl<'b> Canvas for Buffer<'b> {
    fn damage(&mut self, event: Damage) {
        match event {
            Damage::All { surface } => {
                self.composite(&surface, 0, 0);
            }
            Damage::Area { surface, x, y } => {
                self.composite(&surface, x, y);
            }
            Damage::Destroy {
                x,
                y,
                width,
                height,
            } => {
                for x in x..x + width {
                    for y in y..y + height {
                        self.set(x, y, Content::Transparent);
                    }
                }
            }
            _ => {}
        }
    }
    fn get(&self, x: u32, y: u32) -> Content {
        let index = ((x + (y * self.get_width())) * 4) as usize;
        let buf = self.canvas[index..index + 4]
            .try_into()
            .expect("slice with incorrect length");
        let pixel = u32::from_ne_bytes(buf);
        Content::Pixel(pixel)
    }
    fn set(&mut self, x: u32, y: u32, content: Content) {
        let index = (x + (y * self.get_width())) * 4;
        if ((x * y) as usize) < self.canvas.len() {
            match content {
                Content::Pixel(p) => set_pixel(self.canvas, index, p),
                Content::Transparent => set_pixel(self.canvas, index, TRANSPARENT),
                _ => {}
            }
        }
    }
    fn get_buf(&self) -> &[u8] {
        &self.canvas
    }
    fn get_mut_buf(&mut self) -> &mut [u8] {
        &mut self.canvas
    }
    fn composite(&mut self, surface: &(impl Canvas + Geometry), x: u32, y: u32) {
        let mut i = 0;
        let buf = surface.get_buf();
        let width = surface.get_width() as usize * 4;
        let buf_width = (self.width * 4) as usize;
        let mut index = ((x + (y * surface.get_width())) * 4) as usize;
        while i < surface.size() && index < self.canvas.len() {
            let mut writer = BufWriter::new(&mut self.canvas[index..index+buf_width]);
            writer.write(&buf[i..i+width]).unwrap();
            writer.flush().unwrap();
            i += width;
            index += buf_width;
        }
    }
    fn size(&self) -> usize {
        (self.width * self.height * 4) as usize
    }
}

impl<'b> Buffer<'b> {
    pub fn new(width: i32, height: i32, stride: i32, mempool: &mut AutoMemPool) -> Buffer {
        let format = Format::Argb8888;
        let buffer = mempool.buffer(width, height, stride, format).unwrap();
        Buffer {
            width: width as u32,
            height: height as u32,
            wl_buffer: buffer.1,
            canvas: buffer.0,
        }
    }
    pub fn attach(&mut self, surface: &Main<WlSurface>, x: i32, y: i32) {
        surface.attach(Some(&self.wl_buffer), x, y);
    }
}
