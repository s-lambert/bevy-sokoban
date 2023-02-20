use bevy::prelude::*;

const TILE_SIZE: f32 = 16.0;

fn level_one() -> Vec<Vec<i32>> {
    vec![
        vec![0, 0, 0, 0, 0, 0, 0],
        vec![8, 8, 8, 8, 8, 8, 0],
        vec![8, 4, 0, 2, 1, 8, 0],
        vec![8, 8, 8, 0, 0, 8, 0],
        vec![0, 0, 8, 8, 8, 8, 0],
    ]
}

fn level_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let level = level_one();

    let floor_texture: Handle<Image> = asset_server.load("floor.png");
    let wall_texture: Handle<Image> = asset_server.load("wall.png");
    let goal_texture: Handle<Image> = asset_server.load("goal.png");
    let block_texture: Handle<Image> = asset_server.load("block.png");
    let player_texture: Handle<Image> = asset_server.load("player.png");

    for (row_index, row) in level.iter().enumerate() {
        for (col_index, col) in row.iter().enumerate() {
            let position = Vec2::new(col_index as f32 * TILE_SIZE, row_index as f32 * TILE_SIZE);

            commands.spawn(SpriteBundle {
                texture: floor_texture.clone(),
                transform: Transform::from_translation(position.extend(0.0)),
                ..default()
            });

            match col {
                1 => {
                    commands.spawn(SpriteBundle {
                        texture: player_texture.clone(),
                        transform: Transform::from_translation(position.extend(1.0)),
                        ..default()
                    });
                }
                2 => {
                    commands.spawn(SpriteBundle {
                        texture: block_texture.clone(),
                        transform: Transform::from_translation(position.extend(1.0)),
                        ..default()
                    });
                }
                4 => {
                    commands.spawn(SpriteBundle {
                        texture: goal_texture.clone(),
                        transform: Transform::from_translation(position.extend(1.0)),
                        ..default()
                    });
                }
                8 => {
                    commands.spawn(SpriteBundle {
                        texture: wall_texture.clone(),
                        transform: Transform::from_translation(position.extend(1.0)),
                        ..default()
                    });
                }
                0 | _ => {}
            }
        }
    }
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    window: WindowDescriptor {
                        title: "Sokoban!".to_string(),
                        width: 500.0,
                        height: 500.0,
                        ..default()
                    },
                    ..default()
                }),
        )
        .add_system(bevy::window::close_on_esc)
        .add_startup_system(level_setup)
        .run();
}
