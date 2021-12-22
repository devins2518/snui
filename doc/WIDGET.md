# `Widget` trait and widgets

Widgets are composable UI components. You start with a root widget which you add others to build the widget tree. The `widget` tree is turned into a `RenderNode` which builds a _scene graph_ of the GUI.

For the most part widgets are separated in two kinds, layout widgets which often implement the `Container` trait and singles which wrap one widgets or simply directly return a `RenderNode`.

snui at a high level 

1. **sync** : An event from the display or a message is passed down the widget tree through the `sync` method. Widgets are given **mutable** access to the [SyncContext]() which implements the `Controller` trait and wraps your real controller. Many [sync]() can occur before the widget is asked to create its `RenderNode`.

2. **create_node**: Once the state of widgets has been updated, they create their their `RenderNode`. This is the part where layouting is done. The method additionally gives as parameters the `x` and `y` coordinates of the widget. Child's widget `create_node` need to be called from their **real position**.

3. **merge**: After the `RenderNode` is created, it is merged with the previous one to create the new _scene graph_. During this merge, conflicting instructions are replaced and rendered on the [DrawContext](). This step is mostly done behind the [scenes]().

## `WidgetUtil` trait

`WidgetUtil` is a trait implemented by all widgets which adds additional [builder]() type methods. 

The most notable ones being `ext` and `button` which respectively  wrap the widget in a `WidgetExt` and `Button`.

```rust
fn button() -> impl Widget {
	Label::default("click me", 15.)
	.ext()
	.background(BG2)
	.button(|this, _, pointer| match pointer {
		Pointer::MouseClick {time:_, pressed, button} => {
			if pressed && button.is_left() {
				this.set_background(BG0);
				this.edit("I'm clicked");
			}
		}
		_ => {}
	})
}
```


## `Container` trait

The `Container` trait is implemented by all layout widgets provided by **snui**.

Containers implement `FromIterator<Child>` which means they can be created from an iterator of `Child`s. 

```rust
// Example from bin/color.rs
fn sliders() -> WidgetLayout {
    [RED, GRN, BLU, BG0]
        .iter()
        .map(|color| {
            let id = match *color {
                RED => Signal::Red,
                BLU => Signal::Blue,
                GRN => Signal::Green,
                BG0 => Signal::Alpha,
                _ => Signal::Close,
            };
            widgets::slider::Slider::new(200, 8)
                .id(id as u32)
                .background(*color)
                .ext()
                .background(BG2)
                .even_radius(3.)
                .child()
        })
        .collect::<WidgetLayout>()
        .spacing(10.)
        .orientation(Orientation::Vertical)
}
```

The `Child` widget is especially convenient because it tracks damages from the widget it wraps and resets it on `create_node`. It also does some optimization by returning `RenderNode::None` when nothing needs to be damaged. This generally means you don't have to implement these behaviours on your own custom widgets.