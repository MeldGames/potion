pub mod attach;
pub mod cauldron;
pub mod deposit;
pub mod diagnostics;
pub mod egui;
pub mod joint_break;
pub mod network;
pub mod physics;
pub mod player;
pub mod store;
pub mod trees;

use std::{f32::consts::PI, fs::File, io::BufReader};

use bevy_egui::EguiPlugin;
use bevy_embedded_assets::EmbeddedAssetPlugin;
use bevy_inspector_egui_rapier::InspectableRapierPlugin;
use bevy_mod_outline::{Outline, OutlinePlugin};
use bevy_rapier3d::prelude::*;
use cauldron::{CauldronPlugin, Ingredient};
use deposit::DepositPlugin;
use joint_break::{BreakJointPlugin, BreakableJoint};
use obj::Obj;
use trees::TreesPlugin;

use attach::{Attach, AttachTranslation};
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
        .add_plugin(BreakJointPlugin)
        .add_plugin(TreesPlugin)
        .add_plugin(crate::physics::PhysicsPlugin)
        /*
        .add_plugin(RapierDebugRenderPlugin {
            depth_test: true,
            style: Default::default(),
            mode: DebugRenderMode::COLLIDER_SHAPES,
        }) */
        .add_plugin(bevy::diagnostic::DiagnosticsPlugin)
        .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        .add_plugin(crate::diagnostics::DiagnosticsEguiPlugin);
    app.add_plugin(OutlinePlugin);
    app.add_system(outline_meshes);

    app.add_event::<AssetEvent<Mesh>>();

    app.add_startup_system(setup_map);
    app.add_system(update_level_collision);
    app.add_system(decomp_load);
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

    let cauldron = crate::cauldron::spawn_cauldron(
        &mut commands,
        &*asset_server,
        Transform {
            translation: Vec3::new(5.0, 4.0, 0.0),
            scale: Vec3::splat(2.),
            ..default()
        },
    );

    crate::deposit::spawn_deposit_box(
        &mut commands,
        &*asset_server,
        &mut meshes,
        Transform {
            translation: Vec3::new(-2.0, 3.0, -2.0),
            scale: Vec3::splat(2.5),
            ..default()
        },
    );

    crate::trees::spawn_trees(&mut commands, &*asset_server, &mut meshes);

    let _stone = commands
        .spawn_bundle(SceneBundle {
            scene: asset_server.load("models/rock1.glb#Scene0"),
            transform: Transform {
                translation: Vec3::new(-2.0, 5.0, 2.0),
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

    let _sky = commands
        .spawn_bundle(SceneBundle {
            scene: asset_server.load("models/sky.glb#Scene0"),
            transform: Transform {
                translation: Vec3::new(-1.5, 1.3, 1.075),
                ..default()
            },
            ..default()
        })
        .id();

    let sky_mesh: Handle<Mesh> = asset_server.load("models/sky_clouds.glb#Mesh0/Primitive0");

    let _sky_clouds = commands
        .spawn_bundle(SceneBundle {
            scene: asset_server.load("models/sky_clouds.glb#Scene0"),
            transform: Transform {
                translation: Vec3::new(-1.5, 1.3, 1.075),
                ..default()
            },
            ..default()
        })
        .insert(SkyLoad)
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

    let level_collision_mesh3: Handle<Mesh> =
        asset_server.load("models/cauldron_stirrer.glb#Mesh0/Primitive0");

    let mock = commands
        .spawn()
        .insert_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere {
                radius: 0.05,
                ..default()
            })),
            ..default()
        })
        .insert_bundle(TransformBundle::from_transform(Transform::from_xyz(
            0.0, 2.0, -3.0,
        )))
        .insert(Name::new("Mock spring location"))
        .id();

    let stirrer = commands
        .spawn_bundle(SceneBundle {
            scene: asset_server.load("models/cauldron_stirrer.glb#Scene0"),
            transform: Transform {
                // translation: Vec3::new(5., 10., -0.075),
                translation: Vec3::new(0.0, 2.0, -3.0),
                ..default()
            },
            ..default()
        })
        .insert_bundle(Attach::translation(mock))
        .insert(AttachTranslation::Spring {
            strength: 500.0,
            damp_ratio: 0.1,
        })
        .insert_bundle((
            //Collider::cuboid(0.1, 0.2, 0.1),
            //GravityScale(0.0),
            Damping {
                linear_damping: 0.2,
                angular_damping: 0.5,
            },
            //RigidBody::KinematicVelocityBased,
            RigidBody::Dynamic,
            Name::new("Paddle"),
            ExternalImpulse::default(),
            ExternalForce::default(),
            ReadMassProperties::default(),
            Velocity::default(),
            DEFAULT_FRICTION,
        ))
        .insert(ColliderLoad)
        .insert(level_collision_mesh3)
        .id();

    let level_collision_mesh: Handle<Mesh> =
        asset_server.load("models/walls_shop1.glb#Mesh0/Primitive0");

    let scale = Vec3::splat(3.0);
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
        .insert(DecompLoad(
            "assets/models/walls_shop1_decomp.obj".to_owned(),
        ))
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
        .insert_bundle(Attach::all(walls))
        .insert(Name::new("Shop Followers"))
        .add_child(security_check)
        .id();

    let mut hinge_joint = RevoluteJointBuilder::new(Vec3::Y)
        .local_anchor1(Vec3::new(0.85, 0.02, 0.15) * scale)
        .local_anchor2(Vec3::new(0.7, 0.0, 0.15) * scale)
        //.limits([-PI / 2.0 - PI / 8.0, PI / 2.0 + PI / 8.0])
        //.limits([-PI / 2.0 - PI / 8.0, 0.0])
        .limits([0.0, PI / 2.0 + PI / 8.0])
        .build();

    hinge_joint.set_contacts_enabled(false);

    let level_collision_mesh2: Handle<Mesh> = asset_server.load("models/door.glb#Mesh0/Primitive0");

    let _door = commands
        .spawn_bundle(SceneBundle {
            scene: asset_server.load("models/door.glb#Scene0"),
            transform: Transform {
                scale: scale * 0.9,
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
        .insert(BreakableJoint {
            impulse: Vec3::splat(100.0),
            torque: Vec3::splat(100.0),
        })
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

#[derive(Debug, Component, Clone)]
pub struct DecompLoad(String);

fn decomp_load(mut commands: Commands, mut replace: Query<(&mut Collider, &DecompLoad, Entity)>) {
    for (mut collider, decomp, entity) in &mut replace {
        info!("running decomp load");
        let decomp = Obj::load(&decomp.0).unwrap();
        let mut colliders = Vec::new();
        for object in decomp.data.objects {
            let vertices = object
                .groups
                .iter()
                .map(|group| {
                    group
                        .polys
                        .iter()
                        .map(|poly| poly.0.iter().map(|index| index.0))
                })
                .flatten()
                .flatten()
                .map(|index| decomp.data.position[index])
                .map(|f| Vec3::from(f))
                .collect::<Vec<_>>();
            let collider = Collider::convex_hull(&vertices).unwrap();
            colliders.push((Vec3::ZERO, Quat::IDENTITY, collider));
        }

        *collider = Collider::compound(colliders);
        commands.entity(entity).remove::<DecompLoad>();
    }
}

#[derive(Debug, Component, Clone, Copy)]
pub struct ColliderLoad;

fn update_level_collision(
    mut commands: Commands,
    mut ev_asset: EventReader<AssetEvent<Mesh>>,
    mut assets: ResMut<Assets<Mesh>>,
    mut replace: Query<(Option<&mut Collider>, &Handle<Mesh>, Entity), With<ColliderLoad>>,
) {
    for ev in ev_asset.iter() {
        match ev {
            AssetEvent::Created { handle } => {
                if let Some(loaded_mesh) = assets.get_mut(handle) {
                    for (mut col, inner_handle, e) in replace.iter_mut() {
                        if *inner_handle == *handle {
                            let new_collider =
                                Collider::from_bevy_mesh(loaded_mesh, &COMPUTE_SHAPE_PARAMS)
                                    .unwrap();
                            match col {
                                Some(mut col) => {
                                    *col = new_collider;
                                }
                                None => {
                                    commands.entity(e).insert(new_collider);
                                }
                            }
                            commands.entity(e).remove::<ColliderLoad>();
                        }
                    }
                }
            }
            AssetEvent::Modified { handle: _ } => {}
            AssetEvent::Removed { handle: _ } => {}
        }
    }
}

#[derive(Debug, Component, Clone, Copy)]
pub struct SkyLoad;

/*
pub const COMPUTE_SHAPE_PARAMS: ComputedColliderShape = ComputedColliderShape::TriMesh;
*/
pub const COMPUTE_SHAPE_PARAMS: ComputedColliderShape =
    ComputedColliderShape::ConvexDecomposition(VHACDParameters {
        /// Maximum concavity.
        ///
        /// Default: 0.1 (in 2D), 0.01 (in 3D).
        /// Valid range `[0.0, 1.0]`.
        concavity: 0.01,
        /// Controls the bias toward clipping along symmetry planes.
        ///
        /// Default: 0.05.
        /// Valid Range: `[0.0, 1.0]`.
        alpha: 0.05,
        /// Controls the bias toward clipping along revolution planes.
        ///
        /// Default: 0.05.
        /// Valid Range: `[0.0, 1.0]`.
        beta: 0.05,
        /// Resolution used during the voxelization stage.
        ///
        /// Default: 256 (in 2D), 64 (in 3D).
        resolution: 64,
        /// Controls the granularity of the search for the best
        /// clipping plane during the decomposition.
        ///
        /// Default: 4
        plane_downsampling: 4,
        /// Controls the precision of the convex-hull generation
        /// process during the clipping plane selection stage.
        ///
        /// Default: 4
        convex_hull_downsampling: 4,
        /// Controls the way the input mesh or polyline is being
        /// voxelized.
        ///
        /// Default: `FillMode::FloodFill { detect_cavities: false, detect_self_intersections: false }`
        //fill_mode: FillMode::SurfaceOnly,
        fill_mode: FillMode::FloodFill {
            detect_cavities: false,
        },
        /// Controls whether the convex-hull should be approximated during the decomposition stage.
        /// Setting this to `true` increases performances with a slight degradation of the decomposition
        /// quality.
        ///
        /// Default: true
        convex_hull_approximation: true,
        /// Controls the max number of convex-hull generated by the convex decomposition.
        ///
        /// Default: 1024
        max_convex_hulls: 1024,
    });
