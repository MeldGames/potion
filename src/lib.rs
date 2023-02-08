pub mod attach;
//pub mod brew;
pub mod cauldron;
pub mod debug;
pub mod deposit;
pub mod diagnostics;
pub mod egui;
pub mod joint_break;
pub mod network;
pub mod physics;
pub mod player;
pub mod slot;
pub mod store;
pub mod trees;

use std::f32::consts::PI;

use bevy_mod_inverse_kinematics::InverseKinematicsPlugin;
use bevy_rapier3d::prelude::*;
use cauldron::{CauldronPlugin, Ingredient};
use deposit::DepositPlugin;
use joint_break::BreakJointPlugin;
use obj::Obj;
use slot::{Slot, SlotGracePeriod, SlotPlugin, SlotSettings, Slottable};
use trees::TreesPlugin;

use bevy_mod_outline::*;

pub use debug::DebugVisible;

use attach::Attach;

use store::{SecurityCheck, StoreItem, StorePlugin};

//use crate::network::NetworkPlugin;
use crate::player::PlayerPlugin;

use bevy::{
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
    window::{CursorGrabMode, WindowPlugin}, scene::SceneInstance,
};

use bevy_prototype_debug_lines::*;

pub const DEFAULT_FRICTION: Friction = Friction::coefficient(0.5);
pub const TICK_RATE: std::time::Duration = sabi::prelude::tick_hz(60);

pub fn setup_app(app: &mut App) {
    //app.insert_resource(bevy::ecs::schedule::ReportExecutionOrderAmbiguities);
    let default_res = (1728.0, 1117.0);
    //let default_res = (800.0, 500.0);
    //let default_res = (1920.0, 1080.0);
    let half_width = ((default_res.0 / 2.0), default_res.1);
    let (title, (width, height), position) = match (
        app.world.contains_resource::<sabi::Local>(),
        app.world.contains_resource::<sabi::Client>(),
        app.world.contains_resource::<sabi::Server>(),
    ) {
        (true, _, _) => (
            "Potion Cellar Local".to_owned(),
            default_res,
            WindowPosition::Automatic,
        ),
        (_, true, _) => (
            "Potion Cellar Client".to_owned(),
            half_width,
            WindowPosition::At(Vec2::new(half_width.0, 0.0)),
        ),
        (_, _, true) => (
            "Potion Cellar Server".to_owned(),
            half_width,
            WindowPosition::At(Vec2::new(0.0, 0.0)),
        ),
        _ => {
            panic!("unknown program")
        }
    };

    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                window: WindowDescriptor {
                    title: title,
                    width: width,
                    height: height,
                    cursor_visible: true,
                    position: position,
                    cursor_grab_mode: CursorGrabMode::None,
                    present_mode: bevy::window::PresentMode::Immediate,
                    ..default()
                },
                ..default()
            })
            .set(AssetPlugin {
                watch_for_changes: true,
                ..default()
            }),
    );

    app.insert_resource(bevy_framepace::FramepaceSettings {
        limiter: bevy_framepace::Limiter::Off,
        //limiter: bevy_framepace::Limiter::Manual(sabi::prelude::tick_hz(60)),
    });
    app.insert_resource(bevy::pbr::DirectionalLightShadowMap { size: 2 << 10 });
    app.add_plugin(DebugLinesPlugin::default());
    app.add_plugin(crate::egui::SetupEguiPlugin);
    app.add_plugin(bevy_editor_pls::EditorPlugin);
    app.add_plugin(crate::network::NetworkPlugin);

    //app.add_plugin(bevy_framepace::FramepacePlugin);
    app.insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.3)))
        .add_plugin(PlayerPlugin)
        .add_plugin(CauldronPlugin)
        .add_plugin(SlotPlugin)
        .add_plugin(StorePlugin)
        .add_plugin(DepositPlugin)
        .add_plugin(BreakJointPlugin)
        .add_plugin(InverseKinematicsPlugin)
        .add_plugin(crate::debug::DebugPlugin)
        .add_plugin(TreesPlugin)
        .add_plugin(crate::physics::PhysicsPlugin)
        .add_plugin(crate::physics::MusclePlugin)
        .add_plugin(RapierDebugRenderPlugin {
            always_on_top: false,
            enabled: true,
            style: Default::default(),
            mode: DebugRenderMode::COLLIDER_SHAPES,
        })
        //.add_plugin(bevy::diagnostic::DiagnosticsPlugin)
        .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        .add_plugin(crate::diagnostics::DiagnosticsEguiPlugin);
    app.add_plugin(OutlinePlugin);
    //.add_plugin(AutoGenerateOutlineNormalsPlugin);
    app.add_system(outline_meshes);

    //app.add_system(bevy_mod_picking::debug::debug_draw_egui);

    app.add_event::<AssetEvent<Mesh>>();

    app.add_startup_system(fallback_camera);

    app.add_system(update_level_collision);
    app.add_system(active_cameras);
    app.add_system(decomp_load);
    //app.add_system(prepare_scene);

    app.add_plugin(crate::player::CustomWanderlustPlugin);
}

#[derive(Component)]
pub struct NoOutline;

fn outline_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<(Entity, &Handle<Mesh>), (Added<Handle<Mesh>>, Without<OutlineVolume>, Without<NotShadowReceiver>)>,
) {
    for (entity, mesh) in &query {
        if let Some(mesh) = meshes.get_mut(mesh) {
            if mesh.contains_attribute(Mesh::ATTRIBUTE_NORMAL) {
                let _ = mesh.generate_outline_normals();

                commands
                    .entity(entity)
                    .insert(SetOutlineDepth::Real)
                    .insert(OutlineBundle {
                        outline: OutlineVolume {
                            visible: true,
                            width: 5.0,
                            colour: Color::rgba(0.0, 0.0, 0.0, 1.0),
                        },
                        ..default()
                    });
            }
        }
    }
}

fn fallback_camera(mut commands: Commands) {
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(0., 12., 10.))
                .looking_at(Vec3::new(0.0, 0.3, 0.0), Vec3::Y),
            camera: Camera {
                priority: -50,
                is_active: false,
                ..default()
            },
            ..default()
        })
        .insert(Name::new("Fallback camera"));
}

pub fn setup_map(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    _materials: ResMut<Assets<StandardMaterial>>,
    _assets: Res<AssetServer>,
) {
    commands
        .spawn((
            SceneBundle {
                scene: asset_server.load("models/map.gltf#Scene0"),
                ..default()
            },
            NotShadowCaster,
            NotShadowReceiver,
            Name::new("Ground")
        )).add_children(|children| {
            children
                .spawn(TransformBundle::from_transform(Transform::from_xyz(
                    0.0, -10.0, 0.0,
                )))
                .insert((
                    RigidBody::Fixed,
                    Collider::cuboid(50.0, 10.0, 50.0),
                    Name::new("Plane"),
                    crate::physics::TERRAIN_GROUPING,
                    DEFAULT_FRICTION,
                    NotShadowReceiver,
                ));
        });

    commands.insert_resource(AmbientLight {
        color: Color::ALICE_BLUE,
        brightness: 0.72,
    });

    const HALF_SIZE: f32 = 100.0;
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            // Configure the projection to better fit the scene
            shadow_projection: OrthographicProjection {
                left: -HALF_SIZE,
                right: HALF_SIZE,
                bottom: -HALF_SIZE,
                top: HALF_SIZE,
                near: -10.0,
                far: 1000.0,
                ..default()
            },
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

    commands
        .spawn(TransformBundle::from_transform(Transform::from_xyz(
            5.0, 2.0, 5.0,
        )))
        .insert((
            RigidBody::KinematicPositionBased,
            Collider::capsule(Vec3::ZERO, Vec3::Y, 0.5),
            Name::new("Test capsule"),
            crate::physics::TERRAIN_GROUPING,
        ));

    let _cauldron = crate::cauldron::spawn_cauldron(
        &mut commands,
        &*asset_server,
        Transform {
            translation: Vec3::new(5.0, 2.0, 0.0),
            scale: Vec3::splat(2.),
            ..default()
        },
        &mut meshes,
    );

    crate::deposit::spawn_deposit_box(
        &mut commands,
        &*asset_server,
        &mut meshes,
        Transform {
            translation: Vec3::new(-4.0, 10.0, -2.0),
            scale: Vec3::splat(2.5),
            ..default()
        },
    );

    crate::trees::spawn_trees(&mut commands, &*asset_server, &mut meshes);

    let _stone = commands
        .spawn(SceneBundle {
            scene: asset_server.load("models/rock1.glb#Scene0"),
            transform: Transform {
                translation: Vec3::new(-2.0, 5.0, 2.0),
                ..default()
            },
            ..default()
        })
        .insert((
            Ingredient,
            crate::deposit::Value::new(1),
            Collider::cuboid(0.3, 0.3, 0.3),
            RigidBody::Dynamic,
            StoreItem,
            Slottable,
            ReadMassProperties::default(),
            ExternalImpulse::default(),
            Name::new("Stone"),
            Velocity::default(),
            DEFAULT_FRICTION,
        ))
        .id();

    let _cellar = commands.spawn((
        SceneBundle{
            scene: asset_server.load("models/cellar.gltf#Scene0"),
            transform: Transform {
                translation: Vec3::new(-16.5, -3.0, 1.075),
                ..default()
            },
            ..default()
        },
        SpawnedScene
    )).id();

    let _cart = commands.spawn((
        SpatialBundle{
            transform: Transform {
                translation: Vec3::new(-10.5, 7.3, -10.),
                rotation: Quat::from_axis_angle(Vec3::Z, PI/2.),
                ..default()
            },
            ..default()
        },

        Collider::cylinder(1.8, 1.3),
        RigidBody::Dynamic,
        Name::new("cart collider"),
        ColliderMassProperties::Density(2.0),
        crate::physics::TERRAIN_GROUPING,
        DEFAULT_FRICTION,
    )).add_children(|commands| {
        commands.spawn(SceneBundle{            
            scene: asset_server.load("models/cart.gltf#Scene0"),
            transform: Transform {
                rotation: Quat::from_axis_angle(Vec3::Z, -PI/2.),
                scale: Vec3::splat(2.),
                ..default()
            },
            ..default()
        });
        commands.spawn((
            SpatialBundle{
                transform: Transform {
                    translation: Vec3::new(-0.1, 0., -0.5),
                    ..default()
                },
                ..default()
            },            
            Collider::cuboid(0.1, 1.2, 2.9),
        ));
    });
    

    let _sky = commands
        .spawn(SceneBundle {
            scene: asset_server.load("models/skybox.gltf#Scene0"),
            transform: Transform {
                translation: Vec3::new(-1.5, 1.3, 1.075),
                ..default()
            },
            ..default()
        })
        .insert((
            NotShadowCaster,
            NotShadowReceiver,
        )).id();

    /* 
    let _sky_clouds = commands
        .spawn(SceneBundle {
            scene: asset_server.load("models/sky_clouds.glb#Scene0"),
            transform: Transform {
                translation: Vec3::new(-1.5, 1.3, 1.075),
                scale: Vec3::splat(2.0),
                ..default()
            },
            ..default()
        })
        .insert((
            NotShadowCaster,
            NotShadowReceiver,
        )).id();
    */
    let _donut = commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Torus {
                radius: 0.4,
                ring_radius: 0.2,
                ..default()
            })),
            transform: Transform::from_xyz(1.0, 6.0, -2.0),
            ..default()
        })
        .insert((
            Ingredient,
            crate::deposit::Value::new(5),
            Collider::round_cylinder(0.025, 0.4, 0.2),
            RigidBody::Dynamic,
            Name::new("Donut"),
            Velocity::default(),
            ExternalImpulse::default(),
            Slottable,
            ReadMassProperties::default(),
            DEFAULT_FRICTION,
        ))
        .id();

    

    let _prallet = commands
        .spawn(SceneBundle {
            scene: asset_server.load("models/prallet.glb#Scene0"),
            transform: Transform {
                translation: Vec3::new(-12.5, 3.3, -0.075),
                scale: Vec3::splat(1.),
                ..default()
            },
            ..default()
        })
        .insert((
            Ingredient,
            crate::deposit::Value::new(1),
            Collider::cuboid(0.3, 0.3, 0.3),
            RigidBody::Dynamic,
            Name::new("Prallet"),
            Velocity::default(),
            ExternalImpulse::default(),
            Slottable,
            ReadMassProperties::default(),
            DEFAULT_FRICTION,
        ))
        .id();

    let _thorns = commands
        .spawn(SceneBundle {
            scene: asset_server.load("models/thorns.glb#Scene0"),
            transform: Transform {
                translation: Vec3::new(-1.5, 8.3, -0.075),
                scale: Vec3::splat(1.),
                ..default()
            },
            ..default()
        })
        .insert((
            Ingredient,
            crate::deposit::Value::new(1),
            Collider::cuboid(0.3, 0.3, 0.3),
            RigidBody::Dynamic,
            StoreItem,
            Slottable,
            ReadMassProperties::default(),
            ExternalImpulse::default(),
            Name::new("Thorns"),
            Velocity::default(),
            DEFAULT_FRICTION,
        ))
        .id();

    let welt = commands
        .spawn(SceneBundle {
            scene: asset_server.load("models/weltberry.glb#Scene0"),
            transform: Transform {
                translation: Vec3::new(-2.5, 2.3, -0.075),
                ..default()
            },
            ..default()
        })
        .insert((
            Ingredient,
            Slottable,
            crate::deposit::Value::new(1),
            Collider::ball(0.2),
            RigidBody::Dynamic,
            Name::new("Weltberry"),
            Velocity::default(),
            ExternalImpulse::default(),
            ExternalForce::default(),
            //ColliderMassProperties::Density(50.0),
            ReadMassProperties::default(),
            DEFAULT_FRICTION,
        ))
        .id();

    let _welt_slot = commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere {
                radius: 0.05,
                ..default()
            })),
            ..default()
        })
        .insert((
            TransformBundle::from_transform(Transform {
                translation: Vec3::new(-2.5, 2.3, -0.075),
                ..default()
            }),
            Velocity::default(),
            Name::new("Welt slot"),
            ReadMassProperties::default(),
            Damping {
                linear_damping: 5.0,
                angular_damping: 5.0,
            },
            Slot {
                containing: Some(welt),
            },
            SlotGracePeriod::default(),
            SlotSettings(springy::SpringState {
                spring: springy::Spring {
                    strength: 1.0,
                    damp_ratio: 1.0,
                    rest_distance: 0.0,
                    limp_distance: 0.0,
                },
                breaking: Some(springy::SpringBreak {
                    tear_force: 4.0,
                    tear_step: 0.02,
                    heal_step: 0.05,
                    ..default()
                }),
                ..default()
            })
        ));

    let level_collision_mesh3: Handle<Mesh> =
        asset_server.load("models/cauldron_stirrer.glb#Mesh0/Primitive0");

    let _mock = commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere {
                radius: 0.00,
                ..default()
            })),
            ..default()
        })
        .insert(TransformBundle::from_transform(Transform::from_xyz(
            0.0, 2.0, -3.0,
        )))
        .insert(Name::new("Mock spring location"))
        .id();


    let col_mesh_mortar: Handle<Mesh> =
    asset_server.load("models/mortar.gltf#Mesh0/Primitive0");

    let _mortar = commands.spawn((
        SceneBundle {
            scene: asset_server.load("models/mortar.gltf#Scene0"),
            transform: Transform {
                // translation: Vec3::new(5., 10., -0.075),
                translation: Vec3::new(20.0, 5.0, -3.0),
                scale: Vec3::splat(2.),
                ..default()
            },
            ..default()
        },
        ColliderLoad,
        Name::new("Mortar & Pestle"),
        col_mesh_mortar,
        RigidBody::Dynamic,
        crate::physics::TERRAIN_GROUPING,
    )).id();
    
    let _stirrer = commands
        .spawn(SceneBundle {
            scene: asset_server.load("models/cauldron_stirrer.glb#Scene0"),
            transform: Transform {
                // translation: Vec3::new(5., 10., -0.075),
                translation: Vec3::new(0.0, 10.0, -3.0),
                scale: Vec3::splat(2.5),
                ..default()
            },
            ..default()
        })
        .insert(player::grab::AutoAim::Line {
            start: Vec3::new(0.0, 0.3, 0.0),
            end: Vec3::new(0.0, 1.2, 0.0),
        })
        .insert((
            //Collider::cuboid(0.1, 0.2, 0.1),
            //GravityScale(0.0),
            Damping {
                linear_damping: 0.5,
                angular_damping: 0.5,
            },
            //RigidBody::KinematicVelocityBased,
            RigidBody::Dynamic,
            Name::new("Stirrer"),
            ExternalImpulse::default(),
            ExternalForce::default(),
            ReadMassProperties::default(),
            Velocity::default(),
            DEFAULT_FRICTION,
            DecompLoad("assets/models/stirrer_decomp.obj".to_owned()),
            level_collision_mesh3,
        ))
        .id();

    let level_collision_mesh: Handle<Mesh> =
        asset_server.load("models/walls_shop1.glb#Mesh0/Primitive0");

    let scale = Vec3::splat(3.0);
    let walls = commands
        .spawn(SceneBundle {
            scene: asset_server.load("models/walls_shop1.glb#Scene0"),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, -10.0),
                scale: scale,
                ..default()
            },
            ..default()
        })
        .insert((
            Collider::cuboid(1.0, 1.0, 1.0),
            RigidBody::Fixed,
            Name::new("Walls Shop"),
            Velocity::default(),
            DecompLoad(
                "assets/models/walls_shop1_decomp.obj".to_owned(),
            ),
            level_collision_mesh,
        ))
        .id();

    let security_check = commands
        .spawn(TransformBundle::from_transform(Transform::from_xyz(
            1.1, 1.0, 0.5,
        )))
        .insert((
            Collider::cuboid(0.5, 1.0, 0.5),
            RigidBody::Fixed,
            Sensor,
            SecurityCheck { push: -Vec3::Z },
            Name::new("Security Check"),
        ))
        .id();

    let _shop_follower = commands
        .spawn(TransformBundle::default())
        .insert(Attach::all(walls))
        .insert(Name::new("Shop Followers"))
        .add_child(security_check)
        .id();

    let mut hinge_joint = RevoluteJointBuilder::new(Vec3::Y)
        .local_anchor1(Vec3::new(0.75, 0.02, 0.15) * scale)
        .local_anchor2(Vec3::new(0.75, 0.0, 0.15) * scale)
        //.limits([-PI / 2.0 - PI / 8.0, PI / 2.0 + PI / 8.0])
        //.limits([-PI / 2.0 - PI / 8.0, 0.0])
        .limits([0.0, PI / 2.0 + PI / 8.0])
        .build();

    hinge_joint.set_contacts_enabled(false);

    let level_collision_mesh2: Handle<Mesh> = asset_server.load("models/door.glb#Mesh0/Primitive0");

    let _door = commands
        .spawn(SceneBundle {
            scene: asset_server.load("models/door.glb#Scene0"),
            transform: Transform {
                scale: scale,
                ..default()
            },
            ..default()
        })
        .insert((
            Collider::cuboid(1.0, 1.0, 1.0),
            RigidBody::Dynamic,
            ColliderMassProperties::Density(10.0),
            Name::new("Door"),
            Velocity::default(),
            DEFAULT_FRICTION,
            ImpulseJoint::new(walls, hinge_joint),
            ColliderLoad,
            level_collision_mesh2
        ))
        /*
               .insert(BreakableJoint {
                   impulse: Vec3::splat(100.0),
                   torque: Vec3::splat(100.0),
               })
        */
        .id();

    // Bounds
    commands
        .spawn(TransformBundle::from_transform(Transform::from_xyz(
            0.0, 10.0, 50.0,
        )))
        .insert((
            RigidBody::Fixed,
            Collider::cuboid(50.0, 20.0, 1.0),
            Name::new("Bound Wall"),
            crate::physics::TERRAIN_GROUPING,
        ));
    commands
        .spawn(TransformBundle::from_transform(Transform::from_xyz(
            0.0, 10.0, -50.0,
        )))
        .insert((
            RigidBody::Fixed,
            Collider::cuboid(50.0, 20.0, 1.0),
            Name::new("Bound Wall"),
            crate::physics::TERRAIN_GROUPING,
        ));

    commands
        .spawn(TransformBundle::from_transform(Transform::from_xyz(
            50.0, 10.0, 0.0,
        )))
        .insert((
            RigidBody::Fixed,
            Collider::cuboid(1.0, 20.0, 50.0),
            Name::new("Bound Wall"),
            crate::physics::TERRAIN_GROUPING,
        ));
    commands
        .spawn(TransformBundle::from_transform(Transform::from_xyz(
            -50.0, 10.0, 0.0,
        )))
        .insert((
            RigidBody::Fixed,
            Collider::cuboid(1.0, 20.0, 50.0),
            Name::new("Bound Wall"),
            crate::physics::TERRAIN_GROUPING,
        ));
}

pub fn active_cameras(_names: Query<&Name>, cameras: Query<(Entity, &Camera)>) {
    let mut active = 0;
    for (_entity, camera) in &cameras {
        if camera.is_active {
            active += 1;
        }
    }

    if active > 1 {
        warn!("More than 1 active camera");
    }
}

#[derive(Component)]
pub struct SpawnedScene;

fn prepare_scene(
    mut commands: Commands,
    mut ev_asset: EventReader<AssetEvent<Scene>>,
    scene_root_nodes: Query<&Children>,
    objects: Query<(Entity, &Name)>,
    scenes: Query<&Children, With<SceneInstance>>,
) {
    for _event in ev_asset.iter() {
        for scene_root in scenes.iter() {
            info!("finished loading scene");
            for &root_node in scene_root.iter() {
                dbg!(root_node);
                for &scene_objects in scene_root_nodes.get(root_node).unwrap() {
                    if let Ok((e, name)) = objects.get(scene_objects) {
                        if name.contains("Light") {
                            let point_light = commands
                                .spawn(PointLightBundle {
                                    point_light: PointLight {
                                        range: 2000.,
                                        intensity: 800.0,
                                        color: Color::rgb(0.9, 0.4, 0.1),
                                        shadows_enabled: true,
                                        ..default()
                                    },
                                    ..default()
                                })
                                .id();
                            commands.entity(e).add_child(point_light);
                        }
                    }
                }
            }
        }
    }
}


#[derive(Debug, Component, Clone, Reflect)]
#[reflect(Component)]
pub struct DecompLoad(String);

impl Default for DecompLoad {
    fn default() -> Self {
        Self("".to_owned())
    }
}

fn decomp_load(
    mut commands: Commands,
    mut replace: Query<(Option<&mut Collider>, &DecompLoad, Entity)>,
) {
    for (collider, decomp, entity) in &mut replace {
        info!("running decomp load: {:?}", decomp);
        if let Ok(decomp) = Obj::load(&decomp.0) {
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

            let new_collider = Collider::compound(colliders);
            match collider {
                Some(mut collider) => {
                    *collider = new_collider;
                }
                None => {
                    commands.entity(entity).insert(new_collider);
                }
            }
        }

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
                    for (col, inner_handle, e) in replace.iter_mut() {
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
