use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::{
    resources::{BoardOptions, UNOPENED_INDEX},
    CAMERA_SCALE, WINDOW_PADDING, WINDOW_TOP_HEIGHT,
};

pub const TILE_SIZE: f32 = 16.;

pub fn board_setup(
    mut commands: Commands,
    tilemap_query: Query<Entity, With<TilemapId>>,
    asset_server: Res<AssetServer>,
    board_options: Res<BoardOptions>,
) {
    for tilemap_entity in tilemap_query.iter() {
        commands.entity(tilemap_entity).despawn();
    }

    let texture_handle = asset_server.load::<Image>("texture.png");

    let map_size = TilemapSize::new(board_options.width, board_options.height);

    let tilemap_id = commands.spawn_empty().id();

    let mut tile_storage = TileStorage::empty(map_size);

    for i in 0..map_size.x {
        for j in 0..map_size.y {
            let tile_pos = TilePos::new(i, j);
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_id),
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

    let mut transform = get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0);
    transform.translation.y -= 15.0;

    commands.entity(tilemap_id).insert(TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle),
        tile_size,
        transform,
        ..default()
    });
}

pub fn board_resize(
    // mut tilemap_query: Query<&mut Transform, With<TilemapType>>,
    mut window_query: Query<&mut Window>,
    board_options: Res<BoardOptions>,
) {
    let board_width = board_options.width;
    let board_height = board_options.height;

    if let Ok(mut window) = window_query.get_single_mut() {
        let window_width = (TILE_SIZE * board_width as f32 + WINDOW_PADDING * 2.0) * CAMERA_SCALE;
        let window_height = (TILE_SIZE * board_height as f32 + WINDOW_PADDING * 2.0) * CAMERA_SCALE
            + WINDOW_TOP_HEIGHT;
        window.resolution.set(window_width, window_height);

        // window.resolution = WindowResolution::new(window_width, window_height);
    };
}
