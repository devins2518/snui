pub mod button;
pub mod container;
pub mod image;
pub mod shapes;
pub mod slider;
pub mod text;

pub use crate::widgets::image::Image;
pub use container::layout::WidgetLayout;
use raqote::*;
pub use shapes::Shape;

pub fn u32_to_source(color: u32) -> SolidSource {
    let color = color.to_be_bytes();
    SolidSource {
        a: color[0],
        r: color[1],
        g: color[2],
        b: color[3],
    }
}

pub fn blend(pix_a: &[u8], pix_b: &[u8], t: f32) -> [u8; 4] {
    let (r_a, g_a, b_a, a_a) = (
        pix_a[1] as f32,
        pix_a[2] as f32,
        pix_a[3] as f32,
        pix_a[0] as f32,
    );
    let (r_b, g_b, b_b, a_b) = (
        pix_b[1] as f32,
        pix_b[2] as f32,
        pix_b[3] as f32,
        pix_b[0] as f32,
    );
    let red = blend_f32(r_a, r_b, t);
    let green = blend_f32(g_a, g_b, t);
    let blue = blend_f32(b_a, b_b, t);
    let alpha = blend_f32(a_a, a_b, t);
    [alpha as u8, red as u8, green as u8, blue as u8]
}

fn blend_f32(a: f32, b: f32, r: f32) -> f32 {
    a + ((b - a) * r)
}
