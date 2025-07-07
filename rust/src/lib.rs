use godot::{classes::Engine, prelude::*};

use crate::state::GameState;

mod definitions;
mod dialog;
mod object;
mod player;
mod state;

pub(crate) mod singletons {
    use godot::obj::Gd;

    use crate::state::GameState;

    pub const GAME_STATE: &str = "GameState";

    pub fn game_state() -> Gd<GameState> {
        godot::classes::Engine::singleton()
            .get_singleton(GAME_STATE)
            .expect("Something went wrong with getting singleton")
            .cast()
    }
}

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {
    fn on_level_init(level: InitLevel) {
        if level == InitLevel::Scene {
            Engine::singleton().register_singleton(
                singletons::GAME_STATE,
                &GameState::new_alloc(),
            );
        }
    }

    fn on_level_deinit(level: InitLevel) {
        if level == InitLevel::Scene {
            let mut engine = Engine::singleton();

            let singleton_name = singletons::GAME_STATE;
            if let Some(singleton) = engine.get_singleton(singleton_name) {
                engine.unregister_singleton(singleton_name);
                singleton.free();
            } else {
                // You can either recover, or panic from here.
                godot_error!("Failed to get singleton");
            }
        }
    }
}
