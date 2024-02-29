use bevy::prelude::*;

#[derive(Component)]
pub struct GameButton;

#[derive(Component)]
pub struct BombCount;

#[derive(Component)]
pub struct StartTimeCount;

#[derive(Component)]
pub struct MainButton;

#[derive(Component)]
pub struct LevelButton;

#[derive(Component, Debug)]
pub enum NumberIndex {
    One,
    Two,
    Threr,
}

#[derive(Component)]
pub enum Level {
    Base,
    Pro,
    Expert,
}
