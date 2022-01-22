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

impl Default for Coords {
    fn default() -> Self {
        Coords { x: 0., y: 0. }
    }
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
pub enum Texture {
    Transparent,
    Image(Coords, Image),
    LinearGradient {
        start: Coords,
        end: Coords,
        angle: f32,
        mode: SpreadMode,
        stops: Rc<[GradientStop]>,
    },
    Composite(Vec<Texture>),
    Color(Color),
}

impl PartialEq for Texture {
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
            Self::Composite(sl) => {
                if let Self::Composite(ol) = other {
                    return sl.eq(ol);
                }
            }
        }
        false
    }
}

impl From<ShapeStyle> for Texture {
    fn from(style: ShapeStyle) -> Self {
        match style {
            ShapeStyle::Background(texture) => texture,
            ShapeStyle::Border(_, _) => Texture::Transparent,
        }
    }
}

impl From<u32> for Texture {
    fn from(color: u32) -> Self {
        Texture::Color(u32_to_source(color))
    }
}

impl From<Color> for Texture {
    fn from(color: Color) -> Self {
        Texture::Color(color)
    }
}

impl From<ColorU8> for Texture {
    fn from(color: ColorU8) -> Self {
        color.get().into()
    }
}

impl From<Image> for Texture {
    fn from(image: Image) -> Self {
        Texture::Image(Coords::new(0., 0.), image)
    }
}

impl From<Instruction> for Texture {
    fn from(instruction: Instruction) -> Self {
        match instruction.primitive {
            PrimitiveType::Rectangle(r) => r.get_style().background(),
            PrimitiveType::Image(image) => {
                let coords = Coords::new(instruction.transform.tx, instruction.transform.ty);
                Texture::Image(coords, image.clone())
            }
            _ => Texture::Transparent,
        }
    }
}

impl From<&Instruction> for Texture {
    fn from(instruction: &Instruction) -> Self {
        match &instruction.primitive {
            PrimitiveType::Rectangle(r) => r.get_style().background(),
            PrimitiveType::Image(image) => {
                let coords = Coords::new(instruction.transform.tx, instruction.transform.ty);
                Texture::Image(coords, image.clone())
            }
            _ => Texture::Transparent,
        }
    }
}

impl Texture {
    pub fn solid(color: u32) -> Texture {
        Texture::Color(u32_to_source(color))
    }
    // The angle is a radiant representing the tild of the gradient clock wise.
    pub fn linear_gradient(stops: Vec<GradientStop>, mode: SpreadMode, angle: f32) -> Texture {
        let stops: Rc<[GradientStop]> = stops.into();
        Texture::LinearGradient {
            angle,
            start: Coords::new(0., 0.),
            end: Coords::new(0., 0.),
            mode,
            stops,
        }
    }
    pub fn is_transparent(&self) -> bool {
        match self {
            Self::Transparent => true,
            _ => false,
        }
    }
    pub fn merge(&self, other: Self) -> Self {
        match self {
            Texture::Color(acolor) => match other {
                Texture::Color(bcolor) => {
                    if bcolor.is_opaque() {
                        return other;
                    }
                    Texture::Color(blend(acolor, &bcolor))
                }
                Texture::Image(_, _) => Texture::Composite(vec![self.clone(), other]),
                Texture::Transparent => self.clone(),
                Texture::Composite(mut layers) => {
                    layers.insert(0, self.clone());
                    Texture::Composite(layers)
                }
                _ => Texture::Composite(vec![self.clone(), other]),
            },
            Texture::LinearGradient {
                start: _,
                end: _,
                stops: _,
                mode: _,
                angle: _,
            } => match other {
                Texture::Color(color) => {
                    if color.is_opaque() {
                        return other;
                    } else {
                        Texture::Composite(vec![self.clone(), other])
                    }
                }
                Texture::Transparent => return self.clone(),
                Texture::Composite(mut layers) => {
                    layers.insert(0, self.clone());
                    Texture::Composite(layers)
                }
                _ => Texture::Composite(vec![self.clone(), other]),
            },
            Texture::Image(_, _) => match other {
                Texture::Color(color) => {
                    if color.is_opaque() {
                        return other;
                    } else {
                        Texture::Composite(vec![self.clone(), other])
                    }
                }
                Texture::Transparent => return self.clone(),
                Texture::Composite(mut layers) => {
                    layers.insert(0, self.clone());
                    Texture::Composite(layers)
                }
                _ => Texture::Composite(vec![self.clone(), other]),
            },
            Texture::Composite(sl) => {
                let mut layers = sl.clone();
                if let Some(last) = layers.pop() {
                    let background = last.merge(other);
                    match background {
                        Texture::Composite(mut ol) => {
                            layers.append(&mut ol);
                            return Texture::Composite(layers);
                        }
                        _ => layers.push(background),
                    }
                }
                Texture::Composite(layers)
            }
            Texture::Transparent => other,
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

impl Geometry for PrimitiveType {
    fn width(&self) -> f32 {
        match self {
            Self::Other {
                name: _,
                id: _,
                primitive,
            } => primitive.width(),
            Self::Label(l) => l.width(),
            Self::Rectangle(r) => r.width(),
            Self::Image(i) => i.width(),
        }
    }
    fn height(&self) -> f32 {
        match self {
            Self::Other {
                name: _,
                id: _,
                primitive,
            } => primitive.height(),
            Self::Label(l) => l.height(),
            Self::Rectangle(r) => r.height(),
            Self::Image(i) => i.height(),
        }
    }
    fn set_height(&mut self, height: f32) -> Result<(), f32> {
        match self {
            Self::Other {
                name: _,
                id: _,
                primitive,
            } => primitive.set_height(height),
            Self::Label(l) => l.set_height(height),
            Self::Rectangle(r) => r.set_height(height),
            Self::Image(i) => Err(i.height()),
        }
    }
    fn set_width(&mut self, width: f32) -> Result<(), f32> {
        match self {
            Self::Other {
                name: _,
                id: _,
                primitive,
            } => primitive.set_width(width),
            Self::Label(l) => l.set_width(width),
            Self::Rectangle(r) => r.set_width(width),
            Self::Image(i) => Err(i.width()),
        }
    }
}

impl Clone for PrimitiveType {
    fn clone(&self) -> Self {
        match self {
            Self::Image(image) => image.primitive_type(),
            Self::Rectangle(rect) => rect.primitive_type(),
            Self::Label(label) => label.clone().into(),
            Self::Other {
                name: _,
                id: _,
                primitive,
            } => primitive.primitive_type(),
        }
    }
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

impl Primitive for PrimitiveType {
    fn get_texture(&self) -> Texture {
        match self {
            Self::Image(image) => image.get_texture(),
            Self::Rectangle(rectangle) => rectangle.get_texture(),
            Self::Label(_) => Texture::Transparent,
            Self::Other {
                name: _,
                id: _,
                primitive,
            } => primitive.get_texture(),
        }
    }
    fn apply_texture(&self, background: Texture) -> Self {
        match self {
            Self::Image(image) => image.apply_texture(background),
            Self::Rectangle(rectangle) => rectangle.apply_texture(background),
            Self::Label(_) => Rectangle::empty(self.width(), self.height())
                .background(background)
                .into(),
            Self::Other {
                name: _,
                id: _,
                primitive,
            } => primitive.apply_texture(background),
        }
    }
    fn contains(&self, region: &Region) -> bool {
        match &self {
            PrimitiveType::Rectangle(rect) => Primitive::contains(rect, &region),
            _ => true,
        }
    }
    fn draw_with_transform_clip(
        &self,
        ctx: &mut DrawContext,
        transform: tiny_skia::Transform,
        clip: Option<&tiny_skia::ClipMask>,
    ) {
        match self {
            Self::Image(image) => image.draw_with_transform_clip(ctx, transform, clip),
            Self::Rectangle(rectangle) => rectangle.draw_with_transform_clip(ctx, transform, clip),
            Self::Label(l) => ctx.draw_label(l, transform.tx, transform.ty),
            Self::Other {
                name: _,
                id: _,
                primitive,
            } => primitive.draw_with_transform_clip(ctx, transform, clip),
        }
    }
    // Basically Clone
    fn primitive_type(&self) -> scene::PrimitiveType {
        self.clone()
    }
}

impl PrimitiveType {
    fn merge(&self, other: Self) -> Self {
        let background = other.get_texture();
        let background = self.get_texture().merge(background);
        other.apply_texture(background)
    }
    fn instruction(&self, region: Region) -> Instruction {
        let mut p = self.clone();
        let _ = p.set_size(region.width, region.height);
        Instruction::new(region.x, region.y, p)
    }
}

impl Default for PrimitiveType {
    fn default() -> Self {
        Rectangle::empty(0., 0.).into()
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
            transform: Transform::from_translate(x.round(), y.round()),
        }
    }
    pub fn transform(mut self, tranform: Transform) -> Instruction {
        self.transform = self.transform.post_concat(tranform);
        self
    }
    pub fn empty(x: f32, y: f32, width: f32, height: f32) -> Instruction {
        Instruction {
            primitive: Rectangle::empty(width, height).into(),
            transform: Transform::from_translate(x, y),
        }
    }
    fn contains(&self, region: &Region) -> bool {
        let region = region.relative_to(self.transform.tx, self.transform.ty);
        match &self.primitive {
            PrimitiveType::Rectangle(rect) => Primitive::contains(rect, &region),
            _ => true,
        }
    }
}

impl Instruction {
    fn render(&self, ctx: &mut DrawContext, clip: Option<&ClipMask>) {
        let x = self.transform.tx;
        let y = self.transform.ty;
        match &self.primitive {
            PrimitiveType::Image(i) => {
                i.draw_with_transform_clip(ctx, self.transform, clip);
            }
            PrimitiveType::Other {
                id: _,
                name: _,
                primitive,
            } => {
                primitive.draw_with_transform_clip(ctx, self.transform, clip);
            }
            PrimitiveType::Rectangle(r) => {
                r.draw_with_transform_clip(ctx, self.transform, clip);
            }
            PrimitiveType::Label(l) => {
                ctx.draw_label(l, x, y);
            }
        }
        ctx.commit(self.region());
    }
    fn region(&self) -> Region {
        Region::new(
            self.transform.tx,
            self.transform.ty,
            self.width(),
            self.height(),
        )
    }
}

impl Geometry for Instruction {
    fn width(&self) -> f32 {
        match &self.primitive {
            PrimitiveType::Image(i) => i.width(),
            PrimitiveType::Rectangle(r) => r.width(),
            PrimitiveType::Label(l) => l.width(),
            PrimitiveType::Other {
                id: _,
                name: _,
                primitive,
            } => primitive.width(),
        }
    }
    fn height(&self) -> f32 {
        match &self.primitive {
            PrimitiveType::Image(i) => i.height(),
            PrimitiveType::Rectangle(r) => r.height(),
            PrimitiveType::Label(l) => l.height(),
            PrimitiveType::Other {
                id: _,
                name: _,
                primitive,
            } => primitive.height(),
        }
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

#[derive(Debug)]
pub struct ClipRegion<'c> {
    region: Region,
    clipmask: Option<&'c mut ClipMask>,
}

impl<'c> AsRef<Region> for ClipRegion<'c> {
    fn as_ref(&self) -> &Region {
        &self.region
    }
}

use std::ops::{Deref, DerefMut};

impl<'c> Deref for ClipRegion<'c> {
    type Target = Region;
    fn deref(&self) -> &Self::Target {
        &self.region
    }
}

impl<'c> DerefMut for ClipRegion<'c> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.region
    }
}

impl<'c> ClipRegion<'c> {
    pub fn new(region: Region, clipmask: Option<&'c mut ClipMask>) -> Self {
        Self { region, clipmask }
    }
    fn set_region(&mut self, width: f32, height: f32, region: Region) -> Option<()> {
        if region == Region::new(0., 0., width, height) {
            self.region = region;
            self.clear();
            None
        } else {
            self.region = region;
            if let Some(clipmask) = &mut self.clipmask {
                clipmask.set_path(
                    width as u32,
                    height as u32,
                    &PathBuilder::from_rect(region.into()),
                    FillRule::EvenOdd,
                    false,
                )
            } else {
                None
            }
        }
    }
    fn clear(&mut self) {
        if let Some(clipmask) = &mut self.clipmask {
            clipmask.clear();
        }
    }
    fn clipmask(&self) -> Option<&ClipMask> {
        if let Some(clipmask) = &self.clipmask {
            if clipmask.is_empty() {
                return None;
            } else {
                Some(&**clipmask)
            }
        } else {
            None
        }
    }
}

/// Node of the scene graph.
#[derive(Debug, PartialEq)]
pub enum RenderNode {
    None,
    Instruction(Instruction),
    Extension {
        background: Instruction,
        border: Option<Instruction>,
        node: Box<RenderNode>,
    },
    Container {
        region: Region,
        nodes: Vec<RenderNode>,
    },
    Clip {
        region: Region,
        node: Box<RenderNode>,
    },
    Draw {
        region: Region,
        steps: Vec<Instruction>,
    },
}

impl Geometry for RenderNode {
    fn width(&self) -> f32 {
        match self {
            RenderNode::Container { region, nodes: _ } => region.width,
            RenderNode::Draw { region, steps: _ } => region.width,
            RenderNode::Instruction(instruction) => instruction.width(),
            RenderNode::Extension {
                background,
                border,
                node: _,
            } => {
                if let Some(border) = border.as_ref() {
                    border.width()
                } else {
                    background.width()
                }
            }
            RenderNode::Clip { region, .. } => region.width,
            RenderNode::None => 0.,
        }
    }
    fn height(&self) -> f32 {
        match self {
            RenderNode::Container { region, nodes: _ } => region.height,
            RenderNode::Draw { region, steps: _ } => region.height,
            RenderNode::Instruction(instruction) => instruction.height(),
            RenderNode::Extension {
                background,
                border,
                node: _,
            } => {
                if let Some(border) = border.as_ref() {
                    border.height()
                } else {
                    background.height()
                }
            }
            RenderNode::Clip { region, .. } => region.height,
            RenderNode::None => 0.,
        }
    }
}

impl From<Instruction> for RenderNode {
    fn from(instruction: Instruction) -> Self {
        RenderNode::Instruction(instruction)
    }
}

impl Default for RenderNode {
    fn default() -> Self {
        RenderNode::None
    }
}

impl RenderNode {
    pub fn is_none(&self) -> bool {
        match self {
            Self::None => true,
            _ => false,
        }
    }
    pub fn as_option(self) -> Option<Self> {
        match self {
            Self::None => None,
            _ => Some(self),
        }
    }
    pub fn snapshot(&self, ctx: &mut DrawContext) -> Option<Image> {
        if self.is_none() {
            return None;
        }
        let mut v = Vec::new();
        let width = self.width() as u32;
        let height = self.height() as u32;
        let mut pixmap = Pixmap::new(width, height)?;
        let mut ctx = DrawContext {
            backend: Backend::Pixmap(pixmap.as_mut()),
            font_cache: ctx.font_cache,
            pending_damage: &mut v,
        };
        self.render(
            &mut ctx,
            &mut ClipRegion {
                region: Region::new(0., 0., width as f32, height as f32),
                clipmask: None,
            },
        );
        Some(Image::from_raw(pixmap.take(), width, height))
    }
    pub fn render(&self, ctx: &mut DrawContext, clip: &mut ClipRegion) {
        match self {
            Self::Instruction(instruction) => instruction.render(ctx, clip.clipmask()),
            Self::Container { region: _, nodes } => {
                for n in nodes {
                    n.render(ctx, clip);
                }
            }
            Self::Extension {
                background,
                border,
                node,
            } => {
                if let Some(border) = border.as_ref() {
                    border.render(ctx, clip.clipmask());
                }
                background.render(ctx, clip.clipmask());
                node.render(ctx, clip);
            }
            Self::Draw { region, steps } => {
                let previous = clip.region;
                // ClipMask expects the mask to be the size of the buffer
                if clip
                    .set_region(ctx.width(), ctx.height(), *region)
                    .is_some()
                {
                    for n in steps {
                        n.render(ctx, clip.clipmask());
                    }
                    clip.set_region(ctx.width(), ctx.height(), previous);
                } else {
                    let mut clip = ClipMask::new();
                    clip.set_path(
                        ctx.width() as u32,
                        ctx.height() as u32,
                        &PathBuilder::from_rect(region.into()),
                        FillRule::EvenOdd,
                        false,
                    );
                    for n in steps {
                        n.render(ctx, Some(&clip));
                    }
                }
            }
            Self::Clip { region, node } => {
                let previous = clip.region;
                if clip
                    .set_region(ctx.width(), ctx.height(), *region)
                    .is_some()
                {
                    node.render(ctx, clip);
                    clip.set_region(ctx.width(), ctx.height(), previous);
                }
            }
            _ => {}
        }
    }
    fn clear(&self, ctx: &mut DrawContext, texture: &Texture, other: Option<&Region>) {
        match self {
            RenderNode::Instruction(instruction) => {
                let region = instruction.region();
                let region = match other {
                    Some(other) => region.merge(other),
                    None => region,
                };
                ctx.damage_region(texture, region, false);
            }
            RenderNode::Extension {
                background,
                border,
                node: _,
            } => {
                if let Some(rect) = border.as_ref() {
                    let region = rect.region();
                    let region = match other {
                        Some(other) => region.merge(other),
                        None => region,
                    };
                    ctx.damage_region(texture, region, false);
                } else {
                    let region = background.region();
                    let region = match other {
                        Some(other) => region.merge(other),
                        None => region,
                    };
                    ctx.damage_region(texture, region, false);
                }
            }
            RenderNode::Container { region, nodes: _ } => {
                let region = match other {
                    Some(other) => region.merge(other),
                    None => *region,
                };
                ctx.damage_region(texture, region, false);
            }
            RenderNode::Draw { region, steps: _ } => {
                let region = match other {
                    Some(other) => region.merge(other),
                    None => *region,
                };
                ctx.damage_region(texture, region, false);
            }
            RenderNode::Clip { region, .. } => {
                let region = match other {
                    Some(other) => region.merge(other),
                    None => *region,
                };
                ctx.damage_region(texture, region, false);
            }
            RenderNode::None => {
                if let Some(region) = other {
                    ctx.damage_region(texture, *region, false);
                }
            }
        }
    }
    pub fn merge<'r>(&'r mut self, other: Self) {
        match self {
            RenderNode::Container { region, nodes } => {
                let this_region = region;
                let this_nodes = nodes;
                match other {
                    RenderNode::Container { region, mut nodes } => {
                        *this_region = region;
                        *this_nodes = (0..nodes.len())
                            .map(|i| {
                                if let Some(node) = this_nodes.get_mut(i) {
                                    let mut node = mem::take(node);
                                    node.merge(mem::take(&mut nodes[i]));
                                    node
                                } else {
                                    mem::take(&mut nodes[i])
                                }
                            })
                            .collect();
                    }
                    RenderNode::None => {}
                    _ => {
                        *self = other;
                    }
                }
            }
            Self::Extension {
                background,
                border,
                node,
            } => {
                let this_node = node.as_mut();
                let this_border = border;
                let this_background = background;
                match other {
                    RenderNode::Extension {
                        background,
                        border,
                        node,
                    } => {
                        *this_background = background;
                        *this_border = border;
                        this_node.merge(*node);
                    }
                    RenderNode::None => {}
                    _ => {
                        *self = other;
                    }
                }
            }
            Self::Clip { region, node } => {
                let this_region = region;
                let this_node = node.as_mut();
                match other {
                    Self::Clip { region, node } => {
                        *this_region = region;
                        this_node.merge(*node);
                    }
                    RenderNode::None => {}
                    _ => {
                        *self = other;
                    }
                }
            }
            _ => match other {
                Self::None => {}
                _ => {
                    *self = other;
                }
            },
        }
    }
    pub fn draw_merge<'r>(
        &'r mut self,
        other: Self,
        ctx: &mut DrawContext,
        shape: &Instruction,
        clip: &mut ClipRegion,
    ) -> Result<(), Region> {
        match self {
            RenderNode::Instruction(a) => match other {
                RenderNode::Instruction(ref b) => {
                    if b.ne(a) {
                        let r = b.region();
                        if shape.contains(&r) {
                            ctx.damage_region(&Texture::from(shape), a.region().merge(&r), false);
                            b.render(ctx, clip.clipmask());
                            *self = other;
                        } else {
                            *self = other;
                            return Err(r);
                        }
                    }
                }
                RenderNode::None => {}
                _ => {
                    other.clear(ctx, &Texture::from(shape), Some(&a.region()));
                    *self = other;
                    self.render(ctx, clip);
                }
            },
            RenderNode::None => {
                *self = other;
                self.clear(ctx, &Texture::from(shape), None);
                self.render(ctx, clip);
            }
            RenderNode::Container { region, nodes } => {
                let this_region = region;
                let this_nodes = nodes;
                match other {
                    RenderNode::Container { region, mut nodes } => {
                        *this_region = region;
                        if !shape.contains(&region) {
                            self.merge(RenderNode::Container { region, nodes });
                            return Err(region);
                        } else {
                            *this_nodes = (0..nodes.len())
                                .map(|i| {
                                    if let Some(node) = this_nodes.get_mut(i) {
                                        let mut node = mem::take(node);
                                        if let Err(region) = node.draw_merge(
                                            mem::take(&mut nodes[i]),
                                            ctx,
                                            shape,
                                            clip,
                                        ) {
                                            ctx.damage_region(&Texture::from(shape), region, false);
                                            node.render(ctx, clip);
                                        }
                                        node
                                    } else {
                                        mem::take(&mut nodes[i])
                                    }
                                })
                                .collect();
                        }
                    }
                    RenderNode::None => {}
                    _ => {
                        other.clear(ctx, &Texture::from(shape), Some(&this_region));
                        self.merge(other);
                        self.render(ctx, clip);
                    }
                }
            }
            RenderNode::Extension {
                background,
                border,
                node,
            } => {
                let this_node = node.as_mut();
                let this_border = border;
                let this_background = background;
                match other {
                    RenderNode::Extension {
                        background,
                        border,
                        node,
                    } => {
                        if background.eq(this_background) && border.eq(this_border) {
                            let instruction = Instruction {
                                transform: background.transform,
                                primitive: shape.primitive.merge(background.primitive.clone()),
                            };
                            if let Err(region) =
                                this_node.draw_merge(*node, ctx, &instruction, clip)
                            {
                                shape.primitive.instruction(region).render(ctx, None);
                                self.render(ctx, clip);
                            };
                        } else {
                            let merge = if let Some(rect) = this_border.as_ref() {
                                rect.region()
                            } else {
                                this_background.region()
                            }
                            .merge(
                                &if let Some(rect) = border.as_ref() {
                                    rect.region()
                                } else {
                                    background.region()
                                },
                            );
                            this_node.merge(*node);
                            *this_border = border;
                            *this_background = background;
                            if !shape.contains(&merge) {
                                return Err(merge);
                            }
                            ctx.damage_region(&Texture::from(shape), merge, false);
                            self.render(ctx, clip);
                        }
                    }
                    RenderNode::None => {}
                    _ => {
                        other.clear(
                            ctx,
                            &Texture::from(shape),
                            Some(&if let Some(border) = this_border.as_ref() {
                                border.region()
                            } else {
                                this_background.region()
                            }),
                        );
                        self.merge(other);
                        self.render(ctx, clip);
                    }
                }
            }
            RenderNode::Clip { region, node } => {
                let previous = clip.region;
                let this_region = region;
                let this_node = node.as_mut();

                match other {
                    RenderNode::Clip { region, node } => {
                        *this_region = region;
                        clip.set_region(ctx.width(), ctx.height(), region);
                        if let Err(region) = this_node.draw_merge(*node, ctx, shape, clip) {
                            clip.set_region(ctx.width(), ctx.height(), previous);
                            return Err(region);
                        }
                    }
                    RenderNode::None => {}
                    _ => {
                        let region = *this_region;
                        self.clear(ctx, &Texture::from(shape), Some(&region));
                        self.merge(other);
                        self.render(ctx, clip);
                    }
                }

                clip.set_region(ctx.width(), ctx.height(), previous);
            }
            RenderNode::Draw { region, steps } => {
                let this_region = *region;
                let this_steps = steps;
                match other {
                    RenderNode::Draw { region, steps } => {
                        if !shape.contains(&region) {
                            *self = RenderNode::Draw { region, steps };
                            return Err(region);
                        } else if steps.ne(this_steps) {
                            self.clear(ctx, &Texture::from(shape), Some(&region));
                            *self = RenderNode::Draw { region, steps };
                            self.render(ctx, clip);
                        }
                    }
                    RenderNode::None => {}
                    _ => {
                        self.clear(ctx, &Texture::from(shape), Some(&this_region));
                        self.merge(other);
                        self.render(ctx, clip);
                    }
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
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
    pub fn crop(&self, other: &Self) -> Region {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let width = (self.x + self.width).min(other.x + other.width) - x;
        let height = (self.y + self.height).min(other.y + other.height) - y;
        Region::new(x, y, width, height)
    }
    pub fn intersect(&self, other: &Self) -> bool {
        let merge = self.merge(other);
        self.width + other.width >= merge.width && self.height + other.height >= merge.height
    }
    // Assume other occupies an entire side of self
    pub fn substract(&self, other: Self) -> [Self; 2] {
        let crop = self.crop(&other);
        [
            Region::new(
                if crop.x == self.x {
                    crop.x + crop.width
                } else {
                    self.x
                },
                self.y,
                self.width - crop.width,
                self.height,
            ),
            Region::new(
                self.x,
                if crop.y == self.y {
                    crop.y + crop.height
                } else {
                    self.y
                },
                self.width,
                self.height - crop.height,
            ),
        ]
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
    pub fn translate(&self, x: f32, y: f32) -> Self {
        Region::new(self.x + x, self.y + y, self.width, self.height)
    }
    pub fn relative_to(&self, x: f32, y: f32) -> Self {
        Region::new(self.x - x, self.y - y, self.width, self.height)
    }
    pub fn fit(&self, other: &Self) -> bool {
        other.x - self.x + other.width <= self.width
            && other.y - self.y + other.height <= self.height
    }
    pub fn contains(&self, x: f32, y: f32) -> bool {
        self.x <= x && x - self.x < self.width && self.y <= y && y - self.y < self.height
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
