use godot::prelude::*;

mod definitions;
mod dialog;
mod object;
mod player;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}
