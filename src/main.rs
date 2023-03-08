mod edit_plugin;
mod play_plugin;
mod tiles;

use bevy::{
    prelude::*,
    sprite::Anchor,
    utils::{HashMap, HashSet},
};
use edit_plugin::EditPlugin;
use play_plugin::{LevelState, NextLevelEvent, PlayPlugin, Player, UndoStack};
use tiles::spawn_floor;

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum GameState {
    Startup,
    Playing,
    Editing,
    Paused,
}

pub const TILE_SIZE: f32 = 16.0;

#[derive(Component, Copy, Clone, Eq, Hash, PartialEq, Debug)]
pub struct Position {
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

    fn from_translation(translation: Vec3) -> Position {
        Position {
            x: (translation.x / TILE_SIZE) as i32,
            y: (-translation.y / TILE_SIZE) as i32,
        }
    }

    fn to_translation(self) -> Vec3 {
        self.to_translation_z(1.0)
    }

    fn to_translation_z(self, z: f32) -> Vec3 {
        Vec3::new(self.x as f32 * TILE_SIZE, self.y as f32 * -TILE_SIZE, z)
    }
}

#[derive(Clone, PartialEq)]
pub enum Obstacle {
    Block,
    Wall,
}

pub fn level_one() -> Vec<Vec<i32>> {
    vec![
        vec![8, 8, 8, 8, 8, 8],
        vec![8, 4, 0, 2, 1, 8],
        vec![8, 8, 8, 0, 0, 8],
        vec![0, 0, 8, 8, 8, 8],
    ]
}

pub fn level_two() -> Vec<Vec<i32>> {
    vec![
        vec![8, 8, 8, 0, 8, 8, 8, 8],
        vec![8, 4, 8, 8, 8, 2, 1, 8],
        vec![8, 2, 0, 0, 0, 0, 2, 8],
        vec![8, 0, 0, 0, 2, 0, 0, 8],
        vec![8, 8, 8, 8, 8, 8, 8, 8],
    ]
}

pub fn level_three() -> Vec<Vec<i32>> {
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

pub fn level_four() -> Vec<Vec<i32>> {
    vec![
        vec![8, 8, 8, 0, 0],
        vec![8, 1, 8, 8, 0],
        vec![8, 4, 0, 8, 8],
        vec![8, 2, 0, 0, 8],
        vec![8, 0, 0, 0, 8],
        vec![8, 8, 8, 8, 8],
    ]
}

fn get_floor_positions(
    player_position: Position,
    obstacles: HashMap<Position, (Entity, Obstacle)>,
) -> Vec<Position> {
    fn is_not_wall(obstacle: Option<(Entity, Obstacle)>) -> bool {
        obstacle.is_none() || obstacle.unwrap().1 == Obstacle::Block
    }

    let mut visited = HashSet::default();
    let mut to_visit = vec![player_position];

    while !to_visit.is_empty() {
        let current_position = to_visit.pop().unwrap();
        if visited.contains(&current_position) {
            continue;
        }
        visited.insert(current_position);

        let up_position = current_position.add(0, 1);
        if is_not_wall(obstacles.get(&up_position).cloned()) {
            to_visit.push(up_position);
        }
        let down_position = current_position.add(0, -1);
        if is_not_wall(obstacles.get(&down_position).cloned()) {
            to_visit.push(down_position);
        }
        let right_position = current_position.add(1, 0);
        if is_not_wall(obstacles.get(&right_position).cloned()) {
            to_visit.push(right_position);
        }
        let left_position = current_position.add(-1, 0);
        if is_not_wall(obstacles.get(&left_position).cloned()) {
            to_visit.push(left_position);
        }
    }

    visited.into_iter().collect()
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

    let wall_texture: Handle<Image> = asset_server.load("wall.png");
    let goal_texture: Handle<Image> = asset_server.load("goal.png");
    let block_texture: Handle<Image> = asset_server.load("block.png");
    let player_texture: Handle<Image> = asset_server.load("player.png");

    for (row_index, row) in level_layout.iter().enumerate() {
        for (col_index, col) in row.iter().enumerate() {
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
                    let position = Position {
                        x: col_index as i32,
                        y: row_index as i32,
                    };

                    let block_id = commands
                        .spawn(SpriteBundle {
                            sprite: Sprite {
                                anchor: Anchor::TopLeft,
                                ..default()
                            },
                            texture: block_texture.clone(),
                            transform: Transform::from_translation(position.to_translation()),
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
                    let position = Position {
                        x: col_index as i32,
                        y: row_index as i32,
                    };

                    let goal_id = commands
                        .spawn(SpriteBundle {
                            sprite: Sprite {
                                anchor: Anchor::TopLeft,
                                ..default()
                            },
                            texture: goal_texture.clone(),
                            transform: Transform::from_translation(position.to_translation_z(0.5)),
                            ..default()
                        })
                        .id();
                    goals.insert(position, goal_id);
                }
                8 => {
                    let position = Position {
                        x: col_index as i32,
                        y: row_index as i32,
                    };

                    let wall_id = commands
                        .spawn(SpriteBundle {
                            sprite: Sprite {
                                anchor: Anchor::TopLeft,
                                ..default()
                            },
                            texture: wall_texture.clone(),
                            transform: Transform::from_translation(position.to_translation()),
                            ..default()
                        })
                        .id();
                    obstacles.insert(position, (wall_id, Obstacle::Wall));
                }
                0 | _ => {}
            }
        }
    }

    for floor_position in get_floor_positions(player_position.unwrap(), obstacles.clone()) {
        commands.spawn(spawn_floor(&asset_server, floor_position));
    }

    commands.insert_resource(LevelState {
        current_level: level,
        obstacles: obstacles,
        goals: goals,
        player_position: player_position.unwrap(),
    });
    commands.insert_resource(UndoStack(Vec::default()));
}

fn start_playing(
    mut next_level_writer: EventWriter<NextLevelEvent>,
    mut game_state: ResMut<State<GameState>>,
) {
    next_level_writer.send(NextLevelEvent(1));
    game_state.replace(GameState::Playing).ok();
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
        .add_system_set(SystemSet::on_update(GameState::Startup).with_system(start_playing))
        .add_system_set(SystemSet::on_update(GameState::Paused).with_system(unpause_game))
        .add_plugin(PlayPlugin)
        .add_plugin(EditPlugin)
        .run();
}
