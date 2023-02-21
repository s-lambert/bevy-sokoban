use bevy::prelude::*;

const TILE_SIZE: f32 = 16.0;

#[derive(Component)]
struct Player {
    is_moving: bool,
    move_timer: Timer,
    position: Vec2,
}

#[derive(Component)]
struct Moving {
    from: Vec2,
    to: Vec2,
}

fn level_one() -> Vec<Vec<i32>> {
    vec![
        vec![0, 0, 0, 0, 0, 0, 0],
        vec![8, 8, 8, 8, 8, 8, 0],
        vec![8, 4, 0, 2, 1, 8, 0],
        vec![8, 8, 8, 0, 0, 8, 0],
        vec![0, 0, 8, 8, 8, 8, 0],
    ]
}

fn position_to_translation(position: Vec2) -> Vec3 {
    Vec3::new(position.x * TILE_SIZE, -position.y * TILE_SIZE, 2.0)
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
            let position = Vec2::new(
                col_index as f32 * TILE_SIZE,
                -(row_index as f32 * TILE_SIZE),
            );

            commands.spawn(SpriteBundle {
                texture: floor_texture.clone(),
                transform: Transform::from_translation(position.extend(0.0)),
                ..default()
            });

            match col {
                1 => {
                    let player_position = Vec2::new(col_index as f32, row_index as f32);
                    commands.spawn((
                        Player {
                            position: Vec2::new(col_index as f32, row_index as f32),
                            is_moving: false,
                            move_timer: Timer::from_seconds(0.5, TimerMode::Once),
                        },
                        SpriteBundle {
                            texture: player_texture.clone(),
                            transform: Transform::from_translation(position_to_translation(
                                player_position,
                            )),
                            ..default()
                        },
                    ));
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

fn start_moving(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    mut player_query: Query<(Entity, &mut Player)>,
) {
    let Some((player_entity, mut player)) = player_query.iter_mut().next() else { return };
    if player.is_moving {
        return;
    }

    if keyboard_input.pressed(KeyCode::Up) {
        player.is_moving = true;
        commands.entity(player_entity).insert(Moving {
            from: player.position.clone(),
            to: Vec2::new(player.position.x, player.position.y - 1.0),
        });
    } else if keyboard_input.pressed(KeyCode::Down) {
        player.is_moving = true;
        commands.entity(player_entity).insert(Moving {
            from: player.position.clone(),
            to: Vec2::new(player.position.x, player.position.y + 1.0),
        });
    } else if keyboard_input.pressed(KeyCode::Left) {
        player.is_moving = true;
        commands.entity(player_entity).insert(Moving {
            from: player.position.clone(),
            to: Vec2::new(player.position.x - 1.0, player.position.y),
        });
    } else if keyboard_input.pressed(KeyCode::Right) {
        player.is_moving = true;
        commands.entity(player_entity).insert(Moving {
            from: player.position.clone(),
            to: Vec2::new(player.position.x + 1.0, player.position.y),
        });
    }
}

fn lerp(x: f32, y: f32, t: f32) -> f32 {
    (1.0 - t) * x + t * y
}

fn lerpv(a: Vec2, b: Vec2, t: f32) -> Vec2 {
    Vec2::new(lerp(a.x, b.x, t), lerp(a.y, b.y, t))
}

fn move_objects(
    time: Res<Time>,
    mut commands: Commands,
    mut player_query: Query<(Entity, &mut Player, &Moving, &mut Transform)>,
) {
    let Some((player_entity, mut player, moving, mut transform)) = player_query.iter_mut().next() else { return };
    if !player.is_moving {
        return;
    }

    player.move_timer.tick(time.delta());
    if player.move_timer.finished() {
        player.move_timer.reset();
        player.is_moving = false;
        player.position = moving.to;
        transform.translation = position_to_translation(moving.to);
        commands.entity(player_entity).remove::<Moving>();
    } else {
        transform.translation =
            position_to_translation(lerpv(moving.from, moving.to, player.move_timer.percent()));
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
        .add_system(start_moving)
        .add_system(move_objects)
        .add_startup_system(level_setup)
        .run();
}
