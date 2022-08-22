//use bevy_editor_pls::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_fly_camera::FlyCamera;
use bevy_inspector_egui::InspectableRegistry;
use bevy_rapier3d::prelude::*;
use iyes_loopless::prelude::*;
use potion::network::NetworkPlugin;
use potion::player::{LockToggle, MouseState, PlayerPlugin};

use bevy::prelude::*;
//use bevy_fly_camera::{camera_movement_system, mouse_motion_system, FlyCamera};

use potion::player::{mouse_lock, toggle_mouse_lock, window_focused};
use wgpu_types::PrimitiveTopology;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new();
    app.insert_resource(sabi::Server);
    potion::setup_app(&mut app);
    //app.insert_resource(bevy::ecs::schedule::ReportExecutionOrderAmbiguities);

    app.add_loopless_state(MouseState::Locked);
    app.insert_resource(LockToggle::default());

    app.add_system(
        bevy_fly_camera::camera_movement_system
            .run_if(window_focused)
            .label("player_fly_movement"),
    )
    .add_system(
        bevy_fly_camera::mouse_motion_system
            .run_in_state(MouseState::Locked)
            .run_if(window_focused)
            .label("player_mouse_input"),
    );
    app.add_system(
        toggle_mouse_lock
            .run_if(window_focused)
            .label("toggle_mouse_lock"),
    )
    .add_system(mouse_lock.run_if(window_focused).label("toggle_mouse_lock"));

    app.add_startup_system(setup_camera);
    app.add_startup_system(setup_map);
    app.add_system(rotate);

    #[cfg(feature = "public")]
    let ip = sabi::protocol::public_ip()?;
    #[cfg(not(feature = "public"))]
    let ip = sabi::protocol::localhost_ip();

    let new_server: bevy_renet::renet::RenetServer =
        sabi::protocol::new_renet_server(ip, None, sabi::protocol::PORT)
            .expect("could not make new server");
    app.insert_resource(new_server);

    app.run();

    Ok(())
}

fn setup_camera(mut commands: Commands, _asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(0., 12., 10.))
                .looking_at(Vec3::new(0.0, 0.3, 0.0), Vec3::Y),
            ..Default::default()
        })
        .insert(FlyCamera::default());

    commands.spawn_bundle(SceneBundle {
        scene: _asset_server.load("models/cauldron.glb#Scene0"),
        ..default()
    });
}

fn setup_map(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    assets: Res<AssetServer>,
) {
    commands
        .spawn()
        .insert_bundle((GlobalTransform::default(), Transform::default()))
        .insert_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 100.0 })),
            material: materials.add(assets.load("icons/autoattack.png").into()),
            transform: Transform {
                translation: Vec3::new(0.0, -0.01, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert_bundle((
            RigidBody::Fixed,
            Collider::cuboid(50.0, 0.1, 50.0),
            Name::new("Plane"),
            potion::physics::TERRAIN_GROUPING,
        ));

    commands
        .spawn()
        .insert_bundle((GlobalTransform::default(), Transform::default(), Rotate))
        .with_children(|child| {
            child
                .spawn_bundle(TransformBundle::default())
                .insert_bundle((
                    RigidBody::KinematicPositionBased,
                    //Collider::capsule(Vec3::ZERO, Vec3::Y, 0.5),
                    //Collider::ball(1.0),
                    Name::new("Test capsule"),
                    potion::physics::TERRAIN_GROUPING,
                ));
        })
        .insert_bundle(());
}

#[derive(Debug, Clone, Component)]
pub struct Rotate;

pub fn rotate(time: Res<Time>, mut to_rotate: Query<&mut Transform, With<Rotate>>) {
    for mut transform in &mut to_rotate {
        transform.rotation =
            Quat::from_axis_angle(Vec3::Y, time.time_since_startup().as_secs_f32() * 0.01).into();
    }
}
