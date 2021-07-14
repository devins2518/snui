use crate::snui::*;
use smithay_client_toolkit::shm::{AutoMemPool, Format};
use std::io::{BufWriter, Write};
use wayland_client::protocol::{wl_buffer::WlBuffer, wl_surface::WlSurface};
use wayland_client::Main;

const TRANSPARENT: u32 = 0x00_00_00_00;

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
            Damage::Destroy => {
                for _ in 0..self.size() {
                    self.canvas.write_all(&TRANSPARENT.to_ne_bytes()).unwrap();
                }
                self.canvas.flush().unwrap();
            }
            _ => {}
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
        let buf_width = (self.width * 4) as usize;
        let width = surface.get_width() as usize * 4;
        let mut slice = if width > buf_width { buf_width } else { width };
        let mut index = ((x + (y * self.get_width())) * 4) as usize;
        while i < surface.size() && index < self.canvas.len() {
            if self.size() - index < buf_width {
                slice = self.size() - index;
            }
            let mut writer = BufWriter::new(&mut self.canvas[index..index + slice]);
            writer.write(&buf[i..i + slice]).unwrap();
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
