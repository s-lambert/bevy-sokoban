use bevy::{prelude::*, sprite::Anchor};

use crate::Position;

pub fn spawn_floor(asset_server: &Res<AssetServer>, position: Position) -> SpriteBundle {
    let floor_translation = position.to_translation_z(0.0);

    SpriteBundle {
        sprite: Sprite {
            anchor: Anchor::TopLeft,
            ..default()
        },
        texture: asset_server.load("floor.png"),
        transform: Transform::from_translation(floor_translation),
        ..default()
    }
}
