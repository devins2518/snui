# `Controller` trait

The `controller` is the heart of your application. It decides when the widget tree should be updated and information served to the UI.


## Quick overview

```rust
pub trait Controller {
    // Tells the model all incoming messages are linked
    // The Controller returns a token that can be used to deserialize
    fn serialize(&mut self, msg: Message) -> Result<u32, ControllerError>;
    // Ends the serialization
    fn deserialize(&mut self, token: u32) -> Result<(), ControllerError>;
    // These interface are from the point of view of the widgets
    fn get<'c>(&'c self, msg: Message) -> Result<Data<'c>, ControllerError>;
    // The Message must be a u32 serial.
    fn send<'c>(&'c mut self, msg: Message) -> Result<Data<'c>, ControllerError>;
    // Returns an Ok(Message) if the application needs to be synced
    fn sync(&mut self) -> Result<Message<'static>, ControllerError>;
}
```

To communicate with the controller, widgets can use two methods, `send` and `get`. They are nearly similar with the exception that `send` can mutate the content of the controller. For this reason, `get` is recommended to be used to request information about the controller and `send` to share `data` with it.

This `data` is shared through `Message`s which are composed of an object (`u32`) and a `Data<'_>`.

Another couple of methods offered by the controller are `serialize` and `deserialize`. Serialize informs the controller you want it to atomically apply the following messages on `deserialize`.

To avoid conflict, it's recommended to use an unique serial for each serialization.

The last method, `sync` is especially important because it lets your backend know if the widget need to be synced again or not. This is necessary if you want to propagate changes made on the previous `sync` to other widgets. Here's an example of how it can be used.

``` rust
// ev is an event coming from your system or backend
widget.sync(&mut sync_ctx, ev);
while let Ok(message) = sync_ctx.sync() {
	widget.sync(&mut sync_ctx, Event::Message(message)));
}
```

The loop will continue until the controller return an `Err()` which means there's no longer anything to **sync**.
