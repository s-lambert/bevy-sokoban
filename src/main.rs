use bevy::{prelude::*, sprite::Anchor, utils::HashMap};

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum GameState {
    Startup,
    Playing,
    Paused,
}

const TILE_SIZE: f32 = 16.0;

#[derive(Component, Copy, Clone, Eq, Hash, PartialEq)]
struct Position {
    x: i32,
    y: i32,
}

impl Position {
    fn add(&self, x: i32, y: i32) -> Position {
        Position {
            x: self.x + x,
            y: self.y + y,
        }
    }

    fn to_translation(self) -> Vec3 {
        Vec3::new(self.x as f32 * TILE_SIZE, self.y as f32 * -TILE_SIZE, 2.0)
    }
}

#[derive(Clone)]
enum Obstacle {
    Block,
    Wall,
}

#[derive(Resource, Clone)]
struct LevelState {
    current_level: i32,
    obstacles: HashMap<Position, (Entity, Obstacle)>,
    goals: HashMap<Position, Entity>,
    player_position: Position,
}

#[derive(Resource, Deref, DerefMut)]
struct UndoStack(Vec<LevelState>);

struct UndoEvent;

struct NextLevelEvent(i32);

#[derive(Component)]
struct Player {
    is_moving: bool,
    move_timer: Timer,
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

fn level_two() -> Vec<Vec<i32>> {
    vec![
        vec![8, 8, 8, 0, 8, 8, 8, 8],
        vec![8, 4, 8, 8, 8, 2, 1, 8],
        vec![8, 2, 0, 0, 0, 0, 2, 8],
        vec![8, 0, 0, 0, 2, 0, 0, 8],
        vec![8, 8, 8, 8, 8, 8, 8, 8],
    ]
}

fn level_three() -> Vec<Vec<i32>> {
    vec![
        vec![0, 8, 8, 8, 8, 8, 8, 8, 8, 8, 0],
        vec![8, 8, 0, 0, 0, 0, 0, 0, 0, 8, 8],
        vec![8, 4, 2, 2, 0, 0, 2, 0, 2, 1, 8],
        vec![8, 2, 2, 0, 0, 0, 2, 2, 2, 2, 8],
        vec![8, 0, 0, 0, 0, 0, 0, 0, 2, 2, 8],
        vec![8, 2, 0, 0, 0, 0, 0, 0, 0, 0, 8],
        vec![8, 8, 0, 0, 0, 0, 0, 0, 0, 8, 8],
        vec![0, 8, 8, 8, 8, 8, 8, 8, 8, 8, 0],
    ]
}

fn level_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    level: i32,
    level_layout: Vec<Vec<i32>>,
) {
    let last_row_index = level_layout.len() as i32;
    let last_col_index = level_layout.first().unwrap().len() as i32;
    let camera_position = Vec3::new(
        last_col_index as f32 * TILE_SIZE / 2.0,
        -(last_row_index as f32 * TILE_SIZE) / 2.0,
        1000.0,
    );

    commands.spawn(Camera2dBundle {
        transform: Transform {
            translation: camera_position,
            scale: Vec3::new(0.5, 0.5, 1.0),
            ..default()
        },
        ..default()
    });

    let mut obstacles = HashMap::default();
    let mut goals = HashMap::default();
    let mut player_position = None;

    let floor_texture: Handle<Image> = asset_server.load("floor.png");
    let wall_texture: Handle<Image> = asset_server.load("wall.png");
    let goal_texture: Handle<Image> = asset_server.load("goal.png");
    let block_texture: Handle<Image> = asset_server.load("block.png");
    let player_texture: Handle<Image> = asset_server.load("player.png");

    for (row_index, row) in level_layout.iter().enumerate() {
        for (col_index, col) in row.iter().enumerate() {
            let position = Vec2::new(
                col_index as f32 * TILE_SIZE,
                -(row_index as f32 * TILE_SIZE),
            );

            commands.spawn(SpriteBundle {
                sprite: Sprite {
                    anchor: Anchor::TopLeft,
                    ..default()
                },
                texture: floor_texture.clone(),
                transform: Transform::from_translation(position.extend(0.0)),
                ..default()
            });

            match col {
                1 => {
                    player_position = Some(Position {
                        x: col_index as i32,
                        y: row_index as i32,
                    });
                    commands.spawn((
                        Player {
                            is_moving: false,
                            move_timer: Timer::from_seconds(0.3, TimerMode::Once),
                        },
                        SpriteBundle {
                            sprite: Sprite {
                                anchor: Anchor::TopLeft,
                                ..default()
                            },
                            texture: player_texture.clone(),
                            transform: Transform::from_translation(
                                player_position.unwrap().to_translation(),
                            ),
                            ..default()
                        },
                    ));
                }
                2 => {
                    let block_id = commands
                        .spawn(SpriteBundle {
                            sprite: Sprite {
                                anchor: Anchor::TopLeft,
                                ..default()
                            },
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
                            sprite: Sprite {
                                anchor: Anchor::TopLeft,
                                ..default()
                            },
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
                            sprite: Sprite {
                                anchor: Anchor::TopLeft,
                                ..default()
                            },
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

    commands.insert_resource(LevelState {
        current_level: level,
        obstacles: obstacles,
        goals: goals,
        player_position: player_position.unwrap(),
    });
    commands.insert_resource(UndoStack(Vec::default()));
}

fn setup_level_one(commands: Commands, asset_server: Res<AssetServer>) {
    let level = level_one();

    level_setup(commands, asset_server, 1, level);
}

fn handle_input(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    mut undo_writer: EventWriter<UndoEvent>,
    level_state: Res<LevelState>,
    mut player_query: Query<(Entity, &mut Player)>,
) {
    let Some((player_entity, mut player)) = player_query.iter_mut().next() else { return };
    if player.is_moving {
        return;
    }

    if keyboard_input.just_pressed(KeyCode::U) {
        undo_writer.send(UndoEvent);
        return;
    }

    let mut movement: Option<(i32, i32)> = None;
    if keyboard_input.pressed(KeyCode::Up) {
        movement = Some((0, -1));
    } else if keyboard_input.pressed(KeyCode::Down) {
        movement = Some((0, 1));
    } else if keyboard_input.pressed(KeyCode::Left) {
        movement = Some((-1, 0));
    } else if keyboard_input.pressed(KeyCode::Right) {
        movement = Some((1, 0));
    }

    let Some((move_x, move_y)) = movement else { return };
    let move_to = level_state.player_position.add(move_x, move_y);

    match level_state.obstacles.get(&move_to) {
        Some((_, Obstacle::Wall)) => return,
        Some((block_entity, Obstacle::Block)) => {
            let block_move_to = move_to.add(move_x, move_y);
            if level_state.obstacles.contains_key(&block_move_to) {
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
        from: level_state.player_position.clone(),
        to: move_to,
    });
}

fn reset_state(
    mut level_state: ResMut<LevelState>,
    mut undo_stack: ResMut<UndoStack>,
    mut undo_reader: EventReader<UndoEvent>,
    player_query: Query<Entity, With<Player>>,
    mut transform_query: Query<&mut Transform>,
) {
    if undo_reader.iter().next().is_some() {
        let Some(previous_state) = undo_stack.pop() else { return };
        *level_state = previous_state;

        let Some(player_entity) = player_query.iter().next() else { return };
        let Ok(mut player_transform) = transform_query.get_mut(player_entity) else { return };
        player_transform.translation = level_state.player_position.to_translation();

        for (position, (obstacle_entity, obstacle)) in level_state.obstacles.iter() {
            let Obstacle::Block = obstacle else { continue };
            let Ok(mut block_transform) = transform_query.get_mut(*obstacle_entity) else { continue };
            block_transform.translation = position.to_translation();
        }
    }
}

// Adapted from: https://github.com/godotengine/godot/blob/27b2260460ab478707d884a16429add5bb3375f1/scene/animation/easing_equations.h
fn quad_ease_out(x: f32, y: f32, d: f32) -> f32 {
    -(y - x) * d * (d - 2.0) + x
}

fn quad_ease_out_v(a: Vec3, b: Vec3, d: f32) -> Vec3 {
    Vec3::new(
        quad_ease_out(a.x, b.x, d),
        quad_ease_out(a.y, b.y, d),
        quad_ease_out(a.z, b.z, d),
    )
}

fn move_objects(
    time: Res<Time>,
    mut commands: Commands,
    mut level_state: ResMut<LevelState>,
    mut undo_stack: ResMut<UndoStack>,
    mut player_query: Query<(Entity, &mut Player)>,
    mut moving_query: Query<(Entity, &Moving, &mut Transform)>,
    mut next_level_writer: EventWriter<NextLevelEvent>,
) {
    let Some((player_entity, mut player)) = player_query.iter_mut().next() else { return };
    if !player.is_moving {
        return;
    }

    player.move_timer.tick(time.delta());
    if !player.move_timer.finished() {
        for (_entity, moving, mut transform) in &mut moving_query {
            transform.translation = quad_ease_out_v(
                moving.from.to_translation(),
                moving.to.to_translation(),
                player.move_timer.percent(),
            );
        }
    } else {
        player.move_timer.reset();
        player.is_moving = false;
        undo_stack.push(level_state.clone());
        for (entity, moving, mut transform) in &mut moving_query {
            transform.translation = moving.to.to_translation();
            commands.entity(entity).remove::<Moving>();
            if entity == player_entity {
                level_state.player_position = moving.to;
            }

            let Some(obstacle) = level_state.obstacles.remove(&moving.from) else { continue };
            level_state.obstacles.insert(moving.to, obstacle);
        }

        let has_won = level_state
            .goals
            .iter()
            .all(|(goal_position, _)| level_state.obstacles.contains_key(goal_position));
        if has_won {
            next_level_writer.send(NextLevelEvent(level_state.current_level + 1));
        }
    }
}

fn load_next_level(
    mut commands: Commands,
    everything_query: Query<Entity>,
    asset_server: Res<AssetServer>,
    mut next_level_reader: EventReader<NextLevelEvent>,
) {
    let Some(next_level) = next_level_reader.iter().next() else { return };
    for entity in everything_query.iter() {
        commands.entity(entity).despawn();
    }

    let next_level_layout = match next_level.0 {
        2 => level_two(),
        3 => level_three(),
        _ => panic!("Level not found"),
    };
    level_setup(commands, asset_server, next_level.0, next_level_layout);
}

fn start_playing(mut game_state: ResMut<State<GameState>>) {
    game_state.replace(GameState::Playing).ok();
}

fn pause_game(
    mut keyboard_input: ResMut<Input<KeyCode>>,
    mut game_state: ResMut<State<GameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        keyboard_input.reset(KeyCode::Space);
        game_state.replace(GameState::Paused).ok();
        return;
    }
}

fn unpause_game(
    mut keyboard_input: ResMut<Input<KeyCode>>,
    mut game_state: ResMut<State<GameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        keyboard_input.reset(KeyCode::Space);
        game_state.replace(GameState::Playing).ok();
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
        .add_state(GameState::Startup)
        .add_event::<UndoEvent>()
        .add_event::<NextLevelEvent>()
        .add_system_set(SystemSet::on_enter(GameState::Startup).with_system(setup_level_one))
        .add_system_set(SystemSet::on_update(GameState::Startup).with_system(start_playing))
        .add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_system(pause_game)
                .with_system(handle_input.after(pause_game))
                .with_system(reset_state.after(handle_input))
                .with_system(move_objects.after(handle_input))
                .with_system(load_next_level.after(move_objects)),
        )
        .add_system_set(SystemSet::on_update(GameState::Paused).with_system(unpause_game))
        .run();
}
