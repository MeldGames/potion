use potion::{
    player::prelude::{
        CollectInputs, InputSet, MetaInputs, PlayerEvent, PlayerInput, PlayerInputPlugin,
    },
    setup_map,
};

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

fn main() {
    let mut app = App::new();
    potion::setup_app(&mut app);
    app.add_plugin(PlayerInputPlugin);
    app.add_startup_system(spawn_local_player);
    app.add_startup_system(setup_test_map);
    app.add_system(reset_movement.after(CollectInputs).before(MetaInputs));

    app.run();
}

fn setup_test_map(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let test_texture = asset_server.load("models/materials/Placeholder.png");
    let test_material = materials.add(StandardMaterial {
        base_color_texture: Some(test_texture.clone()),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube::new(1.0))),
            material: test_material.clone(),
            transform: Transform {
                translation: Vec3::new(0.0, -3.0, 0.0),
                scale: Vec3::new(100.0, 2.0, 100.0),
                ..default()
            },
            ..default()
        })
        .insert(Name::new("Ground"))
        .insert((
            RigidBody::Fixed,
            Collider::cuboid(0.5, 0.5, 0.5),
            potion::physics::TERRAIN_GROUPING,
            potion::DEFAULT_FRICTION,
        ));

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            // Configure the projection to better fit the scene
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-0.5),
            ..default()
        },
        ..default()
    });
}

fn spawn_local_player(mut spawn_player: EventWriter<PlayerEvent>, _asset_server: Res<AssetServer>) {
    spawn_player.send(PlayerEvent::Spawn { id: 1 });
    //spawn_player.send(PlayerEvent::SetupLocal { id: 1 });
    info!("spawning new player");
}

fn reset_movement(mut input: ResMut<PlayerInput>) {
    let yaw = input.yaw;
    let pitch = input.pitch;
    *input = PlayerInput {
        yaw,
        pitch,
        extend_arm: [true; 8],
        twist: true,
        ..default()
    }
}
