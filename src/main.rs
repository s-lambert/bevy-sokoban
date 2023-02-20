use bevy::prelude::*;

fn level_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn(
        (SpriteBundle {
            texture: asset_server.load("player.png"),
            ..default()
        }),
    );

    commands.spawn(
        (SpriteBundle {
            texture: asset_server.load("block.png"),
            ..default()
        }),
    );

    commands.spawn(
        (SpriteBundle {
            texture: asset_server.load("floor.png"),
            ..default()
        }),
    );

    commands.spawn(
        (SpriteBundle {
            texture: asset_server.load("wall.png"),
            ..default()
        }),
    );
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
