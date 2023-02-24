use bevy::{prelude::*, utils::HashMap};

const TILE_SIZE: f32 = 16.0;

#[derive(Component, Copy, Clone, Eq, Hash, PartialEq)]
struct Position {
    x: i32,
    y: i32,
}

impl Position {
    fn to_translation(self) -> Vec3 {
        Vec3::new(self.x as f32 * TILE_SIZE, self.y as f32 * -TILE_SIZE, 2.0)
    }
}

enum Obstacle {
    Block,
    Wall,
}

#[derive(Resource, Deref, DerefMut)]
struct Obstacles(HashMap<Position, (Entity, Obstacle)>);

#[derive(Resource, Deref, DerefMut)]
struct Goals(HashMap<Position, Entity>);

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
        vec![8, 8, 8, 8, 8, 8],
        vec![8, 4, 0, 2, 1, 8],
        vec![8, 8, 8, 0, 0, 8],
        vec![0, 0, 8, 8, 8, 8],
    ]
}

fn level_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let level = level_one();
    let mut obstacles = Obstacles(HashMap::default());
    let mut goals = Goals(HashMap::default());

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
                            transform: Transform::from_translation(
                                player_position.to_translation(),
                            ),
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
                    let goal_id = commands
                        .spawn(SpriteBundle {
                            texture: goal_texture.clone(),
                            transform: Transform::from_translation(position.extend(1.0)),
                            ..default()
                        })
                        .id();
                    goals.insert(
                        Position {
                            x: col_index as i32,
                            y: row_index as i32,
                        },
                        goal_id,
                    );
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
    commands.insert_resource(goals);
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

// Adapted from: https://github.com/godotengine/godot/blob/27b2260460ab478707d884a16429add5bb3375f1/scene/animation/easing_equations.h
fn quad(x: f32, y: f32, d: f32) -> f32 {
    (y - x) * d.powf(2.0) + x
}

fn quadv(a: Vec3, b: Vec3, d: f32) -> Vec3 {
    Vec3::new(quad(a.x, b.x, d), quad(a.y, b.y, d), quad(a.z, b.z, d))
}

fn move_objects(
    time: Res<Time>,
    mut commands: Commands,
    mut obstacles: ResMut<Obstacles>,
    goals: Res<Goals>,
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
            transform.translation = moving.to.to_translation();
            commands.entity(entity).remove::<Moving>();
            if entity == player_entity {
                player.position = moving.to;
            }

            let Some(obstacle) = obstacles.remove(&moving.from) else { continue };
            obstacles.insert(moving.to, obstacle);
        }
    } else {
        for (_entity, moving, mut transform) in &mut moving_query {
            transform.translation = quadv(
                moving.from.to_translation(),
                moving.to.to_translation(),
                player.move_timer.percent(),
            );
        }
    }

    let has_won = goals
        .iter()
        .all(|(goal_position, _)| obstacles.contains_key(goal_position));
    if has_won {
        println!("Level complete!");
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
