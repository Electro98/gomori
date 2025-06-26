use godot::classes::{AnimatedSprite2D, IAnimatedSprite2D};
use godot::prelude::*;

use crate::definitions::{TileEntity, TilePos};

#[derive(GodotClass)]
#[class(base=AnimatedSprite2D)]
struct InteractiveObject {
    #[export]
    tile_pos: Vector2i,
    #[export]
    is_solid: bool,
    #[export]
    autoplay: bool,
    base: Base<AnimatedSprite2D>,
}

#[godot_api]
impl IAnimatedSprite2D for InteractiveObject {
    fn init(base: Base<AnimatedSprite2D>) -> Self {
        Self {
            tile_pos: Vector2i::new(0, 0),
            is_solid: false,
            autoplay: true,
            base,
        }
    }

    fn ready(&mut self) {
        if self.autoplay {
            self.base_mut().play();
        }
    }
}

#[godot_dyn]
impl TileEntity for InteractiveObject {
    fn set_start_pos(&mut self, x: i32, y: i32) {
        self.tile_pos = Vector2i::new(x, y);
    }

    fn tile_pos(&self) -> TilePos {
        self.tile_pos.to_tuple()
    }

    fn is_solid(&self) -> bool {
        self.is_solid
    }
}
