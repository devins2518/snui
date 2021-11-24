use crate::*;
use crate::widgets::*;
use super::layout::WidgetLayout;
use std::ops::{Deref, DerefMut};
use scene::{Coords, RenderNode};

pub struct BoxLayout {
    width: f32,
    height: f32,
    layout: WidgetLayout,
    anchor: (Alignment, Alignment),
}

impl Geometry for BoxLayout {
    fn width(&self) -> f32 {
        self.width
    }
    fn height(&self) -> f32 {
        self.height
    }
    fn set_size(&mut self, width: f32, height: f32) -> Result<(), (f32, f32)> {
        self.width = self.layout.width().max(width);
        self.height = self.layout.height().max(height);

        if self.width == width && self.height == height {
            return Ok(())
        }
        Err((self.width, self.height))
    }
}

impl Container for BoxLayout {
    fn len(&self) -> usize {
        self.layout.len()
    }
    fn add(&mut self, widget: impl Widget + 'static) {
        match self.layout.orientation() {
            Orientation::Horizontal => {
                self.layout.add(
                    widget.anchor(
                        self.anchor,
                        self.width as u32 / self.len().max(1) as u32,
                        self.height as u32
                    )
                )
            }
            Orientation::Vertical => {
                self.layout.add(
                    widget.anchor(
                        self.anchor,
                        self.width as u32,
                        self.height as u32 / self.len().max(1) as u32
                    )
                )
            }
        }
    }
}

impl Widget for BoxLayout {
    fn create_node(&mut self, x: f32, y: f32) -> RenderNode {
        self.layout.create_node(x, y)
    }
    fn sync<'d>(&'d mut self, ctx: &mut SyncContext, event: Event) {
        self.layout.sync(ctx, event);
    }
}

impl BoxLayout {
    pub fn default(width: u32, height: u32, spacing: u32) -> Self {
        Self {
            width: width as f32,
            height: height as f32,
            anchor: (Alignment::Center, Alignment::Center),
            layout: WidgetLayout::horizontal(spacing)
        }
    }
}

impl Deref for BoxLayout {
    type Target = WidgetLayout;
    fn deref(&self) -> &Self::Target {
        &self.layout
    }
}

impl DerefMut for BoxLayout {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.layout
    }
}
