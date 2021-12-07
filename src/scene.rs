use crate::*;
use context::DrawContext;
use std::mem;
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
            PrimitiveType::Image(image) => {
                let coords = Coords::new(instruction.transform.tx, instruction.transform.ty);
                Background::Image(coords, image.clone())
            }
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

#[derive(Debug)]
pub enum PrimitiveType {
    Label(Label),
    Image(Image),
    Rectangle(Rectangle),
    Other {
        name: &'static str,
        id: u64,
        primitive: Box<dyn Primitive>,
    },
}

impl PartialEq for PrimitiveType {
    fn eq(&self, other: &Self) -> bool {
        match self {
            PrimitiveType::Rectangle(s) => {
                if let PrimitiveType::Rectangle(o) = other {
                    return s.eq(o);
                }
            }
            PrimitiveType::Label(s) => {
                if let PrimitiveType::Label(o) = other {
                    return s.eq(o);
                }
            }
            PrimitiveType::Image(s) => {
                if let PrimitiveType::Image(o) = other {
                    return s.eq(o);
                }
            }
            PrimitiveType::Other {
                name,
                id,
                primitive: _,
            } => {
                let sn = name;
                let sid = id;
                if let PrimitiveType::Other {
                    id,
                    name,
                    primitive: _,
                } = other
                {
                    return sn == name && id == sid;
                }
            }
        }
        false
    }
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
    transform: Transform,
    primitive: PrimitiveType,
}

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
impl Instruction {
    pub fn other<P: 'static + Hash + Primitive>(x: f32, y: f32, primitive: P) -> Instruction {
        let mut hasher = DefaultHasher::new();
        primitive.hash(&mut hasher);
        Instruction {
            transform: Transform::from_translate(x, y),
            primitive: PrimitiveType::Other {
                name: std::any::type_name::<P>(),
                id: hasher.finish(),
                primitive: Box::new(primitive),
            },
        }
    }
    pub fn new<P: Into<PrimitiveType>>(x: f32, y: f32, primitive: P) -> Instruction {
        Instruction {
            primitive: primitive.into(),
            transform: Transform::from_translate(x, y),
        }
    }
    pub fn transform(mut self, tranform: Transform) -> Instruction {
        self.transform = self.transform.post_concat(tranform);
        self
    }
}

impl Instruction {
    fn render(&self, ctx: &mut DrawContext, clip: Option<&ClipMask>) {
        let x = self.transform.tx;
        let y = self.transform.ty;
        match &self.primitive {
            PrimitiveType::Image(i) => {
                i.draw_with_transform_clip(ctx, self.transform, clip);
                ctx.commit(Region::new(x, y, i.width(), i.height()));
            }
            PrimitiveType::Other {
                id: _,
                name: _,
                primitive,
            } => {
                primitive.draw_with_transform_clip(ctx, self.transform, clip);
                ctx.commit(Region::new(x, y, primitive.width(), primitive.height()));
            }
            PrimitiveType::Rectangle(r) => {
                r.draw_with_transform_clip(ctx, self.transform, clip);
                ctx.commit(Region::new(x, y, r.width(), r.height()));
            }
            PrimitiveType::Label(l) => {
                ctx.draw_label(l, x, y);
                ctx.commit(Region::new(x, y, l.width(), l.height()));
            }
        }
    }
    fn region(&self) -> Region {
        Region::new(
            self.transform.tx,
            self.transform.ty,
            match &self.primitive {
                PrimitiveType::Image(i) => i.width(),
                PrimitiveType::Rectangle(r) => r.width(),
                PrimitiveType::Label(l) => l.width(),
                PrimitiveType::Other {
                    id: _,
                    name: _,
                    primitive,
                } => primitive.width(),
            },
            match &self.primitive {
                PrimitiveType::Image(i) => i.height(),
                PrimitiveType::Rectangle(r) => r.height(),
                PrimitiveType::Label(l) => l.height(),
                PrimitiveType::Other {
                    id: _,
                    name: _,
                    primitive,
                } => primitive.height(),
            },
        )
    }
}

impl PartialEq for Instruction {
    fn eq(&self, other: &Self) -> bool {
        self.transform.eq(&other.transform) && self.primitive.eq(&other.primitive)
    }
    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

#[derive(Debug, PartialEq)]
pub enum RenderNode {
    Instruction(Instruction),
    Extension {
        background: Instruction,
        node: Box<(RenderNode, RenderNode)>,
    },
    None,
    Container(Vec<RenderNode>),
    Draw {
        region: Region,
        steps: Vec<Instruction>,
    },
}

impl Default for RenderNode {
    fn default() -> Self {
        RenderNode::None
    }
}

impl RenderNode {
    pub fn render(&self, ctx: &mut DrawContext) {
        match self {
            Self::Instruction(instruction) => instruction.render(ctx, None),
            Self::Container(c) => {
                for n in c {
                    n.render(ctx);
                }
            }
            Self::Extension { background, node } => {
                background.render(ctx, None);
                let (child, border) = node.as_ref();
                child.render(ctx);
                border.render(ctx);
            }
            Self::Draw { region, steps } => {
                // ClipMask expects the mask to be the size of the buffer
                let mut clip = ClipMask::new();
                clip.set_path(
                    ctx.width() as u32,
                    ctx.height() as u32,
                    &PathBuilder::from_rect(region.into()),
                    FillRule::Winding,
                    false,
                );
                for n in steps {
                    n.render(ctx, Some(&clip));
                }
            }
            _ => {}
        }
    }
    fn clear(&self, ctx: &mut DrawContext, bg: &Background) {
        match self {
            RenderNode::Instruction(instruction) => {
                ctx.damage_region(bg, instruction.region());
            }
            RenderNode::Extension { background, node } => {
                if let RenderNode::None = node.1 {
                    ctx.damage_region(bg, background.region());
                } else {
                    node.1.clear(ctx, bg);
                }
            }
            RenderNode::Container(nodes) => {
                for node in nodes {
                    node.clear(ctx, bg)
                }
            }
            RenderNode::Draw { region, steps: _ } => {
                ctx.damage_region(bg, *region);
            }
            _ => {}
        }
    }
    pub fn merge<'r>(&'r mut self, other: Self, ctx: &mut DrawContext, bg: &Background) {
        match self {
            RenderNode::Instruction(a) => match other {
                RenderNode::Instruction(ref b) => {
                    if b.ne(a) {
                        ctx.damage_region(bg, a.region());
                        b.render(ctx, None);
                        *self = other;
                    }
                }
                RenderNode::None => {}
                _ => {
                    ctx.damage_region(bg, a.region());
                    *self = other;
                    self.render(ctx);
                }
            },
            RenderNode::None => {
                *self = other;
                self.render(ctx);
            }
            RenderNode::Container(sv) => match other {
                RenderNode::Container(mut ov) => {
                    if sv.len() != ov.len() {
                        self.clear(ctx, bg);
                        *self = RenderNode::Container(ov);
                        self.render(ctx);
                    } else {
                        for i in 0..ov.len() {
                            sv[i].merge(mem::take(&mut ov[i]), ctx, bg);
                        }
                    }
                }
                RenderNode::None => {}
                _ => {
                    self.clear(ctx, bg);
                    *self = other;
                    self.render(ctx);
                }
            },
            RenderNode::Extension { background, node } => {
                let this_background = background;
                let (this_child, this_border) = node.as_mut();
                if let RenderNode::Extension { background, node } = other {
                    let (other_child, other_border) = *node;
                    if background.eq(this_background) && other_border.eq(this_border) {
                        this_child.merge(
                            other_child,
                            ctx,
                            &bg.merge(Background::from(this_background)),
                        );
                    } else {
                        self.clear(ctx, bg);
                        *self = RenderNode::Extension {
                            background,
                            node: Box::new((other_child, other_border)),
                        };
                        self.render(ctx);
                    }
                } else {
                    self.clear(ctx, bg);
                    *self = other;
                    self.render(ctx);
                }
            }
            RenderNode::Draw { region, steps } => {
                let this_region = region;
                let this_steps = steps;
                match other {
                    RenderNode::Draw { region, steps } => {
                        if region.eq(this_region) && this_steps.len() != steps.len() {
                            self.clear(ctx, bg);
                            *self = RenderNode::Draw { region, steps };
                            self.render(ctx);
                        } else {
                            let mut draw = false;
                            for i in 0..steps.len() {
                                if this_steps[i] != steps[i] {
                                    draw = true;
                                    break;
                                }
                            }
                            if draw {
                                self.clear(ctx, bg);
                                *self = RenderNode::Draw { region, steps };
                                self.render(ctx);
                            }
                        }
                    }
                    RenderNode::None => {}
                    _ => {
                        self.clear(ctx, bg);
                        *self = other;
                        self.render(ctx);
                    }
                }
            }
        }
    }

    // Renders to the DrawContext where the RenderNode differs
    pub fn compare<'r>(&'r self, other: &'r Self, ctx: &mut DrawContext, bg: &Background) {
        match self {
            RenderNode::Instruction(a) => match other {
                RenderNode::Instruction(b) => {
                    if a.ne(b) {
                        ctx.damage_region(bg, a.region());
                        b.render(ctx, None);
                    }
                }
                RenderNode::None => {}
                _ => {
                    ctx.damage_region(bg, a.region());
                    other.render(ctx);
                }
            },
            RenderNode::None => {
                other.render(ctx);
            }
            RenderNode::Container(sv) => match other {
                RenderNode::Container(ov) => {
                    if sv.len() != ov.len() {
                        self.clear(ctx, bg);
                        other.render(ctx);
                    } else {
                        for i in 0..ov.len() {
                            sv[i].compare(&ov[i], ctx, bg);
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
                        this_child.compare(
                            other_child,
                            ctx,
                            &bg.merge(Background::from(this_background)),
                        );
                    } else {
                        self.clear(ctx, bg);
                        other.render(ctx);
                    }
                } else {
                    self.clear(ctx, bg);
                    other.render(ctx);
                }
            }
            RenderNode::Draw { region, steps } => {
                let this_region = region;
                let this_steps = steps;
                match other {
                    RenderNode::Draw { region, steps } => {
                        if this_region == region && this_steps.len() != steps.len() {
                            self.clear(ctx, bg);
                            other.render(ctx);
                        } else {
                            let mut draw = false;
                            for i in 0..steps.len() {
                                if this_steps[i] != steps[i] {
                                    draw = true;
                                    break;
                                }
                            }
                            if draw {
                                self.clear(ctx, bg);
                                other.render(ctx);
                            }
                        }
                    }
                    RenderNode::None => {}
                    _ => {
                        self.clear(ctx, bg);
                        other.render(ctx);
                    }
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

impl From<Region> for Rect {
    fn from(r: Region) -> Self {
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
    pub fn substract(&self, other: Self) -> Self {
        let ox = other.x + other.width;
        let oy = other.y + other.height;
        let sx = self.x + self.width;
        let sy = self.y + self.height;

        if other.contains(self.x, self.y) || other.contains(sx, self.y) {
            let mut new = *self;
            new.x = ox.min(sx);
            if other.contains(new.x, sx) || other.contains(sx, sy) {
                new.y = oy.min(sy);
            }

            new.width = sx - new.x;
            new.height = sy - new.y;

            return new;
        }
        *self
    }
    pub fn merge(&self, other: &Self) -> Self {
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        let fx = (self.x + self.width).max(other.x + other.width);
        let fy = (self.y + self.height).max(other.y + other.height);

        Region {
            x,
            y,
            width: fx - x,
            height: fy - y,
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
