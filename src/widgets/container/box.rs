use crate::*;
use crate::widgets::*;
use layout::WidgetLayout;
use scene::{Coords, RenderNode};

pub struct BoxLayout {
    size: (f32, f32),
    layout: WidgetLayout,
    anchor: (Alignment, Alignment),
}
