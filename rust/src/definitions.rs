use std::ops;

use godot::{
    builtin::Vector2i,
    classes::{Node, Node2D},
    global::godot_print,
    meta::ToGodot,
    obj::Gd,
};

pub type TilePos = (i32, i32);

pub trait TileEntity {
    fn set_start_pos(&mut self, x: i32, y: i32);
    fn tile_pos(&self) -> TilePos;
    fn is_solid(&self) -> bool;
}

pub(crate) trait TilePosExtension<T>
where
    T: ops::Add + ops::Sub,
{
    fn add(&self, other: &Self) -> Self;
    fn sub(&self, other: &Self) -> Self;
    fn scale<S: Copy>(&self, scale: &S) -> Self
    where
        T: ops::Mul<S, Output = T>;
    fn add_s<S: Copy>(&self, scale: &S) -> Self
    where
        T: ops::Add<S, Output = T>;
    fn abs(&self) -> Self;
}

impl TilePosExtension<i32> for TilePos {
    fn add(&self, other: &Self) -> Self {
        (self.0 + other.0, self.1 + other.1)
    }

    fn sub(&self, other: &Self) -> Self {
        (self.0 - other.0, self.1 - other.1)
    }

    fn scale<S: Copy>(&self, scale: &S) -> Self
    where
        i32: ops::Mul<S, Output = i32>,
    {
        (self.0 * *scale, self.1 * *scale)
    }

    fn add_s<S: Copy>(&self, scale: &S) -> Self
    where
        i32: ops::Add<S, Output = i32>,
    {
        (self.0 + *scale, self.1 + *scale)
    }

    fn abs(&self) -> Self {
        (self.0.abs(), self.1.abs())
    }
}

#[derive(Default)]
pub struct Globals {
    world: Option<Gd<Node>>,
}

impl Globals {
    pub fn new(node: &Node2D) -> Self {
        let world = node.get_node_or_null("/root/Scene/World");
        godot_print!("World is {world:?}");
        Self { world }
    }

    pub fn world_is_solid(&mut self, tile_pos: TilePos) -> bool {
        assert!(self.world.is_some(), "World is still empty!");
        self.world
            .as_mut()
            .unwrap()
            .call("is_solid", &[Vector2i::from_tuple(tile_pos).to_variant()])
            .try_to()
            .expect("Must be boolean!")
    }

    pub fn world_is_interactable(&mut self, tile_pos: TilePos) -> bool {
        assert!(self.world.is_some(), "World is still empty!");
        self.world
            .as_mut()
            .unwrap()
            .call(
                "is_interactable",
                &[Vector2i::from_tuple(tile_pos).to_variant()],
            )
            .try_to()
            .expect("Must be boolean!")
    }
}
