use widgets::primitives::*;
use widgets::blend;
use widgets::text::LabelData;
use context::Context;
use std::cmp::Ordering;
use crate::*;
use raqote::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Coords (f32, f32);

#[derive(Clone, Debug, PartialEq)]
enum Instruction {
    Text(LabelData),
    Rectangle(shapes::Rectangle),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Background {
    Transparent,
    Color(SolidSource)
}

impl Background {
    pub fn merge(&self, other: Self) -> Self {
        match other {
            Background::Color(bsource) => match self {
                Background::Color(asource) => {
                    let source = blend(
                        &asource.to_u32().to_be_bytes(),
                        &bsource.to_u32().to_be_bytes(),
                        1.,
                    );
                    Background::Color(SolidSource {
                        a: source[0],
                        r: source[1],
                        g: source[2],
                        b: source[3],
                    })
                }
                Background::Transparent => other,
            },
            _ => self.clone()
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Damage {
    region: Region,
    instruction: Instruction,
}

impl Damage {
    pub fn from_text(region: Region, data: LabelData) -> Damage {
        Damage {
            region,
            instruction: Instruction::Text(data)
        }
    }
    pub fn from_rectangle(x: f32, y: f32, rectangle: shapes::Rectangle) -> Damage {
        Damage {
            region: Region::new(x, y, rectangle.width(), rectangle.height()),
            instruction: Instruction::Rectangle(rectangle)
        }
    }
}

#[derive(Clone, Debug)]
pub enum RenderNode {
    Widget(Damage),
    Extension {
        border: Damage,
        background: Damage,
        node: Box<RenderNode>,
    },
    Container(Region, Vec<RenderNode>)
}

impl Coords {
    pub fn new(x: f32, y: f32) -> Coords {
        Coords (x, y)
    }
}

impl Damage {
    fn render(&self, ctx: &mut Context) {
        let x = self.region.x;
        let y = self.region.y;
        match &self.instruction {
            Instruction::Rectangle(rectangle) => {
                rectangle.draw(ctx, x, y);
            }
            Instruction::Text(data) => {
                ctx.draw_label(x, y, data);
            }
            _ => {}
        }
    }
    fn into_background(&self) -> Background {
        match &self.instruction {
            Instruction::Rectangle(r) => {
                match r.style {
                    Style::Fill(source) => Background::Color(source),
                    _ => Background::Transparent
                }
            }
            _ => Background::Transparent
        }
    }
}

impl Geometry for RenderNode {
    fn width(&self) -> f32 {
        match self {
            RenderNode::Widget(damage) => damage.region.width,
            RenderNode::Extension { border:_, background, node:_ } => background.region.width,
            RenderNode::Container(region, _) => region.width
        }
    }
    fn height(&self) -> f32 {
        match self {
            RenderNode::Widget(damage) => damage.region.height,
            RenderNode::Extension { border:_, background, node:_ } => background.region.height,
            RenderNode::Container(region, _) => region.height
        }
    }
}

impl RenderNode {
    fn render(&self, ctx: &mut Context) {
        match self {
            Self::Widget(d) => d.render(ctx),
            Self::Container(_, c) => for d in c {
                d.render(ctx);
            }
            Self::Extension { border, background, node } => {
                background.render(ctx);
                border.render(ctx);
                node.render(ctx);
            }
        }
    }
    pub fn find_diff<'r>(&'r self, other: &'r Self, ctx: &mut Context, bg: &Background) {
        match self {
            RenderNode::Widget(a) => {
                match other {
                    RenderNode::Widget(b) => {
                        if a != b {
                            ctx.damage_region(bg, &a.region);
                            b.render(ctx);
                        }
                    }
                    _ => {
                        ctx.damage_region(bg, &a.region);
                        other.render(ctx);
                    }
                }
            }
            RenderNode::Container(region, sv) => {
                match other {
                    RenderNode::Container(_, ov) => {
                        if sv.len() != ov.len() {
                            other.render(ctx);
                        } else {
                            for i in 0..ov.len().min(sv.len()) {
                                sv[i].find_diff(&ov[i], ctx, bg);
                            }
                        }
                    }
                    _ => {
                        ctx.damage_region(bg, &region);
                        other.render(ctx)
                    }
                }
            }
            RenderNode::Extension { border, background, node } => {
                let this_node  = node ;
                let this_border = border;
                let this_background = background;
                if let RenderNode::Extension {background, border, node} = other {
                    if this_border == border && this_background == background {
                        this_node.find_diff(node, ctx, &bg.merge(background.into_background()));
                    } else {
                        ctx.damage_region(&this_background.into_background(), &this_background.region);
                        other.render(ctx);
                    }
                } else {
                    ctx.damage_region(&this_background.into_background(), &this_background.region);
                    other.render(ctx);
                }
            }
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Region {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Region {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Region {
        Region {
            x,
            y,
            width,
            height,
        }
    }
    pub fn same(&self, other: &Self) -> bool {
        self.x == other.x
        && self.y == other.y
        && self.width == other.width
        && self.height == other.height
    }
    pub fn crop_region(&self, other: &Self) -> Region {
        let max = self.max(other);
        let min = self.min(other);

        Region::new(max.x - min.x, max.y - min.y, min.width, min.height)
    }
    pub fn merge(&mut self, other: &Self) {
        if self.contains(other.x, other.y) {
            self.width = self.x.max(other.x) + self.width.max(other.width);
            self.height = self.y.max(other.y) + self.height.max(other.height);
        }
    }
    pub fn contains(&self, x: f32, y: f32) -> bool {
        self.x < x && x - self.x <= self.width && self.y < y && y - self.y <= self.height
    }
    pub fn pad(&self, padding: f32) -> Region {
        Self {
            x: self.x - padding,
            y: self.y - padding,
            width: self.width + 2. * padding,
            height: self.height + 2. * padding,
        }
    }
}

impl PartialEq for Region {
    fn eq(&self, other: &Self) -> bool {
        other.x - self.x + other.width <= self.width
            && other.y - self.y + other.height <= self.height
    }
    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl PartialOrd for Region {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.x > other.x + other.width || self.y > other.y + other.height {
            Some(Ordering::Greater)
        } else if self.x + self.width < other.x || self.y + self.height < other.y {
            Some(Ordering::Less)
        } else {
            Some(Ordering::Equal)
        }
    }
}

impl Eq for Region {}

impl Ord for Region {
    fn cmp(&self, other: &Self) -> Ordering {
        if self < other {
            Ordering::Less
        } else if self > other {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}
