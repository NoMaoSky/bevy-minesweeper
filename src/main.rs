use bevy::{
    input::{mouse::MouseButtonInput, ButtonState},
    prelude::*,
    window::WindowResolution,
};
use bevy_ecs_tilemap::{
    map::{TilemapGridSize, TilemapSize, TilemapType},
    tiles::{TilePos, TileTextureIndex},
    TilemapPlugin,
};
use board::{board_resize, board_setup, TILE_SIZE};
use components::{BombCount, GameButton, NumberIndex, StartTimeCount};
use over::{game_over_system, game_reset_system, game_win_system};
use resources::{Board, BoardOptions, LastStep, StartTime, MARKED_INDEX};
use tile::{
    check_tiles_system, mark_tiles_system, number_tiles_system, re_uncover_tile_system,
    safe_step_system, uncover_tiles_system, TileCheckEvent, TileMarkEvent, TileNumberEvent,
    TileUncoverEvent,
};

mod board;
mod components;
mod over;
mod resources;
mod tile;

const WINDOW_TOP_HEIGHT: f32 = 70.0;
const WINDOW_PADDING: f32 = 10.0;
const CAMERA_SCALE: f32 = 2.0;

pub fn main() {
    let window_width =
        (TILE_SIZE * BoardOptions::default().width as f32 + WINDOW_PADDING * 2.) * CAMERA_SCALE;
    let window_height = (TILE_SIZE * 9.0 + WINDOW_PADDING * 2.) * CAMERA_SCALE + WINDOW_TOP_HEIGHT;

    App::new()
        .insert_resource(ClearColor(Color::rgb_u8(204, 204, 204)))
        .init_resource::<BoardOptions>()
        .init_resource::<Board>()
        .init_resource::<LastStep>()
        .init_resource::<StartTime>()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: WindowResolution::new(window_width, window_height),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(TilemapPlugin)
        .add_event::<TileUncoverEvent>()
        .add_event::<TileNumberEvent>()
        .add_event::<TileCheckEvent>()
        .add_event::<TileMarkEvent>()
        .add_state::<GameState>()
        .add_systems(Startup, (game_setup, board_setup))
        .add_systems(
            OnEnter(GameState::InGame),
            (board_resize, re_uncover_tile_system),
        )
        .add_systems(
            Update,
            (
                cursor_movement,
                game_timing,
                game_start_timer,
                game_bomb_count,
                safe_step_system,
                uncover_tiles_system.after(safe_step_system),
                number_tiles_system.after(safe_step_system),
                check_tiles_system.after(safe_step_system),
                mark_tiles_system.after(safe_step_system),
            )
                .chain()
                .distributive_run_if(in_state(GameState::InGame)),
        )
        .add_systems(Update, button_click)
        .add_systems(
            OnEnter(GameState::GameOver(GameOverState::Lose)),
            game_over_system,
        )
        .add_systems(
            OnEnter(GameState::GameOver(GameOverState::Win)),
            game_win_system,
        )
        .add_systems(
            OnEnter(GameState::GameOver(GameOverState::Reset)),
            game_reset_system,
        )
        .run()
}

fn game_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let mut camera = Camera2dBundle::default();
    camera.transform.scale /= CAMERA_SCALE;
    commands.spawn(camera);

    let box_image = asset_server.load::<Image>("box0.png");

    let state_texture_handle = asset_server.load::<Image>("state.png");
    let state_texture_atlas = TextureAtlas::from_grid(
        state_texture_handle,
        Vec2::new(21.0, 21.0),
        3,
        1,
        None,
        None,
    );
    let state_texture_atlas_handle = texture_atlases.add(state_texture_atlas);

    let time_texture_handle = asset_server.load("time.png");
    let time_texture_atlas =
        TextureAtlas::from_grid(time_texture_handle, Vec2::new(13.0, 23.0), 4, 3, None, None);
    let time_texture_atlas_handle = texture_atlases.add(time_texture_atlas);

    commands
        .spawn(NodeBundle {
            style: Style {
                padding: UiRect::all(Val::Px(WINDOW_PADDING * CAMERA_SCALE)),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                width: Val::Percent(100.),
                height: Val::Px(WINDOW_TOP_HEIGHT + WINDOW_PADDING * CAMERA_SCALE),
                ..default()
            },
            ..default()
        })
        .with_children(|children| {
            children
                .spawn(NodeBundle {
                    style: Style {
                        display: Display::Flex,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(2.0)),
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    border_color: BorderColor(Color::rgb_u8(128, 128, 128)),
                    ..default()
                })
                .with_children(|children| {
                    children
                        .spawn(NodeBundle {
                            style: Style {
                                padding: UiRect::left(Val::Px(2.0)),
                                display: Display::Flex,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|children| {
                            children.spawn((
                                AtlasImageBundle {
                                    style: Style {
                                        width: Val::Px(13.0 * 1.4),
                                        height: Val::Px(23.0 * 1.4),
                                        ..default()
                                    },
                                    texture_atlas: time_texture_atlas_handle.clone(),
                                    ..default()
                                },
                                BombCount,
                                NumberIndex::One,
                            ));
                            children.spawn((
                                AtlasImageBundle {
                                    style: Style {
                                        width: Val::Px(13.0 * 1.4),
                                        height: Val::Px(23.0 * 1.4),
                                        ..default()
                                    },
                                    texture_atlas: time_texture_atlas_handle.clone(),
                                    ..default()
                                },
                                BombCount,
                                NumberIndex::Two,
                            ));
                            children.spawn((
                                AtlasImageBundle {
                                    style: Style {
                                        width: Val::Px(13.0 * 1.4),
                                        height: Val::Px(23.0 * 1.4),
                                        ..default()
                                    },
                                    texture_atlas: time_texture_atlas_handle.clone(),
                                    ..default()
                                },
                                BombCount,
                                NumberIndex::Threr,
                            ));
                        });

                    children
                        .spawn(ButtonBundle {
                            button: Button,
                            style: Style {
                                display: Display::Flex,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                width: Val::Px(35.),
                                height: Val::Px(35.),
                                ..default()
                            },
                            image: UiImage::new(box_image),
                            ..default()
                        })
                        .with_children(|children| {
                            children.spawn((
                                AtlasImageBundle {
                                    style: Style {
                                        width: Val::Px(31.0),
                                        height: Val::Px(31.0),
                                        ..default()
                                    },
                                    texture_atlas: state_texture_atlas_handle,
                                    ..Default::default()
                                },
                                GameButton,
                            ));
                        });

                    children
                        .spawn(NodeBundle {
                            style: Style {
                                padding: UiRect::right(Val::Px(2.0)),
                                display: Display::Flex,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|children| {
                            children.spawn((
                                AtlasImageBundle {
                                    style: Style {
                                        width: Val::Px(13.0 * 1.4),
                                        height: Val::Px(23.0 * 1.4),
                                        ..default()
                                    },
                                    texture_atlas: time_texture_atlas_handle.clone(),
                                    ..default()
                                },
                                StartTimeCount,
                                NumberIndex::One,
                            ));
                            children.spawn((
                                AtlasImageBundle {
                                    style: Style {
                                        width: Val::Px(13.0 * 1.4),
                                        height: Val::Px(23.0 * 1.4),
                                        ..default()
                                    },
                                    texture_atlas: time_texture_atlas_handle.clone(),
                                    ..default()
                                },
                                StartTimeCount,
                                NumberIndex::Two,
                            ));
                            children.spawn((
                                AtlasImageBundle {
                                    style: Style {
                                        width: Val::Px(13.0 * 1.4),
                                        height: Val::Px(23.0 * 1.4),
                                        ..default()
                                    },
                                    texture_atlas: time_texture_atlas_handle.clone(),
                                    ..default()
                                },
                                StartTimeCount,
                                NumberIndex::Threr,
                            ));
                        });
                });
        });
}

fn game_timing(last_step: Res<LastStep>, time: Res<Time>, mut start_time: ResMut<StartTime>) {
    if last_step.coord.is_some() && !last_step.uncover {
        start_time.0.tick(time.delta());
    }
}

fn cursor_movement(
    window: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    tilemap_query: Query<(&TilemapSize, &TilemapGridSize, &Transform)>,
    mut mouse_button_events: EventReader<MouseButtonInput>,
    mut tile_uncover_event: EventWriter<TileUncoverEvent>,
    mut tile_mark_event: EventWriter<TileMarkEvent>,
) {
    let window = window.single();
    let (camera, camera_transfrom) = camera_query.single();

    for mouse_event in mouse_button_events.read() {
        if mouse_event.state == ButtonState::Released {
            let cursor_pos = window
                .cursor_position()
                .and_then(|cursor_pos| camera.viewport_to_world_2d(camera_transfrom, cursor_pos))
                .unwrap();

            for (tilemap_size, tilemap_grid_size, tilemap_transfrom) in tilemap_query.iter() {
                let cursor_pos = {
                    let cursor_pos = Vec4::from((cursor_pos, 0.0, 1.0));
                    let cursor_in_map_pos =
                        tilemap_transfrom.compute_matrix().inverse() * cursor_pos;
                    cursor_in_map_pos.xy()
                };

                if let Some(tile_pos) = TilePos::from_world_pos(
                    &cursor_pos,
                    tilemap_size,
                    tilemap_grid_size,
                    &TilemapType::Square,
                ) {
                    match mouse_event.button {
                        MouseButton::Left => tile_uncover_event.send(TileUncoverEvent {
                            coord: (tile_pos.x, tile_pos.y),
                        }),
                        MouseButton::Right => tile_mark_event.send(TileMarkEvent {
                            coord: (tile_pos.x, tile_pos.y),
                        }),
                        _ => (),
                    }
                }
            }
        }
    }
}

fn button_click(
    mut button_query: Query<(&Interaction, &mut UiImage), (With<Button>, Changed<Interaction>)>,
    mut game_state: ResMut<NextState<GameState>>,
    mut last_step: ResMut<LastStep>,
    asset_server: Res<AssetServer>,
) {
    for (interaction, mut ui_image) in button_query.iter_mut() {
        match interaction {
            Interaction::Pressed => {
                ui_image.texture = asset_server.load::<Image>("box1.png");
                last_step.uncover = false;
                game_state.set(GameState::GameOver(GameOverState::Reset));
            }
            _ => {
                ui_image.texture = asset_server.load::<Image>("box0.png");
            }
        }
    }
}

fn game_start_timer(
    mut start_time_query: Query<(&mut UiTextureAtlasImage, &NumberIndex), With<StartTimeCount>>,
    start_time: Res<StartTime>,
) {
    let start_time = start_time.0.elapsed_secs() as i32;

    let hundreds = ((start_time / 100) % 10) as usize;
    let tens = ((start_time / 10) % 10) as usize;
    let units = (start_time % 10) as usize;

    for (mut texture_atlas_image, number_index) in start_time_query.iter_mut() {
        match number_index {
            NumberIndex::One => texture_atlas_image.index = hundreds,
            NumberIndex::Two => texture_atlas_image.index = tens,
            NumberIndex::Threr => texture_atlas_image.index = units,
        }
    }
}

fn game_bomb_count(
    texture_index_query: Query<&TileTextureIndex>,
    mut bomb_count_query: Query<(&mut UiTextureAtlasImage, &NumberIndex), With<BombCount>>,
    board_options: Res<BoardOptions>,
) {
    let marked_count = texture_index_query
        .iter()
        .filter(|index| index.0 == MARKED_INDEX)
        .count();

    let num_int = board_options.bomb_count - marked_count as u32;

    let hundreds = ((num_int / 100) % 10) as usize;
    let tens = ((num_int / 10) % 10) as usize;
    let units = (num_int % 10) as usize;

    for (mut texture_atlas_image, number_index) in bomb_count_query.iter_mut() {
        match number_index {
            NumberIndex::One => texture_atlas_image.index = hundreds,
            NumberIndex::Two => texture_atlas_image.index = tens,
            NumberIndex::Threr => texture_atlas_image.index = units,
        }
    }
}

#[derive(States, Debug, Hash, PartialEq, Eq, Clone)]
enum GameState {
    InGame,
    GameOver(GameOverState),
}

impl Default for GameState {
    fn default() -> Self {
        GameState::GameOver(GameOverState::Reset)
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
enum GameOverState {
    Win,
    Lose,
    Reset,
}
