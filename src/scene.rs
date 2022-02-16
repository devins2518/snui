use crate::widgets::shapes::rectangle::BorderedRectangle;
use crate::*;
use context::DrawContext;
use std::rc::Rc;
use tiny_skia::*;

use cache::image::RawImage as Image;
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

#[derive(Debug, Clone, PartialEq)]
pub enum Texture {
    Transparent,
    Image(Image),
    Color(Color),
}

impl Texture {
    pub fn is_transparent(&self) -> bool {
        match &self {
            Texture::Transparent => true,
            Texture::Image(_) => false,
            Texture::Color(color) => color.eq(&tiny_skia::Color::TRANSPARENT),
        }
    }
    pub fn is_opaque(&self) -> bool {
        match &self {
            Texture::Transparent => false,
            Texture::Image(_) => false,
            Texture::Color(color) => color.is_opaque(),
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
        Rect::from_xywh(r.x, r.y, r.width, r.height).expect(format!("{:?}", r).as_str())
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
            Region::new(self.x, self.y, crop.x - self.x, self.height),
            Region::new(
                crop.x + crop.width,
                self.y,
                self.x + self.width - crop.x - crop.width,
                self.height,
            ),
            Region::new(crop.x, self.y, crop.width, crop.y - self.y),
            Region::new(
                crop.x,
                crop.y + crop.height,
                crop.width,
                self.y + self.height - crop.y - crop.height,
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
    pub fn is_empty(&self) -> bool {
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

#[derive(Debug, Clone, PartialEq)]
pub enum Primitive {
    Rectangle(Rectangle),
    Label(Label),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PrimitiveRef<'p> {
    Rectangle(&'p Rectangle),
    Label(&'p Label),
}

impl Deref for Primitive {
    type Target = dyn Drawable;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Rectangle(rect) => rect,
            Self::Label(label) => label,
        }
    }
}

impl<'p> Deref for PrimitiveRef<'p> {
    type Target = dyn Drawable;
    fn deref(&self) -> &Self::Target {
        match *self {
            Self::Rectangle(rect) => rect,
            Self::Label(label) => label,
        }
    }
}

impl<'p> From<&'p Primitive> for PrimitiveRef<'p> {
    fn from(primitive: &'p Primitive) -> Self {
        match primitive {
            Primitive::Label(label) => PrimitiveRef::Label(label),
            Primitive::Rectangle(rect) => PrimitiveRef::Rectangle(rect),
        }
    }
}

impl<'p> From<PrimitiveRef<'p>> for Primitive {
    fn from(p_ref: PrimitiveRef<'p>) -> Self {
        match p_ref {
            PrimitiveRef::Label(label) => Primitive::Label(label.clone()),
            PrimitiveRef::Rectangle(rect) => Primitive::Rectangle(rect.clone()),
        }
    }
}

// The current stack of background.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Background<'t, 'b> {
    pub(crate) rectangle: &'t Rectangle,
    pub(crate) previous: Option<&'b Background<'t, 'b>>,
    pub(crate) region: Region,
}

impl<'t, 'b> Background<'t, 'b> {
    pub fn new(rectangle: &'t Rectangle) -> Self {
        Background {
            rectangle,
            region: Region::new(0., 0., rectangle.width(), rectangle.height()),
            previous: None,
        }
    }
    pub fn texture(&self) -> &Texture {
        &self.rectangle.texture
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RenderNode {
    None,
    Primitive {
        coords: Coords,
        primitive: Primitive,
    },
    Container {
        bound: Region,
        cursor: usize,
        children: Vec<RenderNode>,
    },
    Clip {
        bound: Region,
        child: Rc<RenderNode>,
    },
    Background {
        coords: Coords,
        background: Rectangle,
        child: Rc<RenderNode>,
    },
    Border {
        coords: Coords,
        border: BorderedRectangle,
        child: Rc<RenderNode>,
    },
}

impl RenderNode {
    pub fn render(
        &self,
        ctx: &mut DrawContext,
        transform: tiny_skia::Transform,
        clip: Option<&Region>,
    ) {
        match self {
            RenderNode::None => {}
            RenderNode::Primitive { coords, primitive } => {
                let transform = ctx.transform().pre_translate(coords.x, coords.y);
                primitive.draw(ctx, transform);
            }
            RenderNode::Clip { bound, child } => {
                ctx.set_clip(*bound);
                child.render(ctx, transform, clip);
                if let Some(region) = clip {
                    ctx.reset_clip(*region);
                }
            }
            RenderNode::Container { children, .. } => {
                for child in children {
                    child.render(ctx, transform, clip)
                }
            }
            RenderNode::Background {
                coords,
                background,
                child,
            } => {
                let transform = ctx.transform().pre_translate(coords.x, coords.y);
                background.draw(ctx, transform);
                child.render(ctx, transform, clip);
            }
            RenderNode::Border {
                coords,
                border,
                child,
            } => {
                let transform = ctx.transform().pre_translate(coords.x, coords.y);
                border.draw(ctx, transform);
                child.render(ctx, transform, clip);
            }
        }
    }
}

impl RenderNode {
    fn primitive(coords: Coords, primitive: impl Into<Primitive>) -> Self {
        RenderNode::Primitive {
            coords,
            primitive: primitive.into(),
        }
    }
    fn background(coords: Coords, background: Rectangle, child: RenderNode) -> Self {
        RenderNode::Background {
            coords,
            background,
            child: Rc::new(child),
        }
    }
    fn border(coords: Coords, border: BorderedRectangle, child: RenderNode) -> Self {
        RenderNode::Border {
            coords,
            border,
            child: Rc::new(child),
        }
    }
    fn clip(coords: Coords, size: Size, child: RenderNode) -> Self {
        RenderNode::Clip {
            bound: Region::new(coords.x, coords.y, size.width, size.height),
            child: Rc::new(child),
        }
    }
    /// Create a new container
    fn container(coords: Coords, size: Size) -> Self {
        RenderNode::Container {
            bound: Region::new(coords.x, coords.y, size.width, size.height),
            cursor: 0,
            children: Vec::new(),
        }
    }
    pub fn region(&self) -> Option<Region> {
        match self {
            RenderNode::Clip { bound, .. } => Some(*bound),
            RenderNode::Container { bound, .. } => Some(*bound),
            RenderNode::Background {
                coords, background, ..
            } => {
                let (tl, tr, br, bl) = background.radius;
                Some(Region::new(
                    coords.x + tl.max(bl),
                    coords.y + tl.max(tr),
                    background.width - tr.max(br),
                    background.height - bl.max(br),
                ))
            }
            RenderNode::Primitive { coords, primitive } => Some(Region::new(
                coords.x,
                coords.y,
                primitive.width(),
                primitive.height(),
            )),
            RenderNode::Border { coords, border, .. } => Some(Region::new(
                coords.x,
                coords.y,
                border.width(),
                border.height(),
            )),
            _ => None,
        }
    }
}

pub struct Scene<'s, 'c, 'b> {
    pending_damage: bool,
    pub(crate) coords: Coords,
    clip: Option<Region>,
    node: &'s mut RenderNode,
    background: Background<'b, 's>,
    pub(crate) context: &'s mut DrawContext<'c>,
}

impl<'s, 'c, 'b> Drop for Scene<'s, 'c, 'b> {
    fn drop(&mut self) {
        if let Some(region) = self.clip {
            self.context.reset_clip(region)
        }
        match self.node {
            RenderNode::Container { cursor, .. } => {
                *cursor = 0;
            }
            _ => {}
        }
    }
}

impl<'s, 'c, 'b> AsRef<RenderNode> for Scene<'s, 'c, 'b> {
    fn as_ref(&self) -> &RenderNode {
        &*self.node
    }
}

impl<'s, 'c, 'b> Scene<'s, 'c, 'b> {
    pub fn new(
        node: &'s mut RenderNode,
        context: &'s mut DrawContext<'c>,
        rectangle: &'b Rectangle,
    ) -> Self {
        Self {
            pending_damage: false,
            coords: Coords::default(),
            clip: None,
            node,
            context,
            background: Background {
                previous: None,
                rectangle,
                region: Region::new(0., 0., rectangle.width, rectangle.height),
            },
        }
    }
    pub fn shift(mut self, x: f32, y: f32) -> Self {
        self.coords.x += x.round();
        self.coords.y += y.round();
        self
    }
    pub fn next_inner_with_damage<'n>(
        &'n mut self,
        pending_damage: bool,
    ) -> Option<Scene<'n, 'c, 'b>>
    where
        's: 'n,
    {
        match self.node {
            RenderNode::Background { child, .. } => Some(Scene {
                pending_damage,
                clip: self.clip,
                coords: self.coords,
                node: Rc::get_mut(child)?,
                background: self.background.clone(),
                context: self.context,
            }),
            RenderNode::Clip { child, .. } => Some(Scene {
                pending_damage,
                clip: self.clip,
                coords: self.coords,
                node: Rc::get_mut(child)?,
                background: self.background.clone(),
                context: self.context,
            }),
            RenderNode::Border { child, .. } => Some(Scene {
                pending_damage,
                clip: self.clip,
                coords: self.coords,
                node: Rc::get_mut(child)?,
                background: self.background.clone(),
                context: self.context,
            }),
            _ => None,
        }
    }
    pub fn next_inner<'n>(&'n mut self) -> Option<Scene<'n, 'c, 'b>>
    where
        's: 'n,
    {
        self.next_inner_with_damage(self.pending_damage)
    }
    pub fn next<'n>(&'n mut self) -> Option<Scene<'n, 'c, 'b>>
    where
        's: 'n,
    {
        match self.node {
            RenderNode::Container {
                cursor, children, ..
            } => {
                let node = children.get_mut(*cursor)?;
                *cursor += 1;
                Some(Scene {
                    clip: self.clip,
                    pending_damage: self.pending_damage,
                    coords: self.coords,
                    node,
                    background: self.background.clone(),
                    context: self.context,
                })
            }
            _ => None,
        }
    }
    /// Appends a new node to a container.
    pub fn append_node<'a>(&'a mut self, node: RenderNode, size: Size) -> Option<Scene<'a, 'c, 'b>>
    where
        's: 'a,
    {
        match self.node {
            RenderNode::Container {
                cursor,
                children,
                bound,
            } => {
                *cursor = children.len() + 1;
                bound.width = size.width;
                bound.height = size.height;
                children.push(node);
                Some(Scene {
                    clip: None,
                    pending_damage: true,
                    coords: self.coords,
                    node: children.last_mut()?,
                    background: self.background.clone(),
                    context: self.context,
                })
            }
            _ => {
                *self.node = RenderNode::container(self.coords, size);
                self.append_node(node, size)
            }
        }
    }
    pub fn truncate(&mut self, size: usize) {
        match self.node {
            RenderNode::Container {
                cursor, children, ..
            } => {
                children.truncate(size);
                *cursor = (*cursor).clamp(0, size - 1);
            }
            _ => {}
        }
    }
    pub fn apply_border<'n>(&'n mut self, rect: &BorderedRectangle) -> Option<Scene<'n, 'c, 'b>>
    where
        's: 'n,
    {
        let mut pending_damage = self.pending_damage;
        let region = Region::new(self.coords.x, self.coords.y, rect.width(), rect.height());
        let transform = self.context.transform();
        match self.node {
            RenderNode::Border { border, coords, .. } => {
                if self.pending_damage || border.ne(&rect) || self.coords.ne(&&coords) {
                    pending_damage = true;
                    *border = rect.clone();
                }
                if pending_damage {
                    if !self.pending_damage {
                        self.context.damage_region(
                            &self.background,
                            region.merge(&Region::new(
                                coords.x,
                                coords.y,
                                border.width(),
                                border.height(),
                            )),
                        );
                    }
                    border.draw(
                        self.context,
                        transform.pre_translate(self.coords.x, self.coords.y),
                    );
                }
                *coords = self.coords;
                self.next_inner_with_damage(pending_damage)
            }
            _ => {
                pending_damage = true;
                let merge = self
                    .node
                    .region()
                    .map(|inner| inner.merge(&region))
                    .unwrap_or(region);
                if !self.pending_damage {
                    // The space occupied by the previous node and the new one is cleaned.
                    self.context.damage_region(&self.background, merge);
                }
                rect.draw(
                    self.context,
                    transform.pre_translate(self.coords.x, self.coords.y),
                );
                // We replace the invalidated node.
                *self.node = RenderNode::border(self.coords, rect.clone(), RenderNode::None);
                self.next_inner_with_damage(pending_damage)
            }
        }
    }
    /// Applies the background and resolve damage before return the scene of the child.
    pub fn apply_background<'n>(&'n mut self, rect: &'b Rectangle) -> Option<Scene<'n, 'c, 'b>>
    where
        's: 'n,
    {
        let mut pending_damage = self.pending_damage;
        let region = Region::new(self.coords.x, self.coords.y, rect.width, rect.height);
        let transform = self.context.transform();
        match self.node {
            RenderNode::Background {
                background,
                child,
                coords,
            } => {
                if self.coords.ne(&&coords) {
                    pending_damage = true;
                    *coords = self.coords;
                }
                if background.ne(&rect) {
                    pending_damage = true;
                    *background = rect.clone();
                }
                let t_background = match rect.texture {
                    Texture::Transparent => self.background,
                    _ => Background {
                        previous: (!rect.texture.is_opaque()).then(|| &self.background),
                        region,
                        rectangle: &rect,
                    },
                };
                if pending_damage {
                    if !self.pending_damage {
                        self.context.damage_region(
                            &self.background,
                            region.merge(&Region::new(
                                coords.x,
                                coords.y,
                                background.width(),
                                background.height(),
                            )),
                        );
                    }
                    background.draw(
                        self.context,
                        transform.pre_translate(self.coords.x, self.coords.y),
                    );
                }
                Some(Scene {
                    clip: None,
                    pending_damage,
                    background: t_background,
                    coords: self.coords,
                    node: Rc::get_mut(child)?,
                    context: self.context,
                })
            }
            _ => {
                pending_damage = true;
                let merge = self
                    .node
                    .region()
                    .map(|inner| inner.merge(&region))
                    .unwrap_or(region);
                if !self.pending_damage {
                    // The space occupied by the previous node and the new one is cleaned.
                    self.context.damage_region(&self.background, merge);
                }
                rect.draw(
                    self.context,
                    transform.post_translate(self.coords.x, self.coords.y),
                );
                // We replace the invalidated node.
                *self.node = RenderNode::background(self.coords, rect.clone(), RenderNode::None);
                self.next_inner_with_damage(pending_damage)
            }
        }
    }
    pub fn apply_clip<'n>(&'n mut self, size: Size) -> Option<Scene<'n, 'c, 'b>>
    where
        's: 'n,
    {
        let mut pending_damage = self.pending_damage;
        let region = Region::new(self.coords.x, self.coords.y, size.width, size.height);
        match self.node {
            RenderNode::Clip { bound, .. } => {
                if bound.ne(&&region) || self.pending_damage {
                    pending_damage = true;
                    self.context
                        .damage_region(&self.background, bound.merge(&region));
                    self.context.set_clip(*bound);
                    *bound = region;
                }
                self.next_inner_with_damage(pending_damage)
            }
            _ => {
                pending_damage = true;
                let merge = self
                    .node
                    .region()
                    .map(|inner| inner.merge(&region))
                    .unwrap_or(region);
                if !self.pending_damage {
                    // The space occupied by the previous node and the new one is cleaned.
                    self.context.damage_region(&self.background, merge);
                }
                self.context.set_clip(region);
                // We replace the invalidated node.
                *self.node = RenderNode::clip(self.coords, size, RenderNode::None);
                self.next_inner_with_damage(pending_damage)
            }
        }
    }
    pub fn push_primitive<P>(&mut self, primitive: &P)
    where
        for<'a> &'a P: Into<PrimitiveRef<'a>>,
    {
        let primitive_ref = primitive.into();
        let transform = self.context.transform();
        let region = Region::new(
            self.coords.x,
            self.coords.y,
            primitive_ref.width(),
            primitive_ref.height(),
        );
        match self.node {
            RenderNode::Primitive { coords, primitive } => {
                let mut pending_damage = self.pending_damage;
                let merge = region.merge(&Region::new(
                    coords.x,
                    coords.y,
                    primitive.width(),
                    primitive.height(),
                ));
                if coords.ne(&&self.coords) {
                    pending_damage = true;
                    *coords = self.coords;
                }
                if PrimitiveRef::from(&*primitive).ne(&primitive_ref) {
                    pending_damage = true;
                    *primitive = primitive_ref.into();
                }
                if pending_damage {
                    if !self.pending_damage {
                        self.context.damage_region(&self.background, merge);
                    }
                    primitive.draw(
                        self.context,
                        transform.pre_translate(self.coords.x, self.coords.y),
                    );
                }
            }
            _ => {
                let merge = self
                    .node
                    .region()
                    .map(|inner| inner.merge(&region))
                    .unwrap_or(region);
                if !self.pending_damage {
                    // The space occupied by the previous node and the new one is cleaned.
                    self.context.damage_region(&self.background, merge);
                }
                // We replace the invalidated node.
                primitive_ref.draw(
                    self.context,
                    transform.pre_translate(self.coords.x, self.coords.y),
                );
                *self.node = RenderNode::primitive(self.coords, Primitive::from(primitive_ref));
            }
        }
    }
}
