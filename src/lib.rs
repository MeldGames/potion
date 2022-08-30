pub mod cauldron;
pub mod deposit;
pub mod diagnostics;
pub mod egui;
pub mod follow;
pub mod network;
pub mod physics;
pub mod player;
pub mod store;

use std::f32::consts::PI;

use bevy_egui::EguiPlugin;
use bevy_embedded_assets::EmbeddedAssetPlugin;
use bevy_inspector_egui_rapier::InspectableRapierPlugin;
use bevy_mod_outline::{Outline, OutlinePlugin};
use bevy_rapier3d::prelude::*;
use cauldron::{CauldronPlugin, Ingredient};
use deposit::DepositPlugin;

use follow::Follow;
use player::PlayerInput;
use sabi::stage::NetworkSimulationAppExt;
use store::{SecurityCheck, StoreItem, StorePlugin};

//use crate::network::NetworkPlugin;
use crate::player::PlayerPlugin;

use bevy::prelude::*;
use bevy_inspector_egui::InspectableRegistry;
use bevy_prototype_debug_lines::*;

pub const DEFAULT_FRICTION: Friction = Friction::coefficient(0.5);
pub const TICK_RATE: std::time::Duration = sabi::prelude::tick_hz(100);

pub fn setup_app(app: &mut App) {
    //app.insert_resource(bevy::ecs::schedule::ReportExecutionOrderAmbiguities);
    app.add_plugins_with(DefaultPlugins, |group| {
        group.add_before::<bevy::asset::AssetPlugin, _>(EmbeddedAssetPlugin)
    });
    app.add_plugin(EguiPlugin);
    app.add_plugin(DebugLinesPlugin::default());
    app.add_plugin(crate::egui::SetupEguiPlugin);
    app.add_plugin(bevy_editor_pls::EditorPlugin);
    app.add_plugin(sabi::plugin::SabiPlugin::<PlayerInput> {
        tick_rate: TICK_RATE,
        phantom: std::marker::PhantomData,
    });

    app.insert_resource(InspectableRegistry::default());

    app.insert_resource(bevy_framepace::FramepaceSettings {
        warn_on_frame_drop: false,
        ..default()
    });
    app.add_plugin(bevy_framepace::FramepacePlugin);
    app.add_plugin(crate::network::NetworkPlugin);
    app.insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.3)))
        .insert_resource(WindowDescriptor {
            title: "Brewalized".to_string(),
            width: 1600.,
            height: 900.,
            cursor_visible: true,
            cursor_locked: false,
            present_mode: bevy::window::PresentMode::Immediate,
            ..Default::default()
        })
        .add_plugin(PlayerPlugin)
        .add_plugin(CauldronPlugin)
        .add_plugin(StorePlugin)
        .add_plugin(DepositPlugin)
        .add_plugin(crate::physics::PhysicsPlugin)
        .add_plugin(RapierDebugRenderPlugin {
            depth_test: true,
            style: Default::default(),
            mode: DebugRenderMode::COLLIDER_SHAPES,
        })
        .add_plugin(bevy::diagnostic::DiagnosticsPlugin)
        .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        .add_plugin(crate::diagnostics::DiagnosticsEguiPlugin);
    app.add_plugin(OutlinePlugin);
    app.add_system(outline_meshes);
    app.add_startup_system(setup_map);
    app.add_event::<AssetEvent<Mesh>>();
    app.add_system(update_level_collision);
    app.add_network_system(crate::player::teleport_player_back);

    app.add_plugin(InspectableRapierPlugin);
    app.add_plugin(crate::player::CustomWanderlustPlugin);
}

fn outline_meshes(
    mut commands: Commands,
    mut outlines: ResMut<Assets<Outline>>,
    meshes: ResMut<Assets<Mesh>>,
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
    _materials: ResMut<Assets<StandardMaterial>>,
    _assets: Res<AssetServer>,
) {
    commands
        .spawn()
        .insert_bundle(SceneBundle {
            scene: asset_server.load("models/ground.glb#Scene0"),
            ..default()
        })
        .add_children(|children| {
            children
                .spawn_bundle(TransformBundle::from_transform(Transform::from_xyz(
                    0.0, -10.0, 0.0,
                )))
                .insert_bundle((
                    RigidBody::Fixed,
                    Collider::cuboid(50.0, 10.0, 50.0),
                    Name::new("Plane"),
                    crate::physics::TERRAIN_GROUPING,
                    DEFAULT_FRICTION,
                ));
        });

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
        Transform {
            translation: Vec3::new(5.0, 3.0, 0.0),
            ..default()
        },
    );

    crate::deposit::spawn_deposit_box(
        &mut commands,
        &*asset_server,
        &mut meshes,
        Transform::from_xyz(4.0, 3.0, -2.0),
    );

    let _stone = commands
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
            Collider::cuboid(0.3, 0.3, 0.3),
            RigidBody::Dynamic,
            StoreItem,
            ExternalImpulse::default(),
            Name::new("Stone"),
            Velocity::default(),
            DEFAULT_FRICTION,
        ))
        .id();

    let _donut = commands
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
            DEFAULT_FRICTION,
        ))
        .id();

    let _prallet = commands
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
            DEFAULT_FRICTION,
        ))
        .id();

    let _thorns = commands
        .spawn_bundle(SceneBundle {
            scene: asset_server.load("models/thorns.glb#Scene0"),
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
            Name::new("Thorns"),
            Velocity::default(),
            DEFAULT_FRICTION,
        ))
        .id();

    let _welt = commands
        .spawn_bundle(SceneBundle {
            scene: asset_server.load("models/weltberry.glb#Scene0"),
            transform: Transform {
                translation: Vec3::new(-2.5, 2.3, -0.075),
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
            Name::new("Weltberry"),
            Velocity::default(),
            DEFAULT_FRICTION,
        ))
        .id();

    
        let tree_positions = vec![
            Vec3::new(12.5, 0., -0.075),
            Vec3::new(16.5, 0., 3.),
            Vec3::new(20.5, 0., -4.),
            Vec3::new(26.5, 0., 2.),
        ];
        for i in tree_positions{
            let tree = commands
            .spawn_bundle(SceneBundle {
                scene: asset_server.load("models/tree.gltf#Scene0"),
                transform: Transform {
                    translation: i.clone(),
                    scale: Vec3::splat(1.),
                    ..default()
                },
                ..default()
            })
            .insert_bundle((
                ColliderMassProperties::Density(5.0),
                RigidBody::Fixed,
                Collider::cylinder(3.4, 0.2),
                Name::new("Cauldron"),
                crate::physics::TERRAIN_GROUPING,
            ))
            .id();
            commands
                .spawn_bundle(SceneBundle {
                    scene: asset_server.load("models/weltberry.glb#Scene0"),
                    transform: Transform {
                        translation: i.clone(),
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
                    Name::new("Weltberry"),
                    Velocity::default(),
                    DEFAULT_FRICTION,
                ));
        }

    let level_collision_mesh2: Handle<Mesh> = asset_server.load("models/door.glb#Mesh0/Primitive0");

    let level_collision_mesh: Handle<Mesh> =
        asset_server.load("models/walls_shop1.glb#Mesh0/Primitive0");

    let scale = Vec3::new(2.0, 2.5, 2.0);
    let walls = commands
        .spawn_bundle(SceneBundle {
            scene: asset_server.load("models/walls_shop1.glb#Scene0"),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, -10.0),
                scale: scale,
                ..default()
            },
            ..default()
        })
        .insert_bundle((
            Collider::cuboid(1.0, 1.0, 1.0),
            RigidBody::Fixed,
            Name::new("Walls Shop"),
            Velocity::default(),
        ))
        .insert(ColliderLoad)
        .insert(level_collision_mesh)
        .id();

    let security_check = commands
        .spawn_bundle(TransformBundle::from_transform(Transform::from_xyz(
            1.1, 1.0, 0.5,
        )))
        .insert_bundle((
            Collider::cuboid(0.5, 1.0, 0.5),
            RigidBody::Fixed,
            Sensor,
            SecurityCheck { push: -Vec3::Z },
            Name::new("Security Check"),
        ))
        .id();

    let shop_follower = commands
        .spawn_bundle(TransformBundle::default())
        .insert_bundle(Follow::all(walls))
        .insert(Name::new("Shop Followers"))
        .add_child(security_check)
        .id();

    let mut hinge_joint = RevoluteJointBuilder::new(Vec3::Y)
        .local_anchor1(Vec3::new(0.7, 0.02, 0.15) * scale)
        .local_anchor2(Vec3::new(0.7, 0.0, 0.13) * scale)
        //.limits([-PI / 2.0 - PI / 8.0, PI / 2.0 + PI / 8.0])
        //.limits([-PI / 2.0 - PI / 8.0, 0.0])
        .limits([0.0, PI / 2.0 + PI / 8.0])
        .build();

    hinge_joint.set_contacts_enabled(false);

    let _door = commands
        .spawn_bundle(SceneBundle {
            scene: asset_server.load("models/door.glb#Scene0"),
            transform: Transform {
                scale: scale,
                ..default()
            },
            ..default()
        })
        .insert_bundle((
            Collider::cuboid(1.0, 1.0, 1.0),
            RigidBody::Dynamic,
            Name::new("Door"),
            Velocity::default(),
            DEFAULT_FRICTION,
        ))
        .insert(ColliderLoad)
        .insert(level_collision_mesh2)
        .insert(ImpulseJoint::new(walls, hinge_joint))
        .id();

    // Bounds
    commands
        .spawn()
        .insert_bundle(TransformBundle::from_transform(Transform::from_xyz(
            0.0, 10.0, 50.0,
        )))
        .insert_bundle((
            RigidBody::Fixed,
            Collider::cuboid(50.0, 20.0, 1.0),
            Name::new("Bound Wall"),
            crate::physics::TERRAIN_GROUPING,
        ));
    commands
        .spawn()
        .insert_bundle(TransformBundle::from_transform(Transform::from_xyz(
            0.0, 10.0, -50.0,
        )))
        .insert_bundle((
            RigidBody::Fixed,
            Collider::cuboid(50.0, 20.0, 1.0),
            Name::new("Bound Wall"),
            crate::physics::TERRAIN_GROUPING,
        ));

    commands
        .spawn()
        .insert_bundle(TransformBundle::from_transform(Transform::from_xyz(
            50.0, 10.0, 0.0,
        )))
        .insert_bundle((
            RigidBody::Fixed,
            Collider::cuboid(1.0, 20.0, 50.0),
            Name::new("Bound Wall"),
            crate::physics::TERRAIN_GROUPING,
        ));
    commands
        .spawn()
        .insert_bundle(TransformBundle::from_transform(Transform::from_xyz(
            -50.0, 10.0, 0.0,
        )))
        .insert_bundle((
            RigidBody::Fixed,
            Collider::cuboid(1.0, 20.0, 50.0),
            Name::new("Bound Wall"),
            crate::physics::TERRAIN_GROUPING,
        ));
}

#[derive(Debug, Component, Clone, Copy)]
pub struct ColliderLoad;

fn update_level_collision(
    mut commands: Commands,
    mut ev_asset: EventReader<AssetEvent<Mesh>>,
    mut assets: ResMut<Assets<Mesh>>,
    mut replace: Query<(&mut Collider, &Handle<Mesh>, Entity), With<ColliderLoad>>,
) {
    for ev in ev_asset.iter() {
        match ev {
            AssetEvent::Created { handle } => {
                let loaded_mesh = assets.get_mut(handle).unwrap();
                for (mut col, inner_handle, e) in replace.iter_mut() {
                    if *inner_handle == *handle {
                        *col = Collider::from_bevy_mesh(
                            loaded_mesh,
                            &ComputedColliderShape::ConvexDecomposition(VHACDParameters::default()),
                        )
                        .unwrap();
                        commands.entity(e).remove::<ColliderLoad>();
                    }
                }
            }
            AssetEvent::Modified { handle: _ } => {}
            AssetEvent::Removed { handle: _ } => {}
        }
    }
}
