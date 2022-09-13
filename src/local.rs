use potion::player::{PlayerEvent, PlayerInputPlugin};

use bevy::{asset::AssetServerSettings, prelude::*};

fn main() {
    let mut app = App::new();
    app.insert_resource(sabi::Local);
    potion::setup_app(&mut app);
    app.add_plugin(PlayerInputPlugin);
    app.insert_resource(AssetServerSettings {
        watch_for_changes: true,
        ..default()
    });
    app.add_startup_system(spawn_local_player);

    app.run();
}

fn spawn_local_player(
    mut spawn_player: EventWriter<PlayerEvent>,
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
) {
    spawn_player.send(PlayerEvent::Spawn { id: 1 });
    spawn_player.send(PlayerEvent::SetupLocal { id: 1 });
    println!("yessir");
    info!("spawning new player");
    commands.insert_resource(AmbientLight {
        color: Color::ALICE_BLUE,
        brightness: 0.72,
    });

    const HALF_SIZE: f32 = 10.0;
    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            // Configure the projection to better fit the scene
            shadow_projection: OrthographicProjection {
                left: -HALF_SIZE,
                right: HALF_SIZE,
                bottom: -HALF_SIZE,
                top: HALF_SIZE,
                near: -10.0 * HALF_SIZE,
                far: 10.0 * HALF_SIZE,
                ..default()
            },
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4),
            ..default()
        },
        ..default()
    });
}
