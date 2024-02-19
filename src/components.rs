use bevy::prelude::*;

#[derive(Component)]
pub struct GameButton;

#[derive(Component)]
pub struct BombCount;

#[derive(Component)]
pub struct StartTimeCount;

#[derive(Component, Debug)]
pub enum NumberIndex {
    One,
    Two,
    Threr,
}
