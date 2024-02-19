use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::{
    resources::{
        Board, BoardOptions, LastStep, BOMB_INDEX, MARKED_INDEX, OPENED_INDEX, UNOPENED_INDEX,
    },
    GameOverState, GameState,
};

#[derive(Event)]
pub struct TileUncoverEvent {
    pub coord: (u32, u32),
}

#[derive(Event)]
pub struct TileNumberEvent {
    pub coord: (u32, u32),
    pub state: u32,
}

#[derive(Event)]
pub struct TileCheckEvent {
    pub coord: (u32, u32),
    pub state: u32,
}

#[derive(Event)]
pub struct TileMarkEvent {
    pub coord: (u32, u32),
}

pub fn re_uncover_tile_system(
    last_step: Res<LastStep>,
    mut tile_uncover_event: EventWriter<TileUncoverEvent>,
) {
    if last_step.uncover {
        if let Some(coord) = last_step.coord {
            tile_uncover_event.send(TileUncoverEvent { coord });
        }
    }
}

pub fn safe_step_system(
    mut tile_uncover_events: EventReader<TileUncoverEvent>,
    board: Res<Board>,
    mut last_step: ResMut<LastStep>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for uncover_event in tile_uncover_events.read() {
        let coord = uncover_event.coord;

        if board.bomb_count_at(coord) == 0 && !board.is_bomb_at(coord) {
            last_step.coord = Some(coord);
            last_step.uncover = false;
        } else {
            if last_step.coord.is_none() {
                last_step.uncover = true;
            }

            last_step.coord = Some(coord);

            if last_step.uncover {
                game_state.set(GameState::GameOver(GameOverState::Reset));
            }
        }
    }
}

pub fn uncover_tiles_system(
    tile_storage_query: Query<&TileStorage>,
    mut tile_texture_inedx_query: Query<&mut TileTextureIndex>,
    mut tile_uncover_events: EventReader<TileUncoverEvent>,
    mut tile_number_event: EventWriter<TileNumberEvent>,
    mut tile_check_event: EventWriter<TileCheckEvent>,
    last_step: Res<LastStep>,
    board: Res<Board>,
) {
    if last_step.uncover {
        return;
    }
    if let Ok(tile_storage) = tile_storage_query.get_single() {
        for uncover_event in tile_uncover_events.read() {
            let coord = uncover_event.coord;
            let tile_pos = TilePos::new(coord.0, coord.1);
            if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                if let Ok(mut texture_index) = tile_texture_inedx_query.get_mut(tile_entity) {
                    let coord = (tile_pos.x, tile_pos.y);
                    let mut state = board.get(coord);
                    match texture_index.0 {
                        UNOPENED_INDEX => {
                            if state != BOMB_INDEX {
                                state = board.bomb_count_at((tile_pos.x, tile_pos.y));
                            }
                            texture_index.0 = state;

                            tile_check_event.send(TileCheckEvent { coord, state });
                        }
                        1..=8 => {
                            tile_number_event.send(TileNumberEvent {
                                coord,
                                state: texture_index.0,
                            });
                        }
                        _ => (),
                    }
                }
            };
        }
    }
}

pub fn number_tiles_system(
    mut tile_number_events: EventReader<TileNumberEvent>,
    mut tile_uncover_event: EventWriter<TileUncoverEvent>,
    tile_storage_query: Query<&TileStorage>,
    tile_texture_inedx_query: Query<&TileTextureIndex>,
    board: Res<Board>,
) {
    if let Ok(tile_storage) = tile_storage_query.get_single() {
        for number_event in tile_number_events.read() {
            let (coord, state) = (number_event.coord, number_event.state);
            let range = board.safe_square_at(coord);
            let mut marked_count = 0;
            let mut unopend_coords = vec![];
            for coord in range {
                let tile_pos = TilePos::new(coord.0, coord.1);
                if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                    if let Ok(texture_inedx) = tile_texture_inedx_query.get(tile_entity) {
                        match texture_inedx.0 {
                            MARKED_INDEX => {
                                marked_count += 1;
                            }
                            UNOPENED_INDEX => {
                                unopend_coords.push(coord);
                            }
                            _ => (),
                        }
                    }
                }
            }

            if state == marked_count {
                for coord in unopend_coords {
                    tile_uncover_event.send(TileUncoverEvent { coord });
                }
            }
        }
    }
}

pub fn check_tiles_system(
    mut tile_check_events: EventReader<TileCheckEvent>,
    mut tile_uncover_event: EventWriter<TileUncoverEvent>,
    tile_texture_inedx_query: Query<&TileTextureIndex>,
    board_options: Res<BoardOptions>,
    board: Res<Board>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for check_event in tile_check_events.read() {
        let (coord, state) = (check_event.coord, check_event.state);

        let mut check = || {
            let uncover_count = tile_texture_inedx_query
                .iter()
                .filter(|index| index.0 == UNOPENED_INDEX || index.0 == MARKED_INDEX)
                .count() as u32;
            if uncover_count == board_options.bomb_count {
                game_state.set(GameState::GameOver(GameOverState::Win));
            }
        };

        match state {
            OPENED_INDEX => {
                for pos in board.safe_square_at(coord) {
                    tile_uncover_event.send(TileUncoverEvent { coord: pos });
                }
                check();
            }
            BOMB_INDEX => {
                game_state.set(GameState::GameOver(GameOverState::Lose));
            }
            _ => {
                check();
            }
        }
    }
}

pub fn mark_tiles_system(
    tile_storage_query: Query<&TileStorage>,
    mut tile_texture_inedx_query: Query<&mut TileTextureIndex>,
    mut tile_mark_events: EventReader<TileMarkEvent>,
) {
    if let Ok(tile_storage) = tile_storage_query.get_single() {
        for mark_event in tile_mark_events.read() {
            let coord = mark_event.coord;
            let tile_pos = TilePos::new(coord.0, coord.1);
            if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                if let Ok(mut texture_index) = tile_texture_inedx_query.get_mut(tile_entity) {
                    if texture_index.0 == UNOPENED_INDEX {
                        texture_index.0 = MARKED_INDEX;
                    } else if texture_index.0 == MARKED_INDEX {
                        texture_index.0 = UNOPENED_INDEX;
                    }
                }
            }
        }
    }
}
