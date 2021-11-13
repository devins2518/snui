use crate::*;
use context::Context;
use raqote::*;
use std::cmp::Ordering;
use widgets::blend;

#[derive(Clone, Debug, PartialEq)]
pub struct Coords {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Background {
    Transparent,
    Color(SolidSource),
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
            _ => self.clone(),
        }
    }
}

#[derive(Debug)]
pub struct Instruction {
    coords: Coords,
    primitive: Box<dyn Primitive>,
}

impl Instruction {
    pub fn new(x: f32, y: f32, primitive: impl Primitive + 'static) -> Instruction {
        Instruction {
            coords: Coords::new(x, y),
            primitive: Box::new(primitive),
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
        self.primitive.draw(x, y, ctx);
    }
    fn region(&self) -> Region {
        Region::new(
            self.coords.x,
            self.coords.y,
            self.primitive.width(),
            self.primitive.height(),
        )
    }
}

impl PartialEq for Instruction {
    fn eq(&self, other: &Self) -> bool {
        self.coords.eq(&other.coords) && self.primitive.same(&other.primitive)
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
            Self::Extension {
                background,
                node,
            } => {
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
            RenderNode::Container(sv) => {
                match other {
                    RenderNode::Container(ov) => {
                        if sv.len() != ov.len() {
                            other.render(ctx);
                        } else {
                            for i in 0..ov.len().min(sv.len()) {
                                sv[i].find_diff(&ov[i], ctx, bg);
                            }
                        }
                    }
                    _ => {
                        // ctx.damage_region(bg, &region);
                        other.render(ctx)
                    }
                }
            }
            RenderNode::Extension {
                background,
                node,
            } => {
                let this_node = node;
                let this_background = background;
                if let RenderNode::Extension {
                    background,
                    node,
                } = other
                {
                    if this_background == background {
                        this_node.find_diff(
                            node,
                            ctx,
                            &bg.merge(this_background.primitive.to_background()),
                        );
                    } else {
                        ctx.damage_region(
                            &this_background.primitive.to_background(),
                            &this_background.region(),
                        );
                        other.render(ctx);
                    }
                } else {
                    ctx.damage_region(
                        &this_background.primitive.to_background(),
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
