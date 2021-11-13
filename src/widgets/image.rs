use crate::*;
use image::io::Reader as ImageReader;
use raqote::*;
use scene::Instruction;
use std::path::Path;
use std::sync::Arc;

#[derive(Clone, PartialEq)]
pub struct Image {
    image: Arc<[u8]>,
    width: u32,
    height: u32,
    radius: [f32; 4],
    size: (u32, u32),
}

const STROKE: StrokeStyle = StrokeStyle {
    width: 0.,
    cap: LineCap::Butt,
    join: LineJoin::Miter,
    miter_limit: 1.,
    dash_array: Vec::new(),
    dash_offset: 0.,
};

impl Image {
    pub fn new(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let dyn_image = ImageReader::open(path)?.decode()?.to_bgra8();

        let (width, height) = dyn_image.dimensions();
        let image: Arc<[u8]> = dyn_image.into_raw().into();

        Ok(Image {
            image,
            width,
            height,
            size: (width, height),
            radius: [0.; 4],
        })
    }
    pub fn new_with_size(
        path: &Path,
        width: u32,
        height: u32,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let dyn_image = ImageReader::open(path)?.decode()?.to_bgra8();

        let size = dyn_image.dimensions();
        let image: Arc<[u8]> = dyn_image.into_raw().into();

        Ok(Image {
            image,
            width,
            height,
            size,
            radius: [0.; 4],
        })
    }
}

impl Geometry for Image {
    fn width(&self) -> f32 {
        self.width as f32
    }
    fn height(&self) -> f32 {
        self.height as f32
    }
    fn set_size(&mut self, width: f32, height: f32) -> Result<(), (f32, f32)> {
        self.width = width.max(0.) as u32;
        self.height = height.max(0.) as u32;
        Ok(())
    }
}

impl Primitive for Image {
    fn draw(&self, x: f32, y: f32, ctx: &mut Context) {
        let p = self.image.as_ptr();
        let len = self.image.len();
        let data =
            unsafe { std::slice::from_raw_parts(p as *mut u32, len / std::mem::size_of::<u32>()) };
        let image = raqote::Image {
            width: self.size.0 as i32,
            height: self.size.1 as i32,
            data,
        };
        ctx.draw_image_with_size(x, y, image, self.width as f32, self.height as f32);

        /*
        // Creating the path
        let mut width = self.width();
        let mut height = self.height();
        let mut pb = PathBuilder::new();

        let mut cursor = (0., 0.);

        // Sides length
        let top = width - self.radius[0] - self.radius[1];
        let right = height - self.radius[1] - self.radius[2];
        let left = height - self.radius[0] - self.radius[3];
        let bottom = width - self.radius[2] - self.radius[3];

        // Positioning the cursor
        cursor.0 += self.radius[0];
        cursor.1 += self.radius[0];

        // Drawing the outline
        pb.arc(cursor.0, cursor.1, self.radius[0], PI, PI / 2.);
        cursor.0 += top;
        cursor.1 -= self.radius[0];
        pb.line_to(cursor.0, cursor.1);
        cursor.1 += self.radius[1];
        pb.arc(cursor.0, cursor.1, self.radius[1], -PI / 2., PI / 2.);
        cursor.0 += self.radius[1];
        cursor.1 += right;
        pb.line_to(cursor.0, cursor.1);
        cursor.0 -= self.radius[2];
        pb.arc(cursor.0, cursor.1, self.radius[2], 0., PI / 2.);
        cursor.1 += self.radius[2];
        cursor.0 -= bottom;
        pb.line_to(cursor.0, cursor.1);
        cursor.1 -= self.radius[3];
        pb.arc(cursor.0, cursor.1, self.radius[3], PI / 2., PI / 2.);
        cursor.0 -= self.radius[3];
        cursor.1 -= left;
        pb.line_to(cursor.0, cursor.1);

        // Closing path
        pb.close();
        let path = pb.finish();
        match &mut ctx.backend {
            Backend::Raqote(dt) => {
                dt.stroke(
                    &path,
                    &Source::Image(
                        image,
                        ExtendMode::Pad,
                        FilterMode::Bilinear,
                        Transform::create_translation(-x, -y).post_scale(image.width as f32 / width, image.height as f32 / height)
                    ),
                    &STROKE,
                    &DrawOptions {
                        blend_mode: BlendMode::SrcAtop,
                        alpha: 1.,
                        antialias: AntialiasMode::Gray,
                    },
                )
            }
            _ => {}
        }

        ctx.draw_image(x, y, image);
        */
    }
}

impl Widget for Image {
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        RenderNode::Instruction(Instruction::new(x, y, self.clone()))
    }
    fn sync<'d>(&'d mut self, ctx: &mut Context, event: Event) {}
}

impl std::fmt::Debug for Image {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Image")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("size", &self.image.len())
            .finish()
    }
}
