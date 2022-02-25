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
pub struct LinearGradient {
    pub orientation: Orientation,
    pub stops: Vec<GradientStop>,
    pub mode: SpreadMode,
}

impl LinearGradient {
    pub fn new(stops: Vec<GradientStop>) -> LinearGradient {
        LinearGradient {
            stops,
            mode: SpreadMode::Reflect,
            orientation: Orientation::Horizontal,
        }
    }
    pub fn mode(mut self, mode: SpreadMode) -> Self {
        self.mode = mode;
        self
    }
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Texture {
    Transparent,
    Image(Image),
    Color(Color),
    LinearGradient(LinearGradient),
}

impl Texture {
    pub fn is_transparent(&self) -> bool {
        !self.is_opaque()
    }
    pub fn is_opaque(&self) -> bool {
        match &self {
            Texture::Color(color) => color.is_opaque(),
            _ => false,
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

impl From<LinearGradient> for Texture {
    fn from(gradient: LinearGradient) -> Self {
        Self::LinearGradient(gradient)
    }
}

impl From<Image> for Texture {
    fn from(raw: Image) -> Self {
        Texture::Image(raw)
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
    pub fn from_size(coords: Coords, size: Size) -> Region {
        Self::new(coords.x, coords.y, size.width, size.height)
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
    pub fn top_anchor(&self) -> Coords {
        Coords::new(self.x + self.width / 2., self.y)
    }
    pub fn left_anchor(&self) -> Coords {
        Coords::new(self.x, self.y + self.height / 2.)
    }
    pub fn bottom_anchor(&self) -> Coords {
        Coords::new(self.x + self.width / 2., self.y + self.height)
    }
    pub fn right_anchor(&self) -> Coords {
        Coords::new(self.x + self.width, self.y + self.height / 2.)
    }
    pub fn start(&self) -> Coords {
        Coords::new(self.x, self.y)
    }
    pub fn end(&self) -> Coords {
        Coords::new(self.x + self.width, self.y + self.height)
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

pub trait Merge<T> {
    fn merge(&mut self, other: T);
}

#[derive(Debug, Clone, PartialEq)]
pub enum Primitive {
    Label(Label),
    Rectangle(Rectangle),
    BorderedRectangle(BorderedRectangle),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PrimitiveRef<'p> {
    Label(&'p Label),
    Rectangle(&'p Rectangle),
    BorderedRectangle(&'p BorderedRectangle),
}

impl Deref for Primitive {
    type Target = dyn Drawable;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Rectangle(rect) => rect,
            Self::Label(label) => label,
            Self::BorderedRectangle(rect) => rect,
        }
    }
}

impl<'p> Deref for PrimitiveRef<'p> {
    type Target = dyn Drawable;
    fn deref(&self) -> &Self::Target {
        match *self {
            Self::Label(label) => label,
            Self::Rectangle(rect) => rect,
            Self::BorderedRectangle(rect) => rect,
        }
    }
}

impl<'p> From<&'p Primitive> for PrimitiveRef<'p> {
    fn from(primitive: &'p Primitive) -> Self {
        match primitive {
            Primitive::Label(label) => PrimitiveRef::Label(label),
            Primitive::Rectangle(rect) => PrimitiveRef::Rectangle(rect),
            Primitive::BorderedRectangle(rect) => PrimitiveRef::BorderedRectangle(rect),
        }
    }
}

impl<'p> From<PrimitiveRef<'p>> for Primitive {
    fn from(p_ref: PrimitiveRef<'p>) -> Self {
        match p_ref {
            PrimitiveRef::Label(label) => Primitive::Label(label.clone()),
            PrimitiveRef::Rectangle(rect) => Primitive::Rectangle(rect.clone()),
            PrimitiveRef::BorderedRectangle(rect) => Primitive::BorderedRectangle(rect.clone()),
        }
    }
}

impl<'p> Merge<PrimitiveRef<'p>> for Primitive {
    fn merge(&mut self, other: PrimitiveRef<'p>) {
        if let Self::Label(label) = self {
            if let PrimitiveRef::Label(other) = other {
                label.text.replace_range(0.., other.as_str());
                for (t_font, font) in label.fonts.iter_mut().zip(other.fonts.iter()) {
                    if t_font.ne(&font) {
                        t_font.name.replace_range(0.., &font.name);
                        t_font.style = font.style;
                    }
                }
                label.settings = other.settings;
                label.font_size = other.font_size;
                label.color = other.color;
                label.size = other.size;
                return;
            }
        }
        *self = Primitive::from(other);
    }
}

// The current stack of background.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Background<'t, 'b> {
    pub(crate) coords: Coords,
    pub(crate) rectangle: &'t Rectangle,
    pub(crate) previous: Option<&'b Background<'t, 'b>>,
}

impl<'t, 'b> Deref for Background<'t, 'b> {
    type Target = Rectangle;
    fn deref(&self) -> &Self::Target {
        self.rectangle
    }
}

impl<'t, 'b> Background<'t, 'b> {
    pub fn new(rectangle: &'t Rectangle) -> Self {
        Background {
            coords: Default::default(),
            rectangle,
            // region: Region::new(0., 0., rectangle.width(), rectangle.height()),
            previous: None,
        }
    }
    pub fn texture(&self) -> &Texture {
        &self.rectangle.texture
    }
    pub fn region(&self) -> Region {
        Region::new(
            self.coords.x,
            self.coords.y,
            self.rectangle.width(),
            self.rectangle.height(),
        )
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
        bounds: Region,
        cursor: usize,
        children: Vec<RenderNode>,
    },
    Clip {
        bounds: Region,
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
            RenderNode::Clip { bounds, child } => {
                ctx.set_clip(bounds);
                child.render(ctx, transform, clip);
                if let Some(region) = clip {
                    ctx.set_clip(region);
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
            bounds: Region::new(coords.x, coords.y, size.width, size.height),
            child: Rc::new(child),
        }
    }
    /// Create a new container
    fn container(coords: Coords, size: Size) -> Self {
        RenderNode::Container {
            bounds: Region::new(coords.x, coords.y, size.width, size.height),
            cursor: 0,
            children: Vec::new(),
        }
    }
    pub fn region(&self) -> Option<Region> {
        match self {
            RenderNode::Clip { bounds, .. } => Some(*bounds),
            RenderNode::Container { bounds, .. } => Some(*bounds),
            RenderNode::Background {
                coords, background, ..
            } => Some(Region::new(
                coords.x,
                coords.y,
                background.width,
                background.height,
            )),
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

/// A scene is an iterator over the RenderNode.
///
/// It provides contextual information necessary to damage the scene and helper methods to build it.
pub struct Scene<'s, 'c, 'b> {
    damage: bool,
    coords: Coords,
    clip: Option<&'s Region>,
    node: &'s mut RenderNode,
    background: Background<'b, 's>,
    pub(crate) context: &'s mut DrawContext<'c>,
}

impl<'s, 'c, 'b> Drop for Scene<'s, 'c, 'b> {
    fn drop(&mut self) {
        match self.node {
            RenderNode::Container { cursor, .. } => {
                // The index of the current node is set to 0
                *cursor = 0;
            }
            RenderNode::Clip { .. } => {
                // Reset the clipmask to the size of its background
                match self.clip {
                    Some(region) => self.context.set_clip(region),
                    None => {
                        if let Some(clipmask) = self.context.clipmask.as_mut() {
                            clipmask.clear();
                        }
                    }
                }
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
            damage: false,
            coords: Coords::default(),
            clip: None,
            node,
            context,
            background: Background::new(rectangle),
        }
    }
    /// Move the scene coords by a delta
    pub fn translate(mut self, x: f32, y: f32) -> Self {
        self.coords.x += x.round();
        self.coords.y += y.round();
        self
    }
    /// Returns the absolute position of the scene.
    pub fn position(&self) -> Coords {
        self.coords
    }
    fn next_inner_with_damage<'n>(&'n mut self, damage: bool) -> Option<Scene<'n, 'c, 'b>>
    where
        's: 'n,
    {
        match self.node {
            RenderNode::Background { child, .. } => Some(Scene {
                damage,
                clip: self.clip,
                coords: self.coords,
                node: Rc::get_mut(child)?,
                background: self.background,
                context: self.context,
            }),
            RenderNode::Clip { child, .. } => Some(Scene {
                damage,
                clip: self.clip,
                coords: self.coords,
                node: Rc::get_mut(child)?,
                background: self.background,
                context: self.context,
            }),
            RenderNode::Border { child, .. } => Some(Scene {
                damage,
                clip: self.clip,
                coords: self.coords,
                node: Rc::get_mut(child)?,
                background: self.background,
                context: self.context,
            }),
            _ => None,
        }
    }
    pub fn next_inner<'n>(&'n mut self) -> Option<Scene<'n, 'c, 'b>>
    where
        's: 'n,
    {
        self.next_inner_with_damage(self.damage)
    }
    pub fn quick_next<'n>(&'n mut self) -> Option<Scene<'n, 'c, 'b>>
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
                    damage: self.damage,
                    coords: self.coords,
                    node,
                    background: self.background,
                    context: self.context,
                })
            }
            _ => None,
        }
    }
    /// Automatically appends a new node the container comes to its limit
    pub fn next<'n>(&'n mut self, size: Size) -> Option<Scene<'n, 'c, 'b>>
    where
        's: 'n,
    {
        match self.node {
            RenderNode::Container {
                children,
                cursor,
                bounds,
            } => {
                if children.len() == *cursor {
                    self.append_node(RenderNode::None, size)
                } else {
                    if (self.coords.x != bounds.x || self.coords.y != bounds.y) && !self.damage {
                        let region = Region::from_size(self.coords, size);
                        self.damage = true;
                        self.context.clear(&self.background, region.merge(bounds));
                        *bounds = region;
                    }
                    bounds.x = self.coords.x;
                    bounds.y = self.coords.y;
                    self.quick_next()
                }
            }
            _ => self.append_node(RenderNode::None, size),
        }
    }
    pub fn damage_state(&self) -> bool {
        self.damage
    }
    /// Puts the scene into a damaged state.
    /// The damage will be passed down to all its child.
    pub fn damage(mut self, size: Size) -> Self {
        if !self.damage {
            self.damage = true;
            let region = Region::from_size(self.coords, size);
            let merge = self
                .node
                .region()
                .map(|inner| inner.merge(&region))
                .unwrap_or(region);
            self.context.clear(&self.background, merge);
        }
        self
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
                bounds,
            } => {
                *cursor = children.len() + 1;
                bounds.width = size.width;
                bounds.height = size.height;
                children.push(node);
                Some(Scene {
                    clip: None,
                    damage: true,
                    coords: self.coords,
                    node: children.last_mut()?,
                    background: self.background,
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
    /// Applies a border to the scene graph and diffs it in the process
    pub fn apply_border<'n>(&'n mut self, rect: &BorderedRectangle) -> Option<Scene<'n, 'c, 'b>>
    where
        's: 'n,
    {
        let region = Region::new(self.coords.x, self.coords.y, rect.width(), rect.height());
        let transform = self.context.transform();
        match self.node {
            RenderNode::Border { border, coords, .. } => {
                let damage_border = border.ne(&rect);
                let damage_coords = self.coords.ne(coords);
                if damage_border || damage_coords || self.damage {
                    if !self.damage {
                        let merge =
                            Region::new(coords.x, coords.y, border.width(), border.height())
                                .merge(&region);
                        self.context.clear(&self.background, merge);
                        self.damage = true;
                    }
                    if damage_border {
                        *border = rect.clone();
                    }
                    border.draw(
                        self.context,
                        transform.pre_translate(self.coords.x, self.coords.y),
                    );
                }
                *coords = self.coords;
                self.next_inner_with_damage(damage_border || damage_coords || self.damage)
            }
            _ => {
                let merge = self
                    .node
                    .region()
                    .map(|inner| inner.merge(&region))
                    .unwrap_or(region);
                if !self.damage {
                    // The space occupied by the previous node and the new one is cleaned.
                    self.context.clear(&self.background, merge);
                }
                rect.draw(
                    self.context,
                    transform.pre_translate(self.coords.x, self.coords.y),
                );
                // We replace the invalidated node.
                *self.node = RenderNode::border(self.coords, rect.clone(), RenderNode::None);
                self.next_inner_with_damage(true)
            }
        }
    }
    /// Applies the background to the scene.
    /// The background is effective for all scenes derived from this one.
    pub fn apply_background<'n>(&'n mut self, rect: &'b Rectangle) -> Option<Scene<'n, 'c, 'b>>
    where
        's: 'n,
    {
        let region = Region::new(self.coords.x, self.coords.y, rect.width, rect.height);
        let transform = self.context.transform();
        match self.node {
            RenderNode::Background {
                background,
                child,
                coords,
            } => {
                let t_background = match rect.texture {
                    Texture::Transparent => self.background,
                    _ => Background {
                        coords: self.coords,
                        previous: (rect.texture.is_transparent()).then(|| &self.background),
                        rectangle: rect,
                    },
                };

                let damage_bg = background.ne(&rect);
                let damage_coords = self.coords.ne(coords);
                if damage_bg || damage_coords || self.damage {
                    if !self.damage {
                        let merge = Region::new(
                            coords.x,
                            coords.y,
                            background.width(),
                            background.height(),
                        )
                        .merge(&region);
                        self.context.clear(&self.background, merge);
                        self.damage = true;
                    }
                    if damage_bg {
                        *background = rect.clone();
                    }
                    background.draw(
                        self.context,
                        transform.pre_translate(self.coords.x, self.coords.y),
                    );
                }

                *coords = self.coords;
                Some(Scene {
                    clip: None,
                    damage: damage_bg || damage_coords || self.damage,
                    background: t_background,
                    coords: self.coords,
                    node: Rc::get_mut(child)?,
                    context: self.context,
                })
            }
            _ => {
                let merge = self
                    .node
                    .region()
                    .map(|inner| inner.merge(&region))
                    .unwrap_or(region);
                if !self.damage {
                    // The space occupied by the previous node and the new one is cleaned.
                    self.context.clear(&self.background, merge);
                }
                rect.draw(
                    self.context,
                    transform.post_translate(self.coords.x, self.coords.y),
                );
                // We replace the invalidated node.
                *self.node = RenderNode::background(self.coords, rect.clone(), RenderNode::None);
                self.next_inner_with_damage(true)
            }
        }
    }
    pub fn apply_clip<'n>(&'n mut self, size: Size) -> Option<Scene<'n, 'c, 'b>>
    where
        's: 'n,
    {
        let region = Region::new(self.coords.x, self.coords.y, size.width, size.height);
        match self.node {
            RenderNode::Clip { bounds, child } => {
                if bounds.ne(&&region) && !self.damage {
                    self.context.clear(&self.background, region.merge(bounds));
                    self.damage = true;
                }
                *bounds = region;
                let clip = self.clip.map(|clip| clip.crop(bounds)).unwrap_or(*bounds);
                self.context.set_clip(&clip);
                Some(Scene {
                    damage: self.damage,
                    clip: Some(bounds),
                    background: self.background,
                    coords: self.coords,
                    node: Rc::get_mut(child)?,
                    context: self.context,
                })
            }
            _ => {
                let merge = self
                    .node
                    .region()
                    .map(|inner| inner.merge(&region))
                    .unwrap_or(region);
                if !self.damage {
                    // The space occupied by the previous node and the new one is cleaned.
                    self.context.clear(&self.background, merge);
                }
                self.context.set_clip(&region);
                // We replace the invalidated node.
                *self.node = RenderNode::clip(self.coords, size, RenderNode::None);
                self.next_inner_with_damage(true)
            }
        }
    }
    pub fn insert_primitive<P>(&mut self, primitive: &P)
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
                let damage_coords = self.coords.ne(coords);
                let damage_prim = PrimitiveRef::from(&*primitive).ne(&primitive_ref);
                if damage_coords || damage_prim || self.damage {
                    let merge = region.merge(&Region::new(
                        coords.x,
                        coords.y,
                        primitive.width(),
                        primitive.height(),
                    ));
                    if damage_prim {
                        primitive.merge(primitive_ref);
                    }
                    if !self.damage {
                        self.context.clear(&self.background, merge);
                    }
                    primitive.draw(
                        self.context,
                        transform.pre_translate(self.coords.x, self.coords.y),
                    );
                    *coords = self.coords;
                }
            }
            _ => {
                let merge = self
                    .node
                    .region()
                    .map(|inner| inner.merge(&region))
                    .unwrap_or(region);
                if !self.damage {
                    // The space occupied by the previous node and the new one is cleaned.
                    self.context.clear(&self.background, merge);
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
