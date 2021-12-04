use crate::*;
use context::DrawContext;
use std::rc::Rc;
pub use tiny_skia::*;
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

impl From<&Coords> for Point {
    fn from(coords: &Coords) -> Self {
        Point::from_xy(coords.x, coords.y)
    }
}

impl Into<Point> for Coords {
    fn into(self) -> Point {
        Point::from_xy(self.x, self.y)
    }
}

impl Coords {
    pub fn new(x: f32, y: f32) -> Coords {
        Coords { x, y }
    }
}

#[derive(Debug, Clone)]
pub enum Background {
    Transparent,
    Image(Coords, Image),
    LinearGradient {
        start: Coords,
        end: Coords,
        angle: f32,
        mode: SpreadMode,
        stops: Rc<[GradientStop]>,
    },
    Composite(Box<Background>, Box<Background>),
    Color(Color),
}

impl PartialEq for Background {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Transparent => {
                if let Self::Transparent = other {
                    return true;
                }
            }
            Self::Color(sc) => {
                if let Self::Color(oc) = other {
                    return sc == oc;
                }
            }
            Self::Image(sc, si) => {
                if let Self::Image(oc, oi) = other {
                    return sc == oc && si.eq(&oi);
                }
            }
            Self::LinearGradient {
                start,
                end,
                angle: _,
                stops,
                mode,
            } => {
                let ss = start;
                let se = end;
                let sg = stops;
                let sm = mode;
                if let Self::LinearGradient {
                    start,
                    end,
                    angle: _,
                    stops,
                    mode,
                } = other
                {
                    return ss == start
                        && se == end
                        && Rc::as_ptr(sg) == Rc::as_ptr(stops)
                        && sm == mode;
                }
            }
            Self::Composite(sb, so) => {
                if let Self::Composite(ob, oo) = other {
                    return sb as *const Box<Background> == ob as *const Box<Background>
                        && so as *const Box<Background> == oo as *const Box<Background>;
                }
            }
        }
        false
    }
}

impl From<ShapeStyle> for Background {
    fn from(style: ShapeStyle) -> Self {
        match style {
            ShapeStyle::Background(bg) => bg,
            ShapeStyle::Border(_, _) => Background::Transparent,
        }
    }
}

impl From<u32> for Background {
    fn from(color: u32) -> Self {
        Background::Color(widgets::u32_to_source(color))
    }
}

impl From<Color> for Background {
    fn from(color: Color) -> Self {
        Background::Color(color)
    }
}

impl From<ColorU8> for Background {
    fn from(color: ColorU8) -> Self {
        color.get().into()
    }
}

impl From<Image> for Background {
    fn from(image: Image) -> Self {
        Background::Image(Coords::new(0., 0.), image)
    }
}

impl Background {
    pub fn solid(color: u32) -> Background {
        Background::Color(widgets::u32_to_source(color))
    }
    pub fn image(path: &std::path::Path) -> Background {
        Background::Image(Coords::new(0., 0.), Image::new(path).unwrap())
    }
    /*
     * The angle is a radiant representing the tild of the gradient clock wise.
     */
    pub fn linear_gradient(stops: Vec<GradientStop>, mode: SpreadMode, angle: f32) -> Background {
        let stops: Rc<[GradientStop]> = stops.into();
        Background::LinearGradient {
            angle,
            start: Coords::new(0., 0.),
            end: Coords::new(0., 0.),
            mode,
            stops,
        }
    }
    fn from(instruction: &Instruction) -> Self {
        match &instruction.primitive {
            PrimitiveType::Rectangle(r) => r.get_style().background(),
            PrimitiveType::Image(image) => Background::Image(instruction.coords, image.clone()),
            _ => Background::Transparent,
        }
    }
    fn merge(&self, other: Self) -> Self {
        match self {
            Background::Color(acolor) => match other {
                Background::Color(bcolor) => {
                    if bcolor.is_opaque() {
                        return other;
                    }
                    Background::Color(blend(acolor, &bcolor, 1.))
                }
                Background::Image(_, _) => {
                    match other {
                        Background::Color(color) => {
                            if color.is_opaque() {
                                return other;
                            }
                        }
                        _ => {}
                    }
                    Background::Composite(Box::new(self.clone()), Box::new(other))
                }
                Background::Transparent => self.clone(),
                _ => Background::Composite(Box::new(self.clone()), Box::new(other)),
            },
            Background::LinearGradient {
                start: _,
                end: _,
                stops: _,
                mode: _,
                angle: _,
            } => match other {
                Background::Color(color) => {
                    if color.is_opaque() {
                        return other;
                    } else {
                        Background::Composite(Box::new(self.clone()), Box::new(other))
                    }
                }
                Background::Transparent => return self.clone(),
                _ => Background::Composite(Box::new(self.clone()), Box::new(other)),
            },
            Background::Image(_, _) => match other {
                Background::Color(color) => {
                    if color.is_opaque() {
                        return other;
                    } else {
                        Background::Composite(Box::new(self.clone()), Box::new(other))
                    }
                }
                Background::Transparent => return self.clone(),
                _ => Background::Composite(Box::new(self.clone()), Box::new(other)),
            },
            Background::Composite(_, overlay) => Background::Composite(
                Box::new(self.clone()),
                Box::new(overlay.as_ref().merge(other)),
            ),
            Background::Transparent => other,
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
    pub fn empty(x: f32, y: f32, width: f32, height: f32) -> RenderNode {
        RenderNode::Instruction(Instruction {
            coords: Coords::new(x, y),
            primitive: Rectangle::empty(width, height).into(),
        })
    }
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
                            &bg.merge(Background::from(this_background)),
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

impl From<&Region> for Rect {
    fn from(r: &Region) -> Self {
        Rect::from_xywh(r.x, r.y, r.width, r.height).unwrap()
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
    pub fn from_coords(start: &Coords, end: &Coords) -> Self {
        let x = start.x.min(end.x);
        let y = start.y.min(end.y);
        Region {
            x,
            y,
            width: start.x.max(end.x) - x,
            height: start.y.max(end.y) - y,
        }
    }
    pub fn same(&self, other: &Self) -> bool {
        self.x == other.x
            && self.y == other.y
            && self.width == other.width
            && self.height == other.height
    }
    pub fn crop(&self, other: &Self) -> Region {
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
