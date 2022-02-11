use crate::*;
use context::DrawContext;
use std::rc::Rc;
use tiny_skia::*;
use widgets::blend;

use widgets::image::InnerImage as Image;
use widgets::label::*;
use widgets::shapes::*;

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
            Primitive::Rectangle(r) => r.style.background(),
            _ => Texture::Transparent,
        }
    }
}

impl From<&Instruction> for Texture {
    fn from(instruction: &Instruction) -> Self {
        match &instruction.primitive {
            Primitive::Rectangle(r) => r.style.background(),
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
pub enum Primitive {
    Label(Label),
    Rectangle(Rectangle),
    Other(Box<dyn Drawable>),
}

impl Geometry for Primitive {
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
    fn set_height(&mut self, height: f32) {
        match self {
            Self::Other(primitive) => primitive.set_height(height),
            Self::Label(l) => Geometry::set_height(l, height),
            Self::Rectangle(r) => r.set_height(height),
        }
    }
    fn set_width(&mut self, width: f32) {
        match self {
            Self::Other(primitive) => primitive.set_width(width),
            Self::Label(l) => Geometry::set_width(l, width),
            Self::Rectangle(r) => r.set_width(width),
        }
    }
}

impl Clone for Primitive {
    fn clone(&self) -> Self {
        match self {
            Self::Label(label) => label.clone().into(),
            Self::Rectangle(rect) => rect.primitive(),
            Self::Other(primitive) => primitive.primitive(),
        }
    }
}

impl Drawable for Primitive {
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
            Self::Label(_) => Rectangle::new(self.width(), self.height())
                .background(background)
                .into(),
            Self::Other(primitive) => primitive.apply_texture(background),
        }
    }
    fn contains(&self, region: &Region) -> bool {
        match &self {
            Primitive::Rectangle(rect) => Drawable::contains(rect, &region),
            Primitive::Other(primitive) => Drawable::contains(&**primitive, &region),
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
            Self::Label(l) => ctx.draw_label(transform.tx, transform.ty, l.as_ref(), clip),
            Self::Other(primitive) => primitive.draw_with_transform_clip(ctx, transform, clip),
        }
    }
    fn primitive(&self) -> scene::Primitive {
        self.clone()
    }
}

impl PartialEq for Primitive {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Primitive::Rectangle(s) => {
                if let Primitive::Rectangle(o) = other {
                    return s.eq(o);
                }
            }
            Primitive::Label(s) => {
                if let Primitive::Label(o) = other {
                    return s.eq(o);
                }
            }
            Primitive::Other(t_primitive) => {
                if let Primitive::Other(primitive) = other {
                    return primitive.as_ref().same(&*t_primitive);
                }
            }
        }
        false
    }
}

impl Primitive {
    fn merge(&self, other: Self) -> Self {
        let background = other.get_texture();
        let background = self.get_texture().merge(background);
        other.apply_texture(background)
    }
}

impl Default for Primitive {
    fn default() -> Self {
        Rectangle::new(0., 0.).into()
    }
}

impl From<Rectangle> for Primitive {
    fn from(r: Rectangle) -> Self {
        Primitive::Rectangle(r)
    }
}

impl From<Label> for Primitive {
    fn from(l: Label) -> Self {
        Primitive::Label(l)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Instruction {
    pub(crate) transform: Transform,
    pub(crate) primitive: Primitive,
}

impl From<Region> for Instruction {
    fn from(region: Region) -> Self {
        Instruction {
            transform: Transform::from_translate(region.x, region.y),
            primitive: Rectangle::new(region.width, region.height).into(),
        }
    }
}

impl Instruction {
    pub fn other<P: 'static + Drawable>(mut transform: Transform, primitive: P) -> Instruction {
        transform.tx = transform.tx.round();
        transform.ty = transform.ty.round();
        Instruction {
            transform,
            primitive: Primitive::Other(Box::new(primitive)),
        }
    }
    pub fn new<P: Into<Primitive>>(mut transform: Transform, primitive: P) -> Instruction {
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
            Primitive::Rectangle(rect) => Drawable::contains(rect, &region),
            _ => true,
        }
    }
}

impl Instruction {
    fn render(&self, ctx: &mut DrawContext, clip: Option<&ClipMask>) {
        match &self.primitive {
            Primitive::Other(primitive) => {
                primitive.draw_with_transform_clip(ctx, self.transform, clip);
            }
            Primitive::Rectangle(r) => {
                r.draw_with_transform_clip(ctx, self.transform, clip);
            }
            Primitive::Label(l) => {
                let x = self.transform.tx;
                let y = self.transform.ty;
                ctx.draw_label(x, y, l.as_ref(), clip);
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
            Primitive::Rectangle(r) => r.width(),
            Primitive::Label(l) => l.width(),
            Primitive::Other(primitive) => primitive.width(),
        }
    }
    fn height(&self) -> f32 {
        match &self.primitive {
            Primitive::Rectangle(r) => r.height(),
            Primitive::Label(l) => l.height(),
            Primitive::Other(primitive) => primitive.height(),
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
    fn set_region(&mut self, ctx: &mut DrawContext, region: Region) -> Option<()> {
        if region == Region::new(0., 0., ctx.width(), ctx.height()) {
            self.region = region;
            self.clear();
            None
        } else {
            if let Some(clipmask) = &mut self.clipmask {
                self.region = region;
                let mut pb = ctx.path_builder();
                pb.push_rect(region.x, region.y, region.width, region.height);
                let path = pb.finish().unwrap();
                if clipmask.is_empty() {
                    clipmask.set_path(
                        ctx.width() as u32,
                        ctx.height() as u32,
                        &path,
                        FillRule::Winding,
                        false,
                    );
                }
                ctx.reset(path.clear());
                Some(())
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
    Decoration {
        background: Instruction,
        border: Option<Instruction>,
        node: Box<RenderNode>,
    },
    Container(Vec<RenderNode>),
    Clip(Instruction, Box<RenderNode>),
}

impl Geometry for RenderNode {
    fn width(&self) -> f32 {
        match self {
            RenderNode::Container(_) => self.region().unwrap_or(Region::default()).width,
            RenderNode::Instruction(instruction) => instruction.width(),
            RenderNode::Decoration {
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
            RenderNode::Container(_) => self.region().unwrap_or(Region::default()).height,
            RenderNode::Instruction(instruction) => instruction.height(),
            RenderNode::Decoration {
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
            Self::Container(v) => v.is_empty(),
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
            Self::Container(node) => node
                .iter()
                .filter_map(|node| node.region())
                .reduce(|merge, node| merge.merge(&node)),
            Self::Decoration {
                background, border, ..
            } => Some(border.as_ref().unwrap_or(background).region()),
            Self::Instruction(instruction) => Some(instruction.region()),
            Self::None => None,
        }
    }
    pub fn render(&self, ctx: &mut DrawContext, clip: &mut ClipRegion) {
        match self {
            Self::Instruction(instruction) => {
                if clip.intersect(&instruction.region()) {
                    instruction.render(ctx, clip.clipmask())
                }
            }
            Self::Container(nodes) => {
                for n in nodes {
                    n.render(ctx, clip);
                }
            }
            Self::Decoration {
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
                if clip.set_region(ctx, region.region()).is_some() {
                    node.render(ctx, clip);
                    clip.set_region(ctx, previous);
                }
            }
            _ => {}
        }
    }
    pub fn merge<'r>(&'r mut self, other: Self) {
        match self {
            RenderNode::Container(t_nodes) => match other {
                RenderNode::Container(nodes) => {
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
            Self::Decoration {
                background,
                border,
                node,
            } => {
                let this_node = node.as_mut();
                let this_border = border;
                let this_background = background;
                match other {
                    RenderNode::Decoration {
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
                            let merge = shape.region().crop(&a.region().merge(&region));
                            ctx.damage_region(&Texture::from(shape), merge, false);
                            b.render(ctx, clip.clipmask());
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
                    *self = other;
                    self.render(ctx, clip);
                }
            }
            RenderNode::Container(t_nodes) => match other {
                RenderNode::Container(nodes) => {
                    let len = nodes.len();
                    let clear = t_nodes.len() > nodes.len();
                    for (i, node) in nodes.into_iter().enumerate() {
                        if let Some(t_node) = t_nodes.get_mut(i) {
                            if let Err(region) = t_node.draw_merge(node, ctx, shape, clip) {
                                ctx.damage_region(&Texture::from(shape), clip.crop(&region), false);
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
                RenderNode::None => {}
                _ => {
                    let region = self.region().unwrap().merge(&other.region().unwrap());
                    ctx.damage_region(&Texture::from(shape), clip.crop(&region), false);
                    self.merge(other);
                    self.render(ctx, clip);
                }
            },
            RenderNode::Decoration {
                background,
                border,
                node,
            } => {
                let t_node = node.as_mut();
                let t_border = border;
                let t_background = background;

                match other {
                    RenderNode::Decoration {
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
                                ctx.damage_region(
                                    &Texture::from(shape),
                                    clip.crop(&instruction.region().merge(&region)),
                                    false,
                                );
                                instruction.render(ctx, clip.clipmask());
                                self.render(ctx, clip);
                            };
                        } else if let Some(region) = node.region() {
                            let dec = border.as_ref().unwrap_or(&background);
                            let t_dec = t_border.as_ref().unwrap_or(&t_background);
                            let contains = shape.contains(t_dec);
                            let merge = t_dec
                                .region()
                                .merge(&region)
                                .merge(&dec.region());
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
                        } else {
                            return Err(self.region().unwrap());
                        }
                    }
                    RenderNode::None => {}
                    _ => {
                        let region = t_border
                            .as_ref()
                            .unwrap_or(&t_background)
                            .region()
                            .merge(node.region().as_ref().unwrap())
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
                        clip.set_region(ctx, t_region.region());
                        let clip_region = Instruction::new(
                            t_region.transform,
                            shape.primitive.merge(t_region.primitive.clone()),
                        );
                        if let Err(region) = t_node.draw_merge(*node, ctx, &clip_region, clip) {
                            ctx.damage_region(&Texture::from(shape), clip.crop(&region), false);
                            t_node.render(ctx, clip);
                        }
                        clip.set_region(ctx, previous);
                    }
                    RenderNode::None => {}
                    _ => {
                        let region = t_region.region().merge(&other.region().unwrap());
                        ctx.damage_region(&Texture::from(shape), clip.crop(&region), false);
                        self.merge(other);
                        self.render(ctx, clip);
                    }
                }

                clip.set_region(ctx, previous);
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
        if let Some(rect) = Rect::from_xywh(r.x, r.y, r.width, r.height) {
            rect
        } else {
            panic!("{:?}", r)
        }
    }
}

impl Default for Region {
    fn default() -> Self {
        Region::new(0., 0., 0., 0.)
    }
}

impl Region {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Region {
        Region {
            x,
            y,
            width: width.max(0.),
            height: height.max(0.),
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
    /// Returns the region other instersect Self
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
    pub fn substract(&self, other: Self) -> [Self; 4] {
        let crop = self.crop(&other);
        [
        	Region::new(
            	self.x,
            	self.y,
            	crop.x - self.x,
            	self.height
        	),
        	Region::new(
            	crop.x + crop.width,
            	self.y,
            	self.x + self.width - crop.x - crop.width,
            	self.height
        	),
            Region::new(
                crop.x,
                self.y,
                self.width - crop.width,
                crop.y,
            ),
            Region::new(
                crop.x,
                crop.y + crop.height,
                self.width - crop.width,
                self.x - self.height - crop.y - crop.height,
            ),
        ]
    }
    /// Combines two region into a single that occupies the space of both
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
    pub fn from_transform(transform: Transform, width: f32, height: f32) -> Self {
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
    /// Checks if a point fits in a Region
    pub fn contains(&self, x: f32, y: f32) -> bool {
        self.x <= x && x - self.x < self.width && self.y <= y && y - self.y < self.height
    }
    pub fn scale(&self, sx: f32, sy: f32) -> Self {
        Self::new(self.x * sx, self.y * sy, self.width * sx, self.height * sy)
    }
    pub fn rect(&self) -> (Transform, Rectangle) {
        (
            Transform::from_translate(self.x, self.y),
            Rectangle::new(self.width, self.height),
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
