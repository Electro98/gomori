use std::cmp::max;

use godot::classes::{INode2D, Node2D};
use godot::global::{clampf, lerp};
use godot::prelude::*;

use crate::definitions::{Globals, TileEntity, TilePos, TilePosExtension};

#[derive(GodotClass)]
#[class(base=Node2D)]
struct Player {
    movement: MovementBlock,
    globals: Globals,
    base: Base<Node2D>,
}

#[derive(Default)]
struct MovementBlock {
    tile_pos: TilePos,
    last_pos: TilePos,
    is_moving: bool,
    moving_time: f64,
    step_time: f64,
    target_pos: TilePos,
    last_direction: Direction,
}

#[derive(GodotConvert, Var, Export, Default, Clone, Copy)]
#[godot(via = GString)]
enum Direction {
    Up = 0x01,
    Left = 0x02,
    #[default]
    Down = 0x04,
    Right = 0x08,
}

impl Direction {
    fn from(dir: TilePos) -> Option<Self> {
        let abs = dir.abs();
        if abs.0 == abs.1 {
            return None;
        }
        let is_x = max(abs.0, abs.1) == abs.0;
        let axis = dir.0 * is_x as i32 + dir.1 * !is_x as i32;
        Some(match (is_x, axis.is_negative()) {
            // y negative
            (false, true) => Direction::Up,
            // x negative
            (true, true) => Direction::Left,
            // y positive
            (false, false) => Direction::Down,
            // x positive
            (true, false) => Direction::Right,
        })
    }

    fn to_dir(self) -> TilePos {
        match self {
            Direction::Up => (0, -1),
            Direction::Left => (-1, 0),
            Direction::Down => (0, 1),
            Direction::Right => (1, 0),
        }
    }
}

#[godot_api]
impl INode2D for Player {
    fn init(base: Base<Node2D>) -> Self {
        Self {
            movement: MovementBlock {
                step_time: 0.3,
                ..Default::default()
            },
            globals: Globals::default(),
            base,
        }
    }

    fn physics_process(&mut self, delta: f64) {
        if let Some(pos) = self.movement.update(delta) {
            self.base_mut().set_position(pos);
        }
    }
}

#[godot_dyn]
impl TileEntity for Player {
    fn set_start_pos(&mut self, x: i32, y: i32) {
        self.teleport_to(x, y)
    }

    fn tile_pos(&self) -> TilePos {
        self.movement.tile_pos
    }

    fn is_solid(&self) -> bool {
        true
    }
}

#[godot_api]
impl Player {
    #[func]
    fn setup(&mut self) {
        self.globals = Globals::new(self.base().upcast_ref());
    }

    #[func]
    fn move_to(&mut self, x: i32, y: i32) {
        if self.globals.world_is_solid((x, y)) {
            let dir = (x, y).sub(&self.movement.tile_pos);
            if let Some(direction) = Direction::from(dir) {
                self.movement.last_direction = direction;
            }
        } else {
            self.movement.move_to(x, y)
        }
    }

    #[func]
    fn teleport_to(&mut self, x: i32, y: i32) {
        self.movement.tile_pos = (x, y);
        self.movement.stop();
        self.base_mut().set_position(to_coordinates_vec((x, y)));
    }

    #[func]
    fn tile_pos(&self) -> Vector2i {
        Vector2i::from_tuple(self.movement.tile_pos)
    }

    #[func]
    fn is_moving(&self) -> bool {
        self.movement.is_moving
    }

    #[func]
    fn direction(&self) -> Direction {
        self.movement.last_direction
    }

    #[func]
    fn in_front(&self) -> Vector2i {
        Vector2i::from_tuple(
            self.movement
                .tile_pos
                .add(&self.movement.last_direction.to_dir()),
        )
    }
}

fn to_coordinates<T: From<i32>>(tile_pos: TilePos) -> (T, T) {
    ((tile_pos.0 * 32).into(), (tile_pos.1 * 32).into())
}

fn to_coordinates_vec(tile_pos: TilePos) -> Vector2 {
    Vector2::new(tile_pos.0 as f32 * 32.0, tile_pos.1 as f32 * 32.0)
}

impl MovementBlock {
    fn move_to(&mut self, x: i32, y: i32) {
        if self.is_moving || self.tile_pos == (x, y) {
            log::debug!("trying to move player when it's not possible!");
            return;
        }
        self.target_pos = (x, y);
        self.last_pos = self.tile_pos;
        self.tile_pos = self.next_pos();
        self.is_moving = true;
        self.last_direction = self.direction(self.tile_pos).unwrap();
        self.moving_time = 0f64;
    }

    fn stop(&mut self) {
        self.is_moving = false;
        self.target_pos = self.tile_pos;
        self.last_pos = self.tile_pos;
    }

    fn update(&mut self, delta: f64) -> Option<Vector2> {
        if !self.is_moving {
            return None;
        }
        self.moving_time += delta;
        if self.moving_time >= self.step_time {
            self.moving_time -= self.step_time;
            self.last_direction = self.direction(self.tile_pos).unwrap();
            self.last_pos = self.tile_pos;
            self.tile_pos = self.next_pos();
        }
        self.is_moving = self.last_pos != self.target_pos;
        let new_pos = lerp(
            &to_coordinates_vec(self.last_pos).to_variant(),
            &to_coordinates_vec(self.tile_pos).to_variant(),
            &clampf(self.moving_time / self.step_time, 0.0, 1.0).to_variant(),
        );

        Some(new_pos.try_to().expect("Should be vector!"))
    }

    fn next_pos(&self) -> TilePos {
        let diff = (
            self.target_pos.0 - self.last_pos.0,
            self.target_pos.1 - self.last_pos.1,
        );
        // let diff = self.target_pos.sub(&self.last_pos);
        let max_diff = max(diff.0.abs(), diff.1.abs());
        if max_diff == 0 {
            self.target_pos
        } else if diff.0.abs() == max_diff {
            (
                self.last_pos.0 + 1 - 2 * diff.0.is_negative() as i32,
                self.last_pos.1,
            )
        } else {
            (
                self.last_pos.0,
                self.last_pos.1 + 1 - 2 * diff.1.is_negative() as i32,
            )
        }
    }

    fn direction(&self, next_pos: TilePos) -> Option<Direction> {
        if !self.is_moving {
            None
        } else {
            let diff = (next_pos.0 - self.last_pos.0, next_pos.1 - self.last_pos.1);
            // let diff = next_pos.sub(&self.last_pos);
            #[cfg(debug_assertions)]
            if diff.0 == diff.1 || (diff.0 != 0) == (diff.1 != 0) {
                godot_warn!("Only allowed difference is 1 off");
            }
            Direction::from(diff)
        }
    }
}
