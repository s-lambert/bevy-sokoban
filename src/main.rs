use bevy::{prelude::*, utils::HashMap};

const TILE_SIZE: f32 = 16.0;

#[derive(Component, Copy, Clone, Eq, Hash, PartialEq)]
struct Position {
    x: i32,
    y: i32,
}

enum Obstacle {
    Block,
    Wall,
}

#[derive(Resource, Deref, DerefMut)]
struct Obstacles(HashMap<Position, (Entity, Obstacle)>);

#[derive(Component)]
struct Player {
    is_moving: bool,
    move_timer: Timer,
    position: Position,
}

#[derive(Component)]
struct Moving {
    from: Position,
    to: Position,
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

fn position_to_translation(position: Position) -> Vec3 {
    Vec3::new(
        position.x as f32 * TILE_SIZE,
        position.y as f32 * -TILE_SIZE,
        2.0,
    )
}

fn level_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let level = level_one();
    let mut obstacles = Obstacles(HashMap::default());

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
                    let player_position = Position {
                        x: col_index as i32,
                        y: row_index as i32,
                    };
                    commands.spawn((
                        Player {
                            position: player_position,
                            is_moving: false,
                            move_timer: Timer::from_seconds(0.3, TimerMode::Once),
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
                    let block_id = commands
                        .spawn(SpriteBundle {
                            texture: block_texture.clone(),
                            transform: Transform::from_translation(position.extend(1.0)),
                            ..default()
                        })
                        .id();
                    obstacles.insert(
                        Position {
                            x: col_index as i32,
                            y: row_index as i32,
                        },
                        (block_id, Obstacle::Block),
                    );
                }
                4 => {
                    commands.spawn(SpriteBundle {
                        texture: goal_texture.clone(),
                        transform: Transform::from_translation(position.extend(1.0)),
                        ..default()
                    });
                }
                8 => {
                    let wall_id = commands
                        .spawn(SpriteBundle {
                            texture: wall_texture.clone(),
                            transform: Transform::from_translation(position.extend(1.0)),
                            ..default()
                        })
                        .id();
                    obstacles.insert(
                        Position {
                            x: col_index as i32,
                            y: row_index as i32,
                        },
                        (wall_id, Obstacle::Wall),
                    );
                }
                0 | _ => {}
            }
        }
    }

    commands.insert_resource(obstacles);
}

fn start_moving(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    obstacles: Res<Obstacles>,
    mut player_query: Query<(Entity, &mut Player)>,
) {
    let Some((player_entity, mut player)) = player_query.iter_mut().next() else { return };
    if player.is_moving {
        return;
    }

    let mut input: Option<(i32, i32)> = None;
    if keyboard_input.pressed(KeyCode::Up) {
        input = Some((0, -1));
    } else if keyboard_input.pressed(KeyCode::Down) {
        input = Some((0, 1));
    } else if keyboard_input.pressed(KeyCode::Left) {
        input = Some((-1, 0));
    } else if keyboard_input.pressed(KeyCode::Right) {
        input = Some((1, 0));
    }

    let Some(move_dir) = input else { return };
    let move_to = Position {
        x: player.position.x + move_dir.0,
        y: player.position.y + move_dir.1,
    };

    match obstacles.get(&move_to) {
        Some((_, Obstacle::Wall)) => return,
        Some((block_entity, Obstacle::Block)) => {
            let block_move_to = Position {
                x: move_to.x + move_dir.0,
                y: move_to.y + move_dir.1,
            };
            if obstacles.contains_key(&block_move_to) {
                return;
            }
            commands.entity(*block_entity).insert(Moving {
                from: move_to.clone(),
                to: block_move_to,
            });
        }
        _ => {}
    }

    player.is_moving = true;
    commands.entity(player_entity).insert(Moving {
        from: player.position.clone(),
        to: move_to,
    });
}

fn lerp(x: f32, y: f32, t: f32) -> f32 {
    (1.0 - t) * x + t * y
}

fn lerpv(a: Vec3, b: Vec3, t: f32) -> Vec3 {
    Vec3::new(lerp(a.x, b.x, t), lerp(a.y, b.y, t), lerp(a.z, b.z, t))
}

fn move_objects(
    time: Res<Time>,
    mut commands: Commands,
    mut obstacles: ResMut<Obstacles>,
    mut player_query: Query<(Entity, &mut Player)>,
    mut moving_query: Query<(Entity, &Moving, &mut Transform)>,
) {
    let Some((player_entity, mut player)) = player_query.iter_mut().next() else { return };
    if !player.is_moving {
        return;
    }

    player.move_timer.tick(time.delta());
    if player.move_timer.finished() {
        player.move_timer.reset();
        player.is_moving = false;
        for (entity, moving, mut transform) in &mut moving_query {
            transform.translation = position_to_translation(moving.to);
            commands.entity(entity).remove::<Moving>();
            if entity == player_entity {
                player.position = moving.to;
            }

            let Some(obstacle) = obstacles.remove(&moving.from) else { continue };
            obstacles.insert(moving.to, obstacle);
        }
    } else {
        for (_entity, moving, mut transform) in &mut moving_query {
            transform.translation = lerpv(
                position_to_translation(moving.from),
                position_to_translation(moving.to),
                player.move_timer.percent(),
            );
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
        .add_system(start_moving)
        .add_system(move_objects)
        .add_startup_system(level_setup)
        .run();
}
