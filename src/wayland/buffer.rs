use crate::snui::*;
use crate::wayland::*;
use crate::widgets::*;
use smithay_client_toolkit::shm::{AutoMemPool, Format};
use std::convert::TryInto;
use wayland_client::protocol::{
    wl_buffer, wl_buffer::WlBuffer, wl_output::WlOutput, wl_surface::WlSurface,
};
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

impl<'b> Canvas for Buffer<'b> {
    fn paint(&self) {}
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
                    for x in y..y + height {
                        self.set(x, y, Content::Transparent);
                    }
                }
            }
            Damage::None => {}
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
    fn composite(&mut self, surface: &(impl Canvas + Drawable), x: u32, y: u32) {
        let width = if x + surface.get_width() <= self.width {
            surface.get_width()
        } else if self.width > x {
            self.width - x
        } else {
            0
        };
        let height = if y + surface.get_height() <= self.height {
            surface.get_height()
        } else if self.height > y {
            self.height - y
        } else {
            0
        };
        for dx in 0..width {
            for dy in 0..height {
                let content = surface.get(dx, dy);
                self.set(dx, dy, content);
            }
        }
    }
}

impl<'b> Drawable for Buffer<'b> {
    fn set_content(&mut self, content: Content) {
        for x in 0..self.width {
            for y in 0..self.height {
                self.set(x, y, content);
            }
        }
    }
    fn get_width(&self) -> u32 {
        self.width
    }
    fn get_height(&self) -> u32 {
        self.height
    }
    fn draw(&self, canvas: &mut Surface, x: u32, y: u32) {}
    fn contains(&mut self, x: u32, y: u32, event: Input) -> bool {
        true
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
