
use godot::prelude::*;

#[derive(GodotClass, Default)]
#[class(base=Object)]
pub struct GameState {
    state: GlobalState
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, GodotConvert)]
#[godot(via = i32)]
pub enum GlobalState {
    UIMenu,
    Dialog,
    #[default]
    World,
}

#[godot_api]
impl IObject for GameState {
    fn init(base: Base<Object>) -> Self {
        GameState { ..Default::default() }
    }
}

#[godot_api]
impl GameState {
    #[constant]
    const UI_MENU: i32 = GlobalState::UIMenu as i32;
    #[constant]
    const DIALOG: i32 = GlobalState::Dialog as i32;
    #[constant]
    const WORLD: i32 = GlobalState::World as i32;

    #[func]
    pub fn change_state(&mut self, state: GlobalState) {
        self.state = state;
    }

    #[func]
    pub fn current_state(&self) -> GlobalState {
        self.state
    }
}
