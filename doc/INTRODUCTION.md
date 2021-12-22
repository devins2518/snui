# Introduction to Snui

snui is a simple UI library with a retained mode API.

Currently snui only provides a Wayland backend implementing the [wlr_layer_shell](https://wayland.app/protocols/wlr-layer-shell-unstable-v1) protocol extension although [xdg_shell](https://wayland.app/protocols/xdg-shell) support is planned.

> Note that this is mostly a convenience for me since it's my platform of interest. I will not offer the same treatment to others. They can be implemented independently.

To create the GUI, you first need to create a `widget` tree. `Widget`s from this tree handle `event`s such as mouse inputs or `message`s. The scene graph of the widget tree is built, compared to the previous and then rendered.


## Current goals

Provide a simple and accessible way to write small GUI applications in Rust. Ideally, [snui]() could serve as the foundation for a desktop shell.


## Key traits

- **Primitive** : Rendered UI components.

- **Controller** : The heart of your application.

- **Widget** : How the user interface is represented.