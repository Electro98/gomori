use godot::prelude::*;

mod definitions;
mod object;
mod player;
mod dialog;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}
