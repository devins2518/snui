# Messages and the `Controller` trait

In the [widget](./WIDGET.MD) documentation, we briefly went over the `M` in the `Widget` trait which stands for **Message**.

**Message** passing is the method use in [snui]() to share data between a `Widget` and the `Controller`.

The `Controller` trait is modelled similarly to rust's [mpsc](https://doc.rust-lang.org/std/sync/mpsc/index.html) channel with the exception that **messages** can be sent bilaterally.
 

## Messages

```rust
pub trait Controller {
    fn get<'m>(&'m self, msg: &'m M) -> Result<Data<'m, M>, ControllerError>;
    fn send<'m>(&'m mut self, msg: M) -> Result<Data<'m, M>, ControllerError>;
	...
}
```

To keep the following section simpler, I have omitted a few methods. We'll come back to them later.

`send` and `get` are relatively similar. The major differences being that `send` mutably borrows the `Controller` and takes an owned value whereas `get` take an immutable reference to your `Message` and `Controller`. This means `get` is meant to query data from your `Controller` and `send` transfer data to it.

### `TryIntoMessage` and `IntoMessage` trait

A companion of your **message** is `TryIntoMessage` and it's big brother `IntoMessage`.

```rust
pub trait TryIntoMessage<T> {
    type Error;
    fn into(&self, _: T) -> Result<Self, Self::Error> where Self : Sized;
}
```

> IntoMessage is identical except it returns `Self`.

What `TryIntoMessage<T>` allows you to do is build a new message from a template and a given `T`.

How it supposed to be used isn't really straightforward so I'll give an example from [snui]().

```rust
pub struct Slider<M: PartialEq + TryIntoMessage<f32>> {
    message: Option<M>,
    size: f32,
    pressed: bool,
    slider: Rectangle,
    orientation: Orientation,
}
```

This is snui's [slider](https://material.io/components/sliders) implementation. As you can see it requires that the `M` implements `TryIntoMessage<f32>` but more importantly that it stores a message.

Let's define our **message** :
```rust
enum System {
	Brightness(f32),
	Volume(f32)
}
```

Our message enum has to two members, both taking an `f32`. A slider that would send the volume to our `Controller` would look like this.

```rust
let slider = Slider::new(400, 10).message(System::Volume(-1.));
```

As volume, I gave `-1.` but obviously I don't want the slider to send continuously `-1.` to my controller, I want it to send its ratio.

This is when `TryIntoMessage<f32>` comes into play.

```rust
impl TryIntoMessage<f32> for System {
	type Error = ();
    fn into(&self, f: f32) -> Result<Self, Self::Error> where Self : Sized {
		match self {
			System::Volume(_) => Ok(System::Volume(f)),
			System::Brightness(_) => Ok(System::Brightness(f))
		}
	}
}
```

> In this instance, we could have implemented `IntoMessage<f32>` since all our enum members can return a new message. All types implementing `IntoMessage<T>` automatically have an implementation of `TryIntoMessage<T>`

Our slider can then do this :

```rust
let ratio: f32 = todo!();
if let Some(message) = self.message.as_ref() {
	if let Ok(msg) = message.into(ratio) {
		let _ = ctx.send(msg)
	}
}
``` 

> `System::Volume(-1.)` becomes `System::Volume(ratio)`

When you click or scroll on the [slider](), it calculates the ratio the bar takes of the available space, build a new message from the given and the ratio and sends it to the `Controller`.

### Summary

Messages are used to communicate Data to your `Controller`.
Your `M`essages can implement `TryIntoMessage<T>` so widgets can build new ones from a given template.

## Serialization

A `Controller` might want to buffer a stream a messages to atomically apply changes.

The `Controller` trait provides three additional methods for that purpose.

```rust
pub trait Controller<M> {
	...
    fn serialize(&mut self) -> Result<u32, ControllerError>;
    fn deserialize(&mut self, serial: u32) -> Result<(), ControllerError>;
    fn send_serialize<'m>(&'m mut self, serial: u32, msg: M) -> Result<Data<'m, M>, ControllerError>;
}
```

- `serialize` : Informs the `Controller` you want it to create a new buffer. If possible, it'll return a serial you can use to send serialized messages. This mean your `Controller` could have multiple buffers operating at the same time. 

- `send_serialize` : Similar to `send` except the message has to be serialized. The serial is the same given by `serialize`.

- `deserialize` : Close the buffer opened on `serialize` and atomically apply the serialized `M`essages.


## Syncing

Last but not least,
```rust
pub trait Controller<M> {
	...
    fn sync(&mut self) -> Result<M, ControllerError>;
}
```

`sync` is especially important because it lets your display backend know if the widget need to be synced again or not. This is necessary if you want to propagate changes made on the previous sync to other widgets. Here's an example of how it can be used.

```rust
// ev is an event coming from your system or backend
widget.sync(&mut sync_ctx, ev);
while let Ok(message) = sync_ctx.sync() {
	widget.sync(&mut sync_ctx, Event::Message(message)));
}
```

The loop will continue until the controller return an `Err()` which means there's no longer anything to **sync**.



