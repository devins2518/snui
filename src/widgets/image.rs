use crate::*;
use image::imageops::{self, FilterType};
use image::io::Reader as ImageReader;
use image::{Bgra, ImageBuffer};
use std::error::Error;
use std::io::{BufWriter, Write};
use std::path::Path;

#[derive(Clone)]
pub struct Image {
    image: ImageBuffer<Bgra<u8>, Vec<u8>>,
}

impl Image {
    pub fn new(image: &Path) -> Result<Self, Box<dyn Error>> {
        let dyn_image = ImageReader::open(image)?.decode()?;

        let image = dyn_image.to_bgra8();

        Ok(Self { image })
    }

    pub fn new_with_size(image: &Path, width: u32, height: u32) -> Result<Self, Box<dyn Error>> {
        let dyn_image = ImageReader::open(image)?.decode()?;
        let scaled_image = dyn_image.resize_to_fill(width, height, FilterType::Triangle);

        let image = scaled_image.to_bgra8();

        Ok(Self { image })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.image = imageops::resize(&self.image, width, height, FilterType::Triangle);
    }

    pub fn thumbnail(&self, width: u32, height: u32) -> Image {
        Image {
            image: imageops::thumbnail(&self.image, width, height),
        }
    }
    pub fn size(&self) -> usize {
        (self.image.width() * self.image.height() * 4) as usize
    }
    pub fn render(&self, canvas: &mut [u8], mut width: usize, x: u32, y: u32) {
        let mut i = 0;
        let image_buf = self.image.as_raw();
        let img_width = (self.get_width() * 4) as usize;
        let mut index = ((x + (y * width as u32)) * 4) as usize;
        width *= 4;
        while i < image_buf.len() && index < canvas.len() {
            let slice = if canvas.len() - index < width {
                canvas.len() - index
            } else {
                img_width
            };
            let mut writer = BufWriter::new(&mut canvas[index..index + slice]);
            writer.write(&image_buf[i..i + img_width]).unwrap();
            writer.flush().unwrap();
            i += img_width;
            index += width;
        }
    }
}

impl Geometry for Image {
    fn get_width(&self) -> u32 {
        self.image.width()
    }
    fn get_height(&self) -> u32 {
        self.image.height()
    }
    // TODO
    fn contains<'d>(
        &'d mut self,
        _widget_x: u32,
        _widget_y: u32,
        _x: u32,
        _y: u32,
        _event: Input,
    ) -> Damage<'d> {
        Damage::None
    }
}

impl Drawable for Image {
    fn set_content(&mut self, content: Content) {
        match content {
            Content::Empty => {
                self.image =
                    ImageBuffer::from_pixel(self.get_width(), self.get_height(), Bgra([0, 0, 0, 0]))
            }
            Content::Transparent => {
                for pixel in self.image.pixels_mut() {
                    pixel.0[3] = 0;
                }
            }
            Content::Pixel(pixel) => {
                let arr = pixel.to_ne_bytes();
                self.image = ImageBuffer::from_pixel(self.get_width(), self.get_height(), Bgra(arr))
            }
            _ => eprintln!("Attempted to perform illegal operation on image!"),
        }
    }

    fn draw(&self, canvas: &mut [u8], width: u32, x: u32, y: u32) {
        self.render(canvas, width as usize, x, y);
    }
}

impl Widget for Image {}
