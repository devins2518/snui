use crate::*;
use context::DrawContext;
use std::rc::Rc;
use tiny_skia::*;
use widgets::blend;

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

#[derive(Debug, Clone)]
pub enum Texture {
    Transparent,
    Image(Image),
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
        Texture::Image(Region::new(0., 0., 0., 0.), image.into())
    }
}

impl From<Primitive> for Texture {
    fn from(primitive: Primitive) -> Self {
        primitive.get_texture()
    }
}

impl From<&Primitive> for Texture {
    fn from(primitive: &Primitive) -> Self {
        primitive.get_texture()
    }
}

impl Texture {
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
    fn set_texture(&self, background: Texture) -> Self {
        match self {
            Self::Rectangle(rectangle) => rectangle.set_texture(background),
            Self::Label(_) => Rectangle::new(self.width(), self.height())
                .background(background)
                .into(),
            Self::Other(primitive) => primitive.set_texture(background),
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
            Self::Label(label) => {
                let scale = transform.sy;
                if scale > 1. {
                    let mut settings = label.settings;
                    settings.max_width = settings.max_width.map(|width| width * scale);
                    settings.max_height = settings.max_height.map(|height| height * scale);
                    let label = LabelRef {
                        color: label.color,
                        font_size: transform.sy * label.font_size,
                        fonts: &label.fonts,
                        settings: &settings,
                        text: &label.text,
                    };
                    ctx.draw_label(transform.tx, transform.ty, label, clip)
                } else {
                    ctx.draw_label(transform.tx, transform.ty, label.as_ref(), clip)
                };
            }
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
        other.set_texture(background)
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
    fn region(&self) -> Region {
        Region::new(
            self.transform.tx,
            self.transform.ty,
            self.primitive.width() * self.transform.sx,
            self.primitive.height() * self.transform.sy,
        )
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
    Container {
        bound: Region,
        children: Vec<RenderNode>,
    },
    Clip {
        bound: Region,
        node: Box<RenderNode>,
    },
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

pub struct Scene {
    transform: Transform,
    primitive: Primitive,
    clipmask: Option<Region>,
}

impl Deref for Scene {
    type Target = Primitive;
    fn deref(&self) -> &Self::Target {
        &self.primitive
    }
}

impl DerefMut for Scene {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.primitive
    }
}

impl Scene {
    pub fn new(width: f32, height: f32) -> Scene {
        Scene {
            primitive: Rectangle::new(width, height).into(),
            transform: Transform::identity(),
            clipmask: None,
        }
    }
    pub fn transform(mut self, transform: Transform) -> Scene {
        self.transform = transform;
        self
    }
    fn contains(&self, region: &Region) -> bool {
        let region = region
            .relative_to(self.transform.tx, self.transform.ty)
            .scale(1. / self.transform.sx, 1. / self.transform.sy);
        Drawable::contains(&self.primitive, &region)
    }
    fn region(&self) -> Region {
        let inner = Region::from_transform(
            self.transform,
            self.primitive.width(),
            self.primitive.height(),
        );
        self.clipmask
            .as_ref()
            .map(|clip| clip.crop(&inner))
            .unwrap_or(inner)
    }
    fn fork(&self, transform: Transform, primitive: &Primitive) -> Self {
        Self {
            transform,
            clipmask: self.clipmask,
            primitive: self.primitive.merge(primitive.clone()),
        }
    }
    fn clip(&self, region: Region) -> Self {
        Self {
            clipmask: Some(
                region
                // self.clipmask
                //     .map(|inner| inner.crop(&region))
                //     .unwrap_or(region),
            ),
            transform: self.transform,
            primitive: self.primitive.clone(),
        }
    }
}

impl Geometry for RenderNode {
    fn width(&self) -> f32 {
        match self {
            Self::Clip { bound, .. } => bound.width,
            Self::Container { bound, .. } => bound.width,
            Self::Decoration { background, border, .. } => {
                border.as_ref()
                	.unwrap_or(background)
                	.primitive
                	.width()
            }
            Self::Instruction(instruction) => instruction.primitive.width(),
            Self::None => 0.
        }
    }
    fn height(&self) -> f32 {
        match self {
            Self::Clip { bound, .. } => bound.height,
            Self::Container { bound, .. } => bound.height,
            Self::Decoration { background, border, .. } => {
                border.as_ref()
                	.unwrap_or(background)
                	.primitive
                	.height()
            }
            Self::Instruction(instruction) => instruction.primitive.height(),
            Self::None => 0.
        }
    }
}

impl RenderNode {
    pub fn is_none(&self) -> bool {
        match self {
            Self::None => true,
            Self::Container { children, .. } => children.is_empty(),
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
    pub fn instruction(self) -> Option<Instruction> {
        match self {
            Self::Instruction(instruction) => Some(instruction),
            _ => None
        }
    }
    pub fn region(&self) -> Option<Region> {
        match self {
            Self::Clip { bound, .. } => Some(*bound),
            Self::Container { bound, .. } => Some(*bound),
            Self::Decoration {
                background, border, ..
            } => Some(border.as_ref().unwrap_or(background).region()),
            Self::Instruction(instruction) => Some(instruction.region()),
            Self::None => None,
        }
    }
    pub fn render(
        &self,
        ctx: &mut DrawContext,
        clipmask: &mut Option<ClipMask>,
        region: Option<Region>,
    ) {
        match self {
            Self::Instruction(instruction) => {
                let clipmask = clipmask
                    .as_ref()
                    .map(|c| if !c.is_empty() { Some(c) } else { None })
                    .flatten();
                let transform = instruction.transform.post_concat(ctx.transform());
                instruction
                    .primitive
                    .draw_with_transform_clip(ctx, transform, clipmask)
            }
            Self::Container { children, .. } => {
                for node in children {
                    node.render(ctx, clipmask, region);
                }
            }
            Self::Decoration {
                background,
                border,
                node,
            } => {
                let clipmask_ref = clipmask
                    .as_ref()
                    .map(|c| if !c.is_empty() { Some(c) } else { None })
                    .flatten();
                if let Some(border) = border.as_ref() {
                    let transform = border.transform.post_concat(ctx.transform());
                    border
                        .primitive
                        .draw_with_transform_clip(ctx, transform, clipmask_ref);
                }
                let transform = background.transform.post_concat(ctx.transform());
                background
                    .primitive
                    .draw_with_transform_clip(ctx, transform, clipmask_ref);
                node.render(ctx, clipmask, region);
            }
            Self::Clip { bound, node } => {
                let bound = bound.scale(ctx.transform().sx, ctx.transform().sy);
                if let Some(clipmask) = clipmask {
                    // Not reseting the previous state could cause issues
                    let mut pb = ctx.path_builder();
                    pb.push_rect(bound.x, bound.y, bound.width, bound.height);
                    let path = pb.finish().unwrap();
                    if clipmask.is_empty() {
                        clipmask.set_path(
                            ctx.width() as u32,
                            ctx.height() as u32,
                            &path,
                            FillRule::Winding,
                            false,
                        );
                    } else {
                        clipmask.intersect_path(&path, FillRule::Winding, false);
                    }
                    ctx.reset(path.clear());
                }
                node.render(ctx, clipmask, Some(bound));
                if let Some(clipmask) = clipmask {
                    if let Some(region) = region
                        .map(|region| {
                            if region.width >= ctx.width() && region.height >= ctx.height() {
                                None
                            } else {
                                Some(region)
                            }
                        })
                        .flatten()
                    {
                        let mut pb = ctx.path_builder();
                        pb.push_rect(region.x, region.y, region.width, region.height);
                        let path = pb.finish().unwrap();
                        clipmask.set_path(
                            ctx.width() as u32,
                            ctx.height() as u32,
                            &path,
                            FillRule::Winding,
                            false,
                        );
                        ctx.reset(path.clear());
                    } else {
                        clipmask.clear();
                    }
                }
            }
            _ => {}
        }
    }
    pub fn merge<'r>(&'r mut self, other: Self) {
        match self {
            RenderNode::Container { bound, children } => {
                let t_bound = bound;
                let t_children = children;
                match other {
                    RenderNode::Container { bound, children } => {
                        *t_bound = bound;
                        let len = children.len();
                        let clear = t_children.len() > children.len();
                        for (i, node) in children.into_iter().enumerate() {
                            if let Some(t_node) = t_children.get_mut(i) {
                                t_node.merge(node);
                            } else {
                                t_children.push(node)
                            }
                        }
                        if clear {
                            t_children.truncate(len);
                        }
                    }
                    RenderNode::None => {}
                    _ => {
                        *self = other;
                    }
                }
            }
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
            Self::Clip { bound, node } => {
                let t_bound = bound;
                let t_node = node.as_mut();
                match other {
                    Self::Clip { bound, node } => {
                        *t_bound = bound;
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
        scene: &Scene,
        clipmask: &mut Option<ClipMask>,
    ) -> Result<(), Region> {
        match self {
            RenderNode::Instruction(a) => match other {
                RenderNode::Instruction(ref b) => {
                    if b.ne(a) {
                        let region = b.region();
                        let merge = a.region().merge(&region);
                        if scene.contains(&region) || clipmask.is_some() {
                            ctx.damage_region(
                                &Texture::from(scene.deref()),
                                scene.region().crop(&merge),
                                false,
                            );
                            *self = other;
                            self.render(ctx, clipmask, None);
                        } else {
                            *self = other;
                            return Err(merge);
                        }
                    }
                }
                RenderNode::None => {}
                _ => {
                    let region = a.region().merge(&other.region().unwrap());
                    ctx.damage_region(
                        &Texture::from(scene.deref()),
                        scene.region().crop(&region),
                        false,
                    );
                    *self = other;
                    self.render(ctx, clipmask, None);
                }
            },
            RenderNode::None => {
                if let Some(region) = other.region() {
                    let crop = scene.region().crop(&region);
                    if scene.contains(&crop) {
                        ctx.damage_region(&Texture::from(scene.deref()), crop, false);
                        *self = other;
                        self.render(ctx, clipmask, None);
                    } else {
                        return Err(region);
                    }
                }
            }
            RenderNode::Container { bound, children } => {
                let t_bound = bound;
                let t_children = children;
                match other {
                    RenderNode::Container { bound, children } => {
                        let merge = t_bound.merge(&bound);
                        if !scene.contains(&merge) {
                            self.merge(RenderNode::Container { bound, children });
                            return Err(merge);
                        }
                        *t_bound = bound;
                        let len = t_children.len();
                        let clear = len > children.len();
                        let region = children
                            .into_iter()
                            .enumerate()
                            .filter_map(|(i, node)| {
                                let region;
                                if let Some(t_node) = t_children.get_mut(i) {
                                    region = t_node.draw_merge(node, ctx, scene, clipmask).err();
                                } else {
                                    let mut t_node = RenderNode::None;
                                    region = t_node.draw_merge(node, ctx, scene, clipmask).err();
                                    t_children.push(t_node);
                                }
                                region
                            })
                            .reduce(|merge, region| merge.merge(&region));
                        if clear {
                            t_children.truncate(len);
                        }
                        if let Some(region) = region {
                            return Err(region);
                        }
                    }
                    RenderNode::None => {}
                    _ => {
                        let region = self.region().unwrap().merge(&other.region().unwrap());
                        ctx.damage_region(
                            &Texture::from(scene.deref()),
                            scene.region().crop(&region),
                            false,
                        );
                        self.merge(other);
                        self.render(ctx, clipmask, None);
                    }
                }
            }
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
                            if let Err(region) = t_node.draw_merge(
                                *node,
                                ctx,
                                &scene.fork(background.transform, &background.primitive),
                                clipmask,
                            ) {
                                ctx.damage_region(
                                    &Texture::from(scene.deref()),
                                    scene.region().merge(&region),
                                    false,
                                );
                                self.render(ctx, clipmask, None);
                            };
                        } else if let Some(region) = node.region() {
                            let dec = border.as_ref().unwrap_or(&background);
                            let t_dec = t_border.as_ref().unwrap_or(&t_background);
                            let merge = t_dec.region().merge(&region).merge(&dec.region());
                            let contains = scene.contains(&merge);
                            t_node.merge(*node);
                            *t_border = border;
                            *t_background = background;
                            if !contains {
                                return Err(merge);
                            }
                            ctx.damage_region(
                                &Texture::from(scene.deref()),
                                scene.region().crop(&merge),
                                false,
                            );
                            self.render(ctx, clipmask, None);
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
                        ctx.damage_region(&Texture::from(scene.deref()), region, false);
                        self.merge(other);
                        self.render(ctx, clipmask, None);
                    }
                }
            }
            RenderNode::Clip { bound, node } => {
                let t_bound = bound;
                let t_node = node.as_mut();
                match other {
                    RenderNode::Clip { bound, node } => {
                        let transform = ctx.transform();
                        let merge = t_bound.merge(&bound);
                        if t_bound.ne(&&bound) {
                            if scene.contains(&merge) {
                                *t_bound = bound;
                                t_node.merge(*node);
                                ctx.damage_region(
                                    &Texture::from(scene.deref()),
                                    scene.region().crop(&merge),
                                    false,
                                );
                                self.render(ctx, clipmask, None);
                            } else {
                                t_node.merge(*node);
                                return Err(merge);
                            }
                        } else {
                            if let Some(clipmask) = clipmask {
                                let bound = bound.scale(transform.sx, transform.sy);
                                let mut pb = ctx.path_builder();
                                pb.push_rect(bound.x, bound.y, bound.width, bound.height);
                                let path = pb.finish().unwrap();
                                if clipmask.is_empty() {
                                    clipmask.set_path(
                                        ctx.width() as u32,
                                        ctx.height() as u32,
                                        &path,
                                        FillRule::Winding,
                                        false,
                                    );
                                } else {
                                    clipmask.intersect_path(&path, FillRule::Winding, false);
                                }
                                ctx.reset(path.clear());
                            }
                            if let Err(region) =
                                t_node.draw_merge(*node, ctx, &scene.clip(bound), clipmask)
                            {
                                let region = bound.crop(&region);
                                ctx.damage_region(&Texture::from(scene.deref()), region, false);
                                t_node.render(
                                    ctx,
                                    clipmask,
                                    Some(region.scale(transform.sx, transform.sy)),
                                );
                            }
                            *t_bound = bound;
                            if let Some(clipmask) = clipmask {
                                if let Some(region) = scene.clipmask {
                                    let region = region.scale(transform.sx, transform.sy);
                                    let mut pb = ctx.path_builder();
                                    pb.push_rect(region.x, region.y, region.width, region.height);
                                    let path = pb.finish().unwrap();
                                    clipmask.set_path(
                                        ctx.width() as u32,
                                        ctx.height() as u32,
                                        &path,
                                        FillRule::Winding,
                                        false,
                                    );
                                    ctx.reset(path.clear());
                                } else {
                                    clipmask.clear();
                                }
                            }
                        }
                    }
                    RenderNode::None => {}
                    _ => {
                        if let Some(region) = other.region() {
                            let region = t_bound.merge(&region);
                            ctx.damage_region(
                                &Texture::from(scene.deref()),
                                scene.region().crop(&region),
                                false,
                            );
                            self.merge(other);
                            self.render(ctx, clipmask, None);
                        }
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
