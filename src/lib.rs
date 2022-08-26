pub mod cauldron;
pub mod diagnostics;
pub mod egui;
pub mod follow;
pub mod network;
pub mod physics;
pub mod player;

use bevy_egui::EguiPlugin;
use bevy_inspector_egui_rapier::InspectableRapierPlugin;
use bevy_mod_outline::{Outline, OutlinePlugin};
use bevy_rapier3d::prelude::*;
use cauldron::{Cauldron, CauldronPlugin, Ingredient};
use follow::Follow;
use iyes_loopless::prelude::*;

use crate::network::NetworkPlugin;
use crate::player::{PlayerInputPlugin, PlayerPlugin};

use bevy::prelude::*;
use bevy_inspector_egui::InspectableRegistry;

pub fn setup_app(app: &mut App) {
    //app.insert_resource(bevy::ecs::schedule::ReportExecutionOrderAmbiguities);
    app.add_plugins(DefaultPlugins);
    app.add_plugin(EguiPlugin);
    app.add_plugin(crate::egui::SetupEguiPlugin);
    app.add_plugin(bevy_editor_pls::EditorPlugin);

    app.add_plugin(bevy_mod_wanderlust::WanderlustPlugin);
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

    let cauldron_model = commands
        .spawn_bundle(SceneBundle {
            scene: asset_server.load("models/cauldron.glb#Scene0"),
            transform: Transform {
                translation: Vec3::new(-5.5, -0.3, -0.075),
                scale: Vec3::splat(1.2),
                ..default()
            },
            ..default()
        })
        .id();

    let cauldron = commands
        .spawn_bundle(TransformBundle::from_transform(Transform::from_xyz(
            -2.0, 3.0, -2.0,
        )))
        .insert_bundle((
            ColliderMassProperties::Density(15.0),
            RigidBody::Dynamic,
            Collider::cylinder(0.4, 0.75),
            Name::new("Cauldron"),
            crate::physics::TERRAIN_GROUPING,
        ))
        .insert_bundle(VisibilityBundle::default())
        .add_child(cauldron_model)
        .id();

    commands
        .spawn_bundle(TransformBundle::from_transform(Transform::from_xyz(
            -2.0, 3.0, -2.0,
        )))
        .insert_bundle(Follow::all(cauldron))
        .insert_bundle((
            Name::new("Cauldron Ingredient Hitbox"),
            crate::physics::TERRAIN_GROUPING,
        ))
        .with_children(|children| {
            children
                .spawn_bundle(TransformBundle::from_transform(Transform::from_xyz(
                    0.0, 0.25, 0.0,
                )))
                .insert(Collider::cylinder(0.4, 0.6))
                .insert(Cauldron)
                .insert(Sensor);
        });

    let stone = commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere {
                radius: 0.3,
                ..default()
            })),
            transform: Transform::from_xyz(1.0, 2.0, -1.0),
            ..default()
        })
        .insert(Ingredient)
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
        .insert(Ingredient)
        .insert_bundle((
            Collider::round_cylinder(0.025, 0.4, 0.2),
            RigidBody::Dynamic,
            Name::new("Donut"),
            Velocity::default(),
        ))
        .id();
}
