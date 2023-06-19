use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};

use bevy_editor_pls::EditorPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(FrameTimeDiagnosticsPlugin)
        .add_plugin(EditorPlugin)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_startup_system(setup)
        .run();
}

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_translation(Vec3::new(0., 10., 10.))
            .looking_at(Vec3::new(0.0, 10., 0.0), Vec3::Y),
        camera: Camera {
            is_active: true,
            ..default()
        },
        ..default()
    });

    let _ground = commands.spawn(SceneBundle {
        scene: asset_server.load("models/ground.gltf#Scene0"),
        ..default()
    });

    let _sky = commands
        .spawn(SceneBundle {
            scene: asset_server.load("models/sky.glb#Scene0"),
            transform: Transform {
                translation: Vec3::new(-1.5, 1.3, 1.075),
                ..default()
            },
            ..default()
        })
        .id();

    let _sky_clouds = commands
        .spawn(SceneBundle {
            scene: asset_server.load("models/sky_clouds.glb#Scene0"),
            transform: Transform {
                translation: Vec3::new(-1.5, 1.3, 1.075),
                ..default()
            },
            ..default()
        })
        .id();
}
