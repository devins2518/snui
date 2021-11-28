use crate::*;
use context::DrawContext;
use raqote::*;
use widgets::blend;
use widgets::shapes::*;
use widgets::text::*;
use widgets::Image;

#[derive(Clone, Copy, Debug, PartialEq)]
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

impl Coords {
    pub fn new(x: f32, y: f32) -> Coords {
        Coords { x, y }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Background {
    Transparent,
    Image(Image),
    Composite {
        coords: Coords,
        base: Box<Background>,
        overlay: Box<Background>,
    },
    Color(SolidSource),
}

impl From<Style> for Background {
    fn from(style: Style) -> Self {
        match style {
            Style::Background(bg) => bg,
            Style::Border(_, _) => Background::Transparent,
        }
    }
}

impl From<u32> for Background {
    fn from(color: u32) -> Self {
        Background::Color(widgets::u32_to_source(color))
    }
}

impl Background {
    pub fn solid(color: u32) -> Background {
        Background::Color(widgets::u32_to_source(color))
    }
    pub fn image(path: &std::path::Path) -> Background {
        Background::Image(Image::new(path).unwrap())
    }
    fn from(instruction: &Instruction) -> Self {
        match &instruction.primitive {
            PrimitiveType::Rectangle(r) => r.style.background(),
            PrimitiveType::Image(image) => Background::Composite {
                coords: instruction.coords,
                overlay: Box::new(Background::Transparent),
                base: Box::new(Background::Image(image.clone())),
            },
            _ => Background::Transparent,
        }
    }
    fn merge<C: Into<Coords>>(&self, other: Self, local: C) -> Self {
        match self {
            Background::Color(asource) => match other {
                Background::Color(bsource) => {
                    if bsource.a == 255 {
                        return other;
                    }
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
                Background::Transparent => self.clone(),
                Background::Image(_) => Background::Composite {
                    coords: local.into(),
                    base: Box::new(self.clone()),
                    overlay: Box::new(other),
                },
                Background::Composite {
                    coords,
                    base: _,
                    overlay: _,
                } => Background::Composite {
                    coords: coords.clone(),
                    base: Box::new(self.clone()),
                    overlay: Box::new(other),
                },
            },
            Background::Image(_) => {
                if let Background::Color(source) = other {
                    if source.a == 255 {
                        return other;
                    }
                }
                Background::Composite {
                    coords: local.into(),
                    base: Box::new(self.clone()),
                    overlay: Box::new(other),
                }
            }
            Background::Composite {
                coords,
                base,
                overlay,
            } => Background::Composite {
                coords: coords.clone(),
                base: base.clone(),
                overlay: Box::new(overlay.as_ref().merge(other, local)),
            },
            Background::Transparent => {
                if let Background::Image(_) = other {
                    return Background::Composite {
                        coords: local.into(),
                        base: Box::new(other),
                        overlay: Box::new(Background::Transparent),
                    };
                }
                other
            }
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, PartialEq)]
pub enum RenderNode {
    // Add a region to None
    None,
    Instruction(Instruction),
    Extension {
        background: Instruction,
        node: Box<(RenderNode, RenderNode)>,
    },
    Container(Vec<RenderNode>),
}

impl Instruction {
    fn render(&self, ctx: &mut DrawContext) {
        let x = self.coords.x;
        let y = self.coords.y;
        match &self.primitive {
            PrimitiveType::Image(i) => i.draw(x, y, ctx),
            PrimitiveType::Rectangle(r) => r.draw(x, y, ctx),
            PrimitiveType::Label(l) => ctx.draw_label(l, x, y),
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
    pub fn render(&self, ctx: &mut DrawContext) {
        match self {
            Self::Instruction(instruction) => instruction.render(ctx),
            Self::Container(c) => {
                for n in c {
                    n.render(ctx);
                }
            }
            Self::Extension { background, node } => {
                background.render(ctx);
                let (child, border) = node.as_ref();
                child.render(ctx);
                border.render(ctx);
            }
            _ => {}
        }
    }
    fn clear(&self, ctx: &mut DrawContext, bg: &Background) {
        match self {
            RenderNode::Instruction(instruction) => {
                ctx.damage_region(bg, &instruction.region());
            }
            RenderNode::Extension {
                background,
                node: _,
            } => {
                ctx.damage_region(bg, &background.region());
            }
            RenderNode::Container(nodes) => {
                for node in nodes {
                    node.clear(ctx, bg)
                }
            }
            RenderNode::None => {}
        }
    }

    /*
     * Renders to the DrawContext where the RenderNode differs
     */
    pub fn invalidate<'r>(&'r self, other: &'r Self, ctx: &mut DrawContext, bg: &Background) {
        match self {
            RenderNode::Instruction(a) => match other {
                RenderNode::Instruction(b) => {
                    if a.ne(b) {
                        ctx.damage_region(bg, &a.region());
                        b.render(ctx);
                    }
                }
                RenderNode::None => {}
                _ => {
                    ctx.damage_region(bg, &a.region());
                    other.render(ctx);
                }
            },
            RenderNode::Container(sv) => match other {
                RenderNode::Container(ov) => {
                    if sv.len() != ov.len() {
                        self.clear(ctx, bg);
                        other.render(ctx);
                    } else {
                        for i in 0..ov.len().min(sv.len()) {
                            sv[i].invalidate(&ov[i], ctx, bg);
                        }
                    }
                }
                RenderNode::None => {}
                _ => {
                    self.clear(ctx, bg);
                    other.render(ctx);
                }
            },
            RenderNode::Extension { background, node } => {
                let this_background = background;
                let (this_child, this_border) = node.as_ref();
                if let RenderNode::Extension { background, node } = other {
                    let (other_child, other_border) = node.as_ref();
                    if this_background == background && this_border == other_border {
                        this_child.invalidate(
                            other_child,
                            ctx,
                            &bg.merge(Background::from(this_background), this_background.region()),
                        );
                    } else {
                        ctx.damage_region(bg, &this_background.region());
                        other.render(ctx);
                    }
                } else {
                    ctx.damage_region(bg, &this_background.region());
                    other.render(ctx);
                }
            }
            RenderNode::None => {
                other.clear(ctx, bg);
                other.render(ctx)
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

impl From<Region> for Coords {
    fn from(r: Region) -> Self {
        Coords::new(r.x, r.y)
    }
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
        Region::new(
            self.x.max(other.x),
            self.y.max(other.y),
            self.width.min(other.width),
            self.height.min(other.height),
        )
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
