use crate::*;
use context::Context;
use raqote::*;
use std::cmp::Ordering;
use widgets::blend;
use widgets::font::text::*;
use widgets::shapes::*;
use widgets::Image;

#[derive(Clone, Debug, PartialEq)]
pub struct Coords {
    pub x: f32,
    pub y: f32,
}

impl From<(f32, f32)> for Coords {
    fn from(coords: (f32, f32)) -> Self {
        Coords {
            x: coords.0,
            y: coords.1,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Background {
    Transparent,
    Color(SolidSource),
}

impl Background {
    pub fn from(instruction: &Instruction) -> Self {
        match instruction.primitive {
            PrimitiveType::Rectangle(r) => Background::Color(r.style.source()),
            _ => Background::Transparent,
        }
    }
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
            _ => self.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum PrimitiveType {
    Label(Label),
    Image(Image),
    Rectangle(Rectangle),
}

impl From<Rectangle> for PrimitiveType {
    fn from(r: Rectangle) -> Self {
        PrimitiveType::Rectangle(r)
    }
}

impl From<Label> for PrimitiveType {
    fn from(l: Label) -> Self {
        PrimitiveType::Label(l)
    }
}

impl From<Image> for PrimitiveType {
    fn from(i: Image) -> Self {
        PrimitiveType::Image(i)
    }
}

#[derive(Debug)]
pub struct Instruction {
    coords: Coords,
    primitive: PrimitiveType,
}

impl Instruction {
    pub fn new<P: Into<PrimitiveType>>(x: f32, y: f32, primitive: P) -> Instruction {
        Instruction {
            coords: Coords::new(x, y),
            primitive: primitive.into(),
        }
    }
}

#[derive(Debug)]
pub enum RenderNode {
    Instruction(Instruction),
    Extension {
        background: Instruction,
        node: Box<RenderNode>,
    },
    Container(Vec<RenderNode>),
}

impl Coords {
    pub fn new(x: f32, y: f32) -> Coords {
        Coords { x, y }
    }
}

impl Instruction {
    fn render(&self, ctx: &mut Context) {
        let x = self.coords.x;
        let y = self.coords.y;
        match &self.primitive {
            PrimitiveType::Image(i) => i.draw(x, y, ctx),
            PrimitiveType::Rectangle(r) => r.draw(x, y, ctx),
            PrimitiveType::Label(l) => l.draw(x, y, ctx),
        }
    }
    fn region(&self) -> Region {
        Region::new(
            self.coords.x,
            self.coords.y,
            match &self.primitive {
                PrimitiveType::Image(i) => i.width(),
                PrimitiveType::Rectangle(r) => r.width(),
                PrimitiveType::Label(l) => l.width(),
            },
            match &self.primitive {
                PrimitiveType::Image(i) => i.height(),
                PrimitiveType::Rectangle(r) => r.height(),
                PrimitiveType::Label(l) => l.height(),
            },
        )
    }
}

impl PartialEq for Instruction {
    fn eq(&self, other: &Self) -> bool {
        self.coords.eq(&other.coords) && self.primitive.eq(&other.primitive)
    }
    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl RenderNode {
    pub fn render(&self, ctx: &mut Context) {
        match self {
            Self::Instruction(instruction) => instruction.render(ctx),
            Self::Container(c) => {
                for n in c {
                    n.render(ctx);
                }
            }
            Self::Extension { background, node } => {
                background.render(ctx);
                node.render(ctx);
            }
        }
    }
    pub fn find_diff<'r>(&'r self, other: &'r Self, ctx: &mut Context, bg: &Background) {
        match self {
            RenderNode::Instruction(a) => match other {
                RenderNode::Instruction(b) => {
                    if a.ne(b) {
                        ctx.damage_region(bg, &a.region());
                        b.render(ctx);
                    }
                }
                _ => {
                    ctx.damage_region(bg, &a.region());
                    other.render(ctx);
                }
            },
            RenderNode::Container(sv) => match other {
                RenderNode::Container(ov) => {
                    if sv.len() != ov.len() {
                        other.render(ctx);
                    } else {
                        for i in 0..ov.len().min(sv.len()) {
                            sv[i].find_diff(&ov[i], ctx, bg);
                        }
                    }
                }
                _ => other.render(ctx),
            },
            RenderNode::Extension { background, node } => {
                let this_node = node;
                let this_background = background;
                if let RenderNode::Extension { background, node } = other {
                    if this_background == background {
                        this_node.find_diff(
                            node,
                            ctx,
                            &bg.merge(Background::from(this_background)),
                        );
                    } else {
                        ctx.damage_region(
                            &Background::from(this_background),
                            &this_background.region(),
                        );
                        other.render(ctx);
                    }
                } else {
                    ctx.damage_region(
                        &Background::from(this_background),
                        &this_background.region(),
                    );
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
