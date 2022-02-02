use crate::*;
use context::DrawContext;
use std::rc::Rc;
pub use tiny_skia::*;
use widgets::blend;

use widgets::label::*;
use widgets::shapes::*;
use widgets::InnerImage as Image;

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

impl From<Coords> for Point {
    fn from(coords: Coords) -> Self {
        Point::from_xy(coords.x, coords.y)
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

impl<I> From<I> for Texture
where
    I: Into<Image>,
{
    fn from(image: I) -> Self {
        Texture::Image(Coords::new(0., 0.), image.into())
    }
}

impl From<Instruction> for Texture {
    fn from(instruction: Instruction) -> Self {
        match instruction.primitive {
            PrimitiveType::Rectangle(r) => r.get_style().background(),
            _ => Texture::Transparent,
        }
    }
}

impl From<&Instruction> for Texture {
    fn from(instruction: &Instruction) -> Self {
        match &instruction.primitive {
            PrimitiveType::Rectangle(r) => r.get_style().background(),
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
    Rectangle(Rectangle),
    Other(Box<dyn Primitive>),
}

impl Geometry for PrimitiveType {
    fn width(&self) -> f32 {
        match self {
            Self::Other(primitive) => primitive.width(),
            Self::Label(l) => l.width(),
            Self::Rectangle(r) => r.width(),
        }
    }
    fn height(&self) -> f32 {
        match self {
            Self::Other(primitive) => primitive.height(),
            Self::Label(l) => l.height(),
            Self::Rectangle(r) => r.height(),
        }
    }
    fn set_height(&mut self, height: f32) -> Result<(), f32> {
        match self {
            Self::Other(primitive) => primitive.set_height(height),
            Self::Label(l) => l.set_height(height),
            Self::Rectangle(r) => r.set_height(height),
        }
    }
    fn set_width(&mut self, width: f32) -> Result<(), f32> {
        match self {
            Self::Other(primitive) => primitive.set_width(width),
            Self::Label(l) => l.set_width(width),
            Self::Rectangle(r) => r.set_width(width),
        }
    }
}

impl Clone for PrimitiveType {
    fn clone(&self) -> Self {
        match self {
            Self::Label(label) => label.clone().into(),
            Self::Rectangle(rect) => rect.primitive_type(),
            Self::Other(primitive) => primitive.primitive_type(),
        }
    }
}

impl Primitive for PrimitiveType {
    fn get_texture(&self) -> Texture {
        match self {
            Self::Rectangle(rectangle) => rectangle.get_texture(),
            Self::Label(_) => Texture::Transparent,
            Self::Other(primitive) => primitive.get_texture(),
        }
    }
    fn apply_texture(&self, background: Texture) -> Self {
        match self {
            Self::Rectangle(rectangle) => rectangle.apply_texture(background),
            Self::Label(_) => Rectangle::empty(self.width(), self.height())
                .background(background)
                .into(),
            Self::Other(primitive) => primitive.apply_texture(background),
        }
    }
    fn contains(&self, region: &Region) -> bool {
        match &self {
            PrimitiveType::Rectangle(rect) => Primitive::contains(rect, &region),
            PrimitiveType::Other(primitive) => Primitive::contains(&**primitive, &region),
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
            Self::Rectangle(rectangle) => rectangle.draw_with_transform_clip(ctx, transform, clip),
            Self::Label(l) => ctx.draw_label(transform.tx, transform.ty, l.as_ref()),
            Self::Other(primitive) => primitive.draw_with_transform_clip(ctx, transform, clip),
        }
    }
    fn primitive_type(&self) -> scene::PrimitiveType {
        self.clone()
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
            PrimitiveType::Other(t_primitive) => {
                if let PrimitiveType::Other(primitive) = other {
                    return primitive.as_ref().same(&*t_primitive);
                }
            }
        }
        false
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
        Instruction::new(Transform::from_translate(region.x, region.y), p)
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

#[derive(Debug, Clone, PartialEq)]
pub struct Instruction {
    pub(crate) transform: Transform,
    pub(crate) primitive: PrimitiveType,
}

impl From<Region> for Instruction {
    fn from(region: Region) -> Self {
        Instruction {
            transform: Transform::from_translate(region.x, region.y),
            primitive: Rectangle::empty(region.width, region.height).into(),
        }
    }
}

impl Instruction {
    pub fn other<P: 'static + Primitive>(mut transform: Transform, primitive: P) -> Instruction {
        transform.tx = transform.tx.round();
        transform.ty = transform.ty.round();
        Instruction {
            transform,
            primitive: PrimitiveType::Other(Box::new(primitive)),
        }
    }
    pub fn new<P: Into<PrimitiveType>>(mut transform: Transform, primitive: P) -> Instruction {
        transform.tx = transform.tx.round();
        transform.ty = transform.ty.round();
        Instruction {
            transform,
            primitive: primitive.into(),
        }
    }
    pub fn transform(mut self, tranform: Transform) -> Instruction {
        self.transform = self.transform.post_concat(tranform);
        self
    }
    fn contains(&self, other: &Self) -> bool {
        let region = Region::new(
            other.transform.tx,
            other.transform.ty,
            other.width(),
            other.height(),
        )
        .relative_to(self.transform.tx, self.transform.ty);
        match &self.primitive {
            PrimitiveType::Rectangle(rect) => Primitive::contains(rect, &region),
            _ => true,
        }
    }
}

impl Instruction {
    fn render(&self, ctx: &mut DrawContext, clip: Option<&ClipMask>) {
        match &self.primitive {
            PrimitiveType::Other(primitive) => {
                primitive.draw_with_transform_clip(ctx, self.transform, clip);
            }
            PrimitiveType::Rectangle(r) => {
                r.draw_with_transform_clip(ctx, self.transform, clip);
            }
            PrimitiveType::Label(l) => {
                let x = self.transform.tx;
                let y = self.transform.ty;
                ctx.draw_label(x, y, l.as_ref());
            }
        }
    }
    fn region(&self) -> Region {
        Region::new(
            self.transform.tx,
            self.transform.ty,
            self.width() * self.transform.sx,
            self.height() * self.transform.sy,
        )
    }
}

impl Geometry for Instruction {
    fn width(&self) -> f32 {
        match &self.primitive {
            PrimitiveType::Rectangle(r) => r.width(),
            PrimitiveType::Label(l) => l.width(),
            PrimitiveType::Other(primitive) => primitive.width(),
        }
    }
    fn height(&self) -> f32 {
        match &self.primitive {
            PrimitiveType::Rectangle(r) => r.height(),
            PrimitiveType::Label(l) => l.height(),
            PrimitiveType::Other(primitive) => primitive.height(),
        }
    }
}

#[derive(Debug)]
pub struct ClipRegion<'c> {
    region: Region,
    clipmask: Option<&'c mut ClipMask>,
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
                    FillRule::Winding,
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
    Container(Instruction, Vec<RenderNode>),
    Clip(Instruction, Box<RenderNode>),
}

impl Geometry for RenderNode {
    fn width(&self) -> f32 {
        match self {
            RenderNode::Container(region, _) => region.width(),
            RenderNode::Instruction(instruction) => instruction.width(),
            RenderNode::Extension {
                background,
                border,
                node: _,
            } => border.as_ref().unwrap_or(background).width(),
            RenderNode::Clip(region, ..) => region.width(),
            RenderNode::None => 0.,
        }
    }
    fn height(&self) -> f32 {
        match self {
            RenderNode::Container(region, _) => region.height(),
            RenderNode::Instruction(instruction) => instruction.height(),
            RenderNode::Extension {
                background,
                border,
                node: _,
            } => border.as_ref().unwrap_or(background).height(),
            RenderNode::Clip(region, ..) => region.height(),
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
            Self::Container(_, v) => v.is_empty(),
            _ => false,
        }
    }
    pub fn as_option(self) -> Option<Self> {
        if self.is_none() {
            None
        } else {
            Some(self)
        }
    }
    pub fn region(&self) -> Option<Region> {
        match self {
            Self::Clip(region, _) => Some(region.region()),
            Self::Container(region, _) => Some(region.region()),
            Self::Extension {
                background, border, ..
            } => Some(border.as_ref().unwrap_or(background).region()),
            Self::Instruction(instruction) => Some(instruction.region()),
            Self::None => None,
        }
    }
    pub fn render(&self, ctx: &mut DrawContext, clip: &mut ClipRegion) {
        match self {
            Self::Instruction(instruction) => instruction.render(ctx, clip.clipmask()),
            Self::Container(_, nodes) => {
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
            Self::Clip(region, node) => {
                let previous = clip.region;
                if clip
                    .set_region(ctx.width(), ctx.height(), region.region())
                    .is_some()
                {
                    node.render(ctx, clip);
                    clip.set_region(ctx.width(), ctx.height(), previous);
                }
            }
            _ => {}
        }
    }
    pub fn merge<'r>(&'r mut self, other: Self) {
        match self {
            RenderNode::Container(t_region, t_nodes) => match other {
                RenderNode::Container(region, nodes) => {
                    *t_region = region;
                    let len = nodes.len();
                    let clear = t_nodes.len() > nodes.len();
                    for (i, node) in nodes.into_iter().enumerate() {
                        if let Some(t_node) = t_nodes.get_mut(i) {
                            t_node.merge(node);
                        } else {
                            t_nodes.push(node)
                        }
                    }
                    if clear {
                        t_nodes.truncate(len);
                    }
                }
                RenderNode::None => {}
                _ => {
                    *self = other;
                }
            },
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
            Self::Clip(region, node) => {
                let t_region = region;
                let t_node = node.as_mut();
                match other {
                    Self::Clip(region, node) => {
                        *t_region = region;
                        t_node.merge(*node);
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
                        let region = b.region();
                        if shape.contains(b) {
                            let merge = clip.crop(&a.region().merge(&region));
                            if clip.intersect(&merge) {
                                ctx.damage_region(&Texture::from(shape), merge, false);
                                b.render(ctx, clip.clipmask());
                            }
                            *self = other;
                        } else {
                            *self = other;
                            return Err(region);
                        }
                    }
                }
                RenderNode::None => {}
                _ => {
                    let region = a.region().merge(&other.region().unwrap());
                    ctx.damage_region(&Texture::from(shape), clip.crop(&region), false);
                    *self = other;
                    self.render(ctx, clip);
                }
            },
            RenderNode::None => {
                if let Some(region) = other.region() {
                    ctx.damage_region(&Texture::from(shape), clip.crop(&region), false);
                }
                *self = other;
                self.render(ctx, clip);
            }
            RenderNode::Container(t_region, t_nodes) => match other {
                RenderNode::Container(region, nodes) => {
                    *t_region = region;
                    if !shape.contains(&t_region) {
                        return Err(t_region.region());
                    } else {
                        let len = nodes.len();
                        let clear = t_nodes.len() > nodes.len();
                        for (i, node) in nodes.into_iter().enumerate() {
                            if let Some(t_node) = t_nodes.get_mut(i) {
                                if let Err(region) = t_node.draw_merge(node, ctx, shape, clip) {
                                    ctx.damage_region(
                                        &Texture::from(shape),
                                        clip.crop(&region),
                                        false,
                                    );
                                    t_node.render(ctx, clip);
                                }
                            } else {
                                t_nodes.push(node);
                            }
                        }
                        if clear {
                            t_nodes.truncate(len);
                        }
                    }
                }
                RenderNode::None => {}
                _ => {
                    let region = t_region.region().merge(&other.region().unwrap());
                    ctx.damage_region(&Texture::from(shape), clip.crop(&region), false);
                    self.merge(other);
                    self.render(ctx, clip);
                }
            },
            RenderNode::Extension {
                background,
                border,
                node,
            } => {
                let t_node = node.as_mut();
                let t_border = border;
                let t_background = background;

                match other {
                    RenderNode::Extension {
                        background,
                        border,
                        node,
                    } => {
                        if background.eq(t_background) && border.eq(t_border) {
                            let instruction = Instruction::new(
                                background.transform,
                                shape.primitive.merge(background.primitive.clone()),
                            );
                            if let Err(region) = t_node.draw_merge(*node, ctx, &instruction, clip) {
                                shape.primitive.instruction(region).render(ctx, None);
                                self.render(ctx, clip);
                            };
                        } else {
                            let instruction = t_border.as_ref().unwrap_or(t_background);
                            let contains = shape.contains(instruction);
                            let merge = instruction
                                .region()
                                .merge(&border.as_ref().unwrap_or(&background).region());
                            if clip.intersect(&merge) {
                                t_node.merge(*node);
                                *t_border = border;
                                *t_background = background;
                                if !contains {
                                    return Err(merge);
                                }
                                ctx.damage_region(&Texture::from(shape), clip.crop(&merge), false);
                                self.render(ctx, clip);
                            }
                        }
                    }
                    RenderNode::None => {}
                    _ => {
                        let region = t_border
                            .as_ref()
                            .unwrap_or(&t_background)
                            .region()
                            .merge(&other.region().unwrap());
                        ctx.damage_region(&Texture::from(shape), clip.crop(&region), false);
                        self.merge(other);
                        self.render(ctx, clip);
                    }
                }
            }
            RenderNode::Clip(t_region, t_node) => {
                let previous = clip.region;

                match other {
                    RenderNode::Clip(region, node) => {
                        *t_region = region;
                        clip.set_region(ctx.width(), ctx.height(), t_region.region());
                        if let Err(region) = t_node.draw_merge(*node, ctx, shape, clip) {
                            clip.set_region(ctx.width(), ctx.height(), previous);
                            return Err(region);
                        }
                    }
                    RenderNode::None => {}
                    _ => {
                        let region = t_region.region().merge(&other.region().unwrap());
                        ctx.damage_region(&Texture::from(shape), clip.crop(&region), false);
                        self.merge(other);
                        self.render(ctx, clip);
                    }
                }

                clip.set_region(ctx.width(), ctx.height(), previous);
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
    /// Returns regions other doesn't occupy in Self.
    /// Because of the way RenderNode are iterated and the layout of the widget system
    /// it is not necessary to return a third Region to accurately damage the scene.
    // +----------------+---------------+
    // |				|				|
    // |				|		2		|
    // |				|				|
    // |		1		+---------------+-----------+
    // |				|							|
    // |				|							|
    // |				|			Other			|
    // +----------------|							|
    // 					|							|
    // 					+---------------------------+
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
                if crop.x == self.x {
                    self.x
                } else {
                    crop.x + crop.width
                },
                if crop.y == self.y {
                    crop.y + crop.height
                } else {
                    self.y
                },
                crop.width,
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
    pub fn transform(transform: Transform, width: f32, height: f32) -> Self {
        Self::new(
            transform.tx,
            transform.ty,
            width * transform.sx,
            height * transform.sy,
        )
    }
    pub fn translate(&self, x: f32, y: f32) -> Self {
        Region::new(self.x + x, self.y + y, self.width, self.height)
    }
    pub fn relative_to(&self, x: f32, y: f32) -> Self {
        Region::new(self.x - x, self.y - y, self.width, self.height)
    }
    pub fn rfit(&self, other: &Self) -> bool {
        other.x - self.x + other.width <= self.width
            && other.y - self.y + other.height <= self.height
    }
    pub fn fit(&self, other: &Self) -> bool {
        other.rfit(self)
    }
    pub fn contains(&self, x: f32, y: f32) -> bool {
        self.x <= x && x - self.x < self.width && self.y <= y && y - self.y < self.height
    }
    pub fn scale(&self, sx: f32, sy: f32) -> Self {
        Self::new(self.x * sx, self.y * sy, self.width * sx, self.height * sy)
    }
    pub fn rect(&self) -> (Transform, Rectangle) {
        (
            Transform::from_translate(self.x, self.y),
            Rectangle::empty(self.width, self.height),
        )
    }
    pub fn null(&self) -> bool {
        self.width == 0. || self.height == 0.
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
