use bevy::{prelude::*, sprite::Anchor, utils::HashMap};

use crate::{GameState, Position, TILE_SIZE};

pub struct EditPlugin;

#[derive(Resource, Default)]
struct EditingState {
    floors: HashMap<Position, Entity>,
    walls: HashMap<Position, Entity>,
    blocks: HashMap<Position, Entity>,
    goals: HashMap<Position, Entity>,
    player: Option<(Position, Entity)>,
}

impl EditingState {
    fn can_place(&self, position: &Position) -> bool {
        self.floors.contains_key(position)
            && !self.blocks.contains_key(position)
            && !self.goals.contains_key(position)
            && (self.player.is_none() || &self.player.unwrap().0 != position)
    }
}

#[derive(Component)]
struct Cursor {
    action_timer: Timer,
}

fn remove_level(mut commands: Commands, everything_query: Query<Entity>) {
    for entity in everything_query.iter() {
        commands.entity(entity).despawn();
    }
}

fn show_cursor(mut commands: Commands, asset_server: Res<AssetServer>) {
    let camera_position = Vec3::new(TILE_SIZE / 2.0, -(TILE_SIZE) / 2.0, 1000.0);
    commands.spawn(Camera2dBundle {
        transform: Transform {
            translation: camera_position,
            scale: Vec3::new(0.5, 0.5, 1.0),
            ..default()
        },
        ..default()
    });

    commands.spawn((
        Cursor {
            action_timer: Timer::from_seconds(0.2, TimerMode::Once),
        },
        SpriteBundle {
            sprite: Sprite {
                anchor: Anchor::TopLeft,
                ..default()
            },
            texture: asset_server.load("cursor.png"),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 2.0)),
            ..default()
        },
    ));

    commands.insert_resource(EditingState::default());
}

fn handle_edit_input(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut editing_state: ResMut<EditingState>,
    mut cursor_query: Query<(&mut Cursor, &mut Transform)>,
) {
    let Some((mut cursor, mut transform)) = cursor_query.iter_mut().next() else { return };

    if !cursor.action_timer.finished() {
        cursor.action_timer.tick(time.delta());
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

    let mut cursor_position = Position::from_translation(transform.translation);

    if let Some((move_x, move_y)) = movement {
        cursor.action_timer.reset();

        cursor_position = cursor_position.add(move_x, move_y);
        transform.translation = cursor_position.to_translation_z(2.0);
    }

    if keyboard_input.pressed(KeyCode::Z) && !editing_state.floors.contains_key(&cursor_position) {
        cursor.action_timer.reset();

        let mut floor_translation = transform.translation.clone();
        floor_translation.z = 0.0;

        let floor_entity = commands
            .spawn(SpriteBundle {
                sprite: Sprite {
                    anchor: Anchor::TopLeft,
                    ..default()
                },
                texture: asset_server.load("floor.png"),
                transform: Transform::from_translation(floor_translation),
                ..default()
            })
            .id();

        editing_state.floors.insert(cursor_position, floor_entity);

        if let Some(wall_entity) = editing_state.walls.get(&cursor_position) {
            commands.entity(*wall_entity).despawn();
            editing_state.walls.remove(&cursor_position);
        }

        let wall_combinations = vec![
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 1),
            (1, -1),
            (1, 0),
            (1, 1),
        ];
        for (relative_x, relative_y) in wall_combinations {
            let wall_position = cursor_position.add(relative_x, relative_y);

            if !editing_state.floors.contains_key(&wall_position)
                && !editing_state.walls.contains_key(&wall_position)
            {
                let wall_id = commands
                    .spawn(SpriteBundle {
                        sprite: Sprite {
                            anchor: Anchor::TopLeft,
                            ..default()
                        },
                        texture: asset_server.load("wall.png"),
                        transform: Transform::from_translation(wall_position.to_translation()),
                        ..default()
                    })
                    .id();
                editing_state.walls.insert(wall_position, wall_id);
            }
        }
    } else if keyboard_input.pressed(KeyCode::C) && editing_state.can_place(&cursor_position) {
        cursor.action_timer.reset();

        let block_translation = cursor_position.to_translation();

        let block_id = commands
            .spawn(SpriteBundle {
                sprite: Sprite {
                    anchor: Anchor::TopLeft,
                    ..default()
                },
                texture: asset_server.load("block.png"),
                transform: Transform::from_translation(block_translation),
                ..default()
            })
            .id();
        editing_state.blocks.insert(cursor_position, block_id);
    } else if keyboard_input.pressed(KeyCode::V) && editing_state.can_place(&cursor_position) {
        cursor.action_timer.reset();

        let goal_translation = cursor_position.to_translation();

        let goal_id = commands
            .spawn(SpriteBundle {
                sprite: Sprite {
                    anchor: Anchor::TopLeft,
                    ..default()
                },
                texture: asset_server.load("goal.png"),
                transform: Transform::from_translation(goal_translation),
                ..default()
            })
            .id();
        editing_state.goals.insert(cursor_position, goal_id);
    } else if keyboard_input.pressed(KeyCode::B) && editing_state.can_place(&cursor_position) {
        cursor.action_timer.reset();

        let player_translation = cursor_position.to_translation();

        let player_id = commands
            .spawn(SpriteBundle {
                sprite: Sprite {
                    anchor: Anchor::TopLeft,
                    ..default()
                },
                texture: asset_server.load("player.png"),
                transform: Transform::from_translation(player_translation),
                ..default()
            })
            .id();

        if editing_state.player.is_some() {
            commands.entity(editing_state.player.unwrap().1).despawn();
        }
        editing_state.player = Some((cursor_position, player_id));
    }
}

impl Plugin for EditPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(GameState::Editing)
                .with_system(remove_level)
                .with_system(show_cursor),
        )
        .add_system_set(SystemSet::on_update(GameState::Editing).with_system(handle_edit_input));
    }
}
