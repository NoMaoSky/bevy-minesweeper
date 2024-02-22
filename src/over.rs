use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::{
    components::GameButton,
    resources::{
        Board, BoardOptions, LastStep, StartTime, BOMB_INDEX, BOMB_RED_INDEX, MARKED_INDEX,
        UNOPENED_INDEX,
    },
    GameState,
};

pub fn game_over_system(
    tile_pos_query: Query<&TilePos>,
    tile_storage_query: Query<&TileStorage>,
    mut tile_texture_index_query: Query<&mut TileTextureIndex>,
    board: Res<Board>,
    last_step: Res<LastStep>,
    mut button_query: Query<&mut UiTextureAtlasImage, With<GameButton>>,
) {
    let tile_storage = tile_storage_query.single();

    if let Ok(mut button_image) = button_query.get_single_mut() {
        button_image.index = 1;
    }

    if let Some(coord) = last_step.coord {
        let tile_pos = TilePos::new(coord.0, coord.1);
        if let Some(tile_entity) = tile_storage.get(&tile_pos) {
            if let Ok(mut texture_inedx) = tile_texture_index_query.get_mut(tile_entity) {
                if texture_inedx.0 == BOMB_INDEX {
                    texture_inedx.0 = BOMB_RED_INDEX;
                }
            }
        }
    }

    for tile_pos in tile_pos_query.iter() {
        let coord = (tile_pos.x, tile_pos.y);
        if board.is_bomb_at(coord) {
            if let Some(tile_entity) = tile_storage.get(tile_pos) {
                if let Ok(mut texture_inedx) = tile_texture_index_query.get_mut(tile_entity) {
                    if texture_inedx.0 == UNOPENED_INDEX {
                        texture_inedx.0 = BOMB_INDEX;
                    }
                }
            }
        }
    }
}

pub fn game_win_system(
    mut tile_texture_index_query: Query<&mut TileTextureIndex>,
    mut button_query: Query<&mut UiTextureAtlasImage, With<GameButton>>,
) {
    for mut texture_index in tile_texture_index_query.iter_mut() {
        if texture_index.0 == UNOPENED_INDEX {
            texture_index.0 = MARKED_INDEX;
        }
    }
    if let Ok(mut button_image) = button_query.get_single_mut() {
        button_image.index = 2;
    }
}

pub fn game_reset_system(
    mut tile_texture_index_query: Query<&mut TileTextureIndex>,
    mut button_query: Query<&mut UiTextureAtlasImage, With<GameButton>>,
    mut board: ResMut<Board>,
    board_options: Res<BoardOptions>,
    mut start_time: ResMut<StartTime>,
    mut last_setp: ResMut<LastStep>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    if let Ok(mut button_image) = button_query.get_single_mut() {
        button_image.index = 0;
    }

    for mut texture_index in tile_texture_index_query.iter_mut() {
        texture_index.0 = UNOPENED_INDEX;
    }

    if !last_setp.uncover {
        last_setp.reset();
    }
    start_time.0.reset();
    board.reset(&board_options);
    game_state.set(GameState::InGame);
}
