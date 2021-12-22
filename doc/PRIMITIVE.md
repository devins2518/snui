# `Primitive` trait

Primitives are the building blocks of the GUI. The only primitives implemented in snui are [Rectangle](../src/widgets/shapes/rectangle.rs) and [Image](../src/widgets/image.rs). The adjacent type to `Primitive` is `PrimitiveType`, an enum with all the commonly used primitives and `Other` for unique ones.

In snui, instead of having a renderer that draws all imaginable shapes, users can implement `Primitive` on a type and have it draw itself on the given rendering backend.

## Quick overview

```rust
pub trait Primitive: Geometry + std::fmt::Debug {
    fn draw_with_transform_clip(
        &self,
        ctx: &mut DrawContext,
        transform: tiny_skia::Transform,
        clip: Option<&tiny_skia::ClipMask>,
    );
    fn get_background(&self) -> scene::Background;
    fn apply_background(&self, background: scene::Background) -> scene::PrimitiveType;
    // Tell if the region can fit inside the Primitive
    // The coordinates will be relative to the Primitive
    fn contains(&self, region: &scene::Region) -> bool;
    // Basically Clone
    fn into_primitive(&self) -> scene::PrimitiveType;
}

```

The only method responsible for rendering is `draw_with_transform_clip`.

> If you use `tiny_skia` to, build the path from the origin (0, 0). `Transform` will translate it.

The other methods are using during _invalidation_ to accurately redraw the scene.

## Custom primtives

```rust
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
```

Brief return on `PrimtiveType`. We discussed about `Rectangle` and `Image` previously but not about [Label](../src/widgets/text.rs). Label doesn't implement `Primitive`. It is drawn directly by the [DrawContext](../src/context.rs).

If you wish to create your own, it is recommended to use this method from `Instruction`.

```rust
fn other<P: 'static + Hash + Primitive>(x: f32, y: f32, primitive: P) -> Instruction {
	todo!()
}
```

The hash of your `primitive` will be the `id` used to compare it to its previous iteration and the `name` is your primitive's name.