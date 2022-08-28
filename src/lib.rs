pub mod cauldron;
pub mod deposit;
pub mod diagnostics;
pub mod egui;
pub mod follow;
pub mod network;
pub mod physics;
pub mod player;
pub mod store;

use bevy_egui::EguiPlugin;
use bevy_inspector_egui_rapier::InspectableRapierPlugin;
use bevy_mod_outline::{Outline, OutlinePlugin};
use bevy_rapier3d::prelude::*;
use cauldron::{Cauldron, CauldronPlugin, Ingredient};
use deposit::DepositPlugin;
use follow::Follow;
use iyes_loopless::prelude::*;

use crate::network::NetworkPlugin;
use crate::player::{PlayerInputPlugin, PlayerPlugin};

use bevy::prelude::*;
use bevy_inspector_egui::InspectableRegistry;
use bevy_prototype_debug_lines::*;

pub fn setup_app(app: &mut App) {
    //app.insert_resource(bevy::ecs::schedule::ReportExecutionOrderAmbiguities);
    app.add_plugins(DefaultPlugins);
    app.add_plugin(EguiPlugin);
    app.add_plugin(DebugLinesPlugin::default());
    app.add_plugin(crate::egui::SetupEguiPlugin);
    app.add_plugin(bevy_editor_pls::EditorPlugin);

    app.insert_resource(InspectableRegistry::default());

    app.insert_resource(bevy_framepace::FramepaceSettings {
        warn_on_frame_drop: false,
        ..default()
    });
    app.add_plugin(bevy_framepace::FramepacePlugin);
    app.add_plugin(NetworkPlugin)
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.3)))
        .insert_resource(WindowDescriptor {
            title: "Brewification".to_string(),
            width: 800.,
            height: 600.,
            cursor_visible: false,
            cursor_locked: false,
            present_mode: bevy::window::PresentMode::Immediate,
            ..Default::default()
        })
        .add_plugin(PlayerPlugin)
        .add_plugin(CauldronPlugin)
        .add_plugin(DepositPlugin)
        .add_plugin(crate::physics::PhysicsPlugin)
        .add_plugin(RapierDebugRenderPlugin {
            depth_test: true,
            style: Default::default(),
            mode: Default::default(),
        })
        .add_plugin(bevy::diagnostic::DiagnosticsPlugin)
        .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        .add_plugin(crate::diagnostics::DiagnosticsEguiPlugin);
    app.add_plugin(OutlinePlugin);
    app.add_system(outline_meshes);
    app.add_startup_system(setup_map);

    app.add_plugin(InspectableRapierPlugin);
    app.add_plugin(crate::player::CustomWanderlustPlugin);
}

fn outline_meshes(
    mut commands: Commands,
    mut outlines: ResMut<Assets<Outline>>,
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<(Entity, &Handle<Mesh>), (With<Handle<Mesh>>, Without<Handle<Outline>>)>,
) {
    for (entity, mesh) in &query {
        if let Some(mesh) = meshes.get(mesh) {
            if mesh.contains_attribute(Mesh::ATTRIBUTE_NORMAL) {
                commands.entity(entity).insert(outlines.add(Outline {
                    colour: Color::rgba(0.0, 0.0, 0.0, 0.8),
                    width: 5.0,
                }));
            }
        }
    }
}

fn setup_map(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    assets: Res<AssetServer>,
) {
    commands
        .spawn()
        .insert_bundle(TransformBundle::from_transform(Transform::from_xyz(
            0.0, -10.0, 0.0,
        )))
        /*
        .insert_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 100.0 })),
            material: materials.add(assets.load("icons/autoattack.png").into()),
            transform: Transform {
                translation: Vec3::new(0.0, -2.00, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })*/
        .insert_bundle((
            RigidBody::Fixed,
            Collider::cuboid(50.0, 10.0, 50.0),
            Name::new("Plane"),
            crate::physics::TERRAIN_GROUPING,
        ));

    commands
        .spawn_bundle(TransformBundle::from_transform(Transform::from_xyz(
            5.0, 2.0, 5.0,
        )))
        .insert_bundle((
            RigidBody::KinematicPositionBased,
            Collider::capsule(Vec3::ZERO, Vec3::Y, 0.5),
            Name::new("Test capsule"),
            crate::physics::TERRAIN_GROUPING,
        ));

    crate::cauldron::spawn_cauldron(
        &mut commands,
        &*asset_server,
        Transform::from_xyz(-2.0, 3.0, -2.0),
    );

    crate::deposit::spawn_deposit_box(
        &mut commands,
        &*asset_server,
        &mut meshes,
        Transform::from_xyz(2.0, 3.0, -2.0),
    );

    let stone = commands
        .spawn_bundle(SceneBundle {
            scene: asset_server.load("models/rock1.glb#Scene0"),
            transform: Transform {
                translation: Vec3::new(-1.5, 1.3, 1.075),
                ..default()
            },
            ..default()
        })
        .insert(Ingredient)
        .insert(crate::deposit::Value::new(1))
        .insert_bundle((
            Collider::ball(0.3),
            RigidBody::Dynamic,
            Name::new("Stone"),
            Velocity::default(),
        ))
        .id();

    let donut = commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Torus {
                radius: 0.4,
                ring_radius: 0.2,
                ..default()
            })),
            transform: Transform::from_xyz(1.0, 2.0, -2.0),
            ..default()
        })
        .insert(crate::deposit::Value::new(5))
        .insert(Ingredient)
        .insert_bundle((
            Collider::round_cylinder(0.025, 0.4, 0.2),
            RigidBody::Dynamic,
            Name::new("Donut"),
            Velocity::default(),
        ))
        .id();

    let prallet = commands
        .spawn_bundle(SceneBundle {
            scene: asset_server.load("models/prallet.glb#Scene0"),
            transform: Transform {
                translation: Vec3::new(-2.5, 1.3, -0.075),
                scale: Vec3::splat(1.),
                ..default()
            },
            ..default()
        })
        .insert(Ingredient)
        .insert(crate::deposit::Value::new(1))
        .insert_bundle((
            Collider::ball(0.3),
            RigidBody::Dynamic,
            Name::new("Prallet"),
            Velocity::default(),
        ))
        .id();

    let thorns = commands
        .spawn_bundle(SceneBundle {
            scene: asset_server.load("models/thorns.glb#Scene0"),
            transform: Transform {
                translation: Vec3::new(-2.5, 10.3, -0.075),
                scale: Vec3::splat(10.),
                ..default()
            },
            ..default()
        })
        .insert(Ingredient)
        .insert(crate::deposit::Value::new(1))
        .insert_bundle((
            Collider::ball(0.3),
            RigidBody::Dynamic,
            Name::new("Thorns"),
            Velocity::default(),
        ))
        .id();

    
    let level_collision_mesh: Handle<Mesh> = asset_server.load("models/door.glb#Mesh0");
    
    let door = commands
    .spawn_bundle(SceneBundle {
        scene: asset_server.load("models/door.glb#Scene0"),
        transform: Transform {
            translation: Vec3::new(-10.5, 1.3, -0.075),
            scale: Vec3::splat(1.),
            ..default()
        },
        ..default()
    })
    .insert_bundle((
        Collider::cuboid(1.0, 1.0, 1.0),
        RigidBody::Dynamic,
        Name::new("Door"),
        Velocity::default(),
    ))
    .id();




    let walls = commands
    .spawn_bundle(SceneBundle {
        scene: asset_server.load("models/walls_shop1.glb#Scene0"),
        transform: Transform {
            translation: Vec3::new(-10.5, 1.3, -0.075),
            scale: Vec3::splat(1.),
            ..default()
        },
        ..default()
    })
    .insert_bundle((
        Collider::cuboid(1.0, 1.0, 1.0),
        RigidBody::Dynamic,
        Name::new("Walls Shop"),
        Velocity::default(),
    ))
    .id();
}
