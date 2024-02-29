use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::{
    resources::{BoardOptions, UNOPENED_INDEX},
    GameResetEvent, CAMERA_SCALE, WINDOW_PADDING, WINDOW_TOP_HEIGHT,
};

pub const TILE_SIZE: f32 = 16.;

pub fn board_setup(
    mut commands: Commands,
    tilemap_query: Query<Entity, With<TilemapId>>,
    mut board_reset_event: EventWriter<GameResetEvent>,
    asset_server: Res<AssetServer>,
    board_options: Res<BoardOptions>,
) {
    for tilemap_entity in tilemap_query.iter() {
        commands.entity(tilemap_entity).despawn();
    }

    let texture_handle = asset_server.load::<Image>("texture.png");

    let map_size = TilemapSize::new(board_options.width, board_options.height);

    let tilemap_entity = commands.spawn_empty().id();

    let mut tile_storage = TileStorage::empty(map_size);

    for i in 0..map_size.x {
        for j in 0..map_size.y {
            let tile_pos = TilePos::new(i, j);
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    texture_index: TileTextureIndex(UNOPENED_INDEX),
                    ..default()
                })
                .id();
            tile_storage.set(&tile_pos, tile_entity)
        }
    }

    let tile_size = TilemapTileSize::new(TILE_SIZE, TILE_SIZE);
    let grid_size = tile_size.into();
    let map_type = TilemapType::default();

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle),
        tile_size,
        transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0),
        ..default()
    });

    board_reset_event.send(GameResetEvent(board_options.clone()));
}

pub fn board_resize(
    mut tilemap_query: Query<&mut Transform, With<TilemapType>>,
    mut window_query: Query<&mut Window>,
    board_options: Res<BoardOptions>,
) {
    let board_size =
        Vec2::splat(TILE_SIZE) * Vec2::new(board_options.width as f32, board_options.height as f32);

    // info!("board_size:{:?}", board_size);

    if let Ok(mut window) = window_query.get_single_mut() {
        let window_width = board_size.x * CAMERA_SCALE + WINDOW_PADDING + WINDOW_PADDING;
        let window_height = board_size.y * CAMERA_SCALE + WINDOW_TOP_HEIGHT + WINDOW_PADDING;

        // info!("WindowSize: {}x{}", window_width, window_height);
        window.resolution.set(window_width, window_height);
        // window.position.center(MonitorSelection::Current);

        if let Ok(mut transform) = tilemap_query.get_single_mut() {
            transform.translation.x = -(window_width / CAMERA_SCALE) / 2.0 + WINDOW_PADDING;
            transform.translation.y = -(window_height / CAMERA_SCALE) / 2.0 + WINDOW_PADDING;
        }
    };
}
