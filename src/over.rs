use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::{
    components::GameButton,
    resources::{
        Board, BoardOptions, LastStep, StartTime, BOMB_INDEX, BOMB_RED_INDEX, MARKED_INDEX,
        UNOPENED_INDEX,
    },
    GameResetEvent, GameState,
};

pub fn game_lose_system(
    tile_pos_query: Query<&TilePos>,
    tile_storage_query: Query<&TileStorage>,
    mut tile_texture_index_query: Query<&mut TileTextureIndex>,
    mut button_query: Query<&mut UiTextureAtlasImage, With<GameButton>>,
    board: Res<Board>,
    last_step: Res<LastStep>,
    mut game_state: ResMut<NextState<GameState>>,
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

    game_state.set(GameState::GameOver);
}

pub fn game_win_system(
    mut tile_texture_index_query: Query<&mut TileTextureIndex>,
    mut button_query: Query<&mut UiTextureAtlasImage, With<GameButton>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for mut texture_index in tile_texture_index_query.iter_mut() {
        if texture_index.0 == UNOPENED_INDEX {
            texture_index.0 = MARKED_INDEX;
        }
    }
    if let Ok(mut button_image) = button_query.get_single_mut() {
        button_image.index = 2;
    }

    game_state.set(GameState::GameOver);
}

pub fn game_reset_system(
    mut commands: Commands,
    mut tilemap_query: Query<(Entity, &mut TilemapSize, &mut TileStorage)>,
    mut tile_texture_index_query: Query<&mut TileTextureIndex>,
    mut button_query: Query<&mut UiTextureAtlasImage, With<GameButton>>,
    mut game_reset_events: EventReader<GameResetEvent>,
    mut board: ResMut<Board>,
    mut board_options: ResMut<BoardOptions>,
    mut start_time: ResMut<StartTime>,
    mut last_setp: ResMut<LastStep>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    if let Ok(mut button_image) = button_query.get_single_mut() {
        button_image.index = 0;
    }

    if !last_setp.uncover {
        last_setp.reset();
    }

    for mut texture_index in tile_texture_index_query.iter_mut() {
        texture_index.0 = UNOPENED_INDEX;
    }

    for game_reset in game_reset_events.read() {
        if board_options.width == game_reset.0.width && board_options.height == game_reset.0.height
        {
            board.reset(&game_reset.0);
            *board_options = game_reset.0.clone();
            continue;
        } else {
            if let Ok((tilemap_entity, mut map_size, mut old_storage)) =
                tilemap_query.get_single_mut()
            {
                let width = game_reset.0.width;
                let height = game_reset.0.height;
                let new_map_size = TilemapSize::new(width, height);
                let mut new_storage = TileStorage::empty(new_map_size);

                for entity in old_storage.iter().flatten() {
                    commands.entity(*entity).despawn_recursive();
                }

                for x in 0..width {
                    for y in 0..height {
                        let tile_pos = TilePos::new(x, y);
                        let tile_entity = commands
                            .spawn(TileBundle {
                                position: tile_pos,
                                tilemap_id: TilemapId(tilemap_entity),
                                texture_index: TileTextureIndex(UNOPENED_INDEX),
                                ..default()
                            })
                            .id();
                        new_storage.set(&tile_pos, tile_entity)
                    }
                }

                *old_storage = new_storage;
                *map_size = new_map_size;
            } else {
                println!("not storage");
            }

            board.reset(&game_reset.0);
            *board_options = game_reset.0.clone();
        }
    }

    start_time.0.reset();
    game_state.set(GameState::InGame);
}
