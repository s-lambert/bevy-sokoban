use crate::{
    level_four, level_one, level_setup, level_three, level_two, GameState, Obstacle, Position,
};
use bevy::{prelude::*, utils::HashMap};

pub struct PlayPlugin;

#[derive(Resource, Clone)]
pub struct LevelState {
    pub current_level: i32,
    pub obstacles: HashMap<Position, (Entity, Obstacle)>,
    pub goals: HashMap<Position, Entity>,
    pub player_position: Position,
}

// Remove default implementation and use resource_exists run condition
impl Default for LevelState {
    fn default() -> Self {
        Self {
            current_level: Default::default(),
            obstacles: Default::default(),
            goals: Default::default(),
            player_position: Position { x: 0, y: 0 },
        }
    }
}

#[derive(Resource, Deref, DerefMut, Default)]
pub struct UndoStack(pub Vec<LevelState>);

#[derive(Event)]
struct UndoEvent;

#[derive(Event)]
pub struct NextLevelEvent(pub i32);

#[derive(Component)]
pub struct Player {
    pub is_moving: bool,
    pub move_timer: Timer,
}

#[derive(Component)]
struct Moving {
    from: Position,
    to: Position,
}

fn handle_input(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    mut undo_writer: EventWriter<UndoEvent>,
    level_state: Res<LevelState>,
    mut player_query: Query<(Entity, &mut Player)>,
) {
    let Some((player_entity, mut player)) = player_query.iter_mut().next() else {
        return;
    };
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

    let Some((move_x, move_y)) = movement else {
        return;
    };
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
    if undo_reader.read().next().is_some() {
        let Some(previous_state) = undo_stack.pop() else {
            return;
        };
        *level_state = previous_state;

        let Some(player_entity) = player_query.iter().next() else {
            return;
        };
        let Ok(mut player_transform) = transform_query.get_mut(player_entity) else {
            return;
        };
        player_transform.translation = level_state.player_position.to_translation();

        for (position, (obstacle_entity, obstacle)) in level_state.obstacles.iter() {
            let Obstacle::Block = obstacle else { continue };
            let Ok(mut block_transform) = transform_query.get_mut(*obstacle_entity) else {
                continue;
            };
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
    let Some((player_entity, mut player)) = player_query.iter_mut().next() else {
        return;
    };
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

            let Some(obstacle) = level_state.obstacles.remove(&moving.from) else {
                continue;
            };
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
    almost_everything_query: Query<Entity, Without<Window>>,
    asset_server: Res<AssetServer>,
    mut next_level_reader: EventReader<NextLevelEvent>,
) {
    let Some(next_level) = next_level_reader.read().next() else {
        return;
    };
    for entity in almost_everything_query.iter() {
        commands.entity(entity).despawn();
    }

    let next_level_layout = match next_level.0 {
        1 => level_one(),
        2 => level_two(),
        3 => level_three(),
        4 => level_four(),
        _ => panic!("Level not found"),
    };
    level_setup(commands, asset_server, next_level.0, next_level_layout);
}

fn pause_game(
    mut keyboard_input: ResMut<Input<KeyCode>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        keyboard_input.reset(KeyCode::Space);
        game_state.set(GameState::Paused);
        return;
    } else if keyboard_input.just_pressed(KeyCode::E) {
        keyboard_input.reset(KeyCode::E);
        game_state.set(GameState::Editing);
    }
}

impl Plugin for PlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<UndoEvent>()
            .add_event::<NextLevelEvent>()
            .insert_resource(LevelState::default())
            .insert_resource(UndoStack::default())
            .add_systems(
                Update,
                (
                    pause_game,
                    handle_input.after(pause_game),
                    reset_state.after(handle_input),
                    move_objects.after(handle_input),
                    load_next_level.after(move_objects),
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}
