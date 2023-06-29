use std::f32::consts::PI;

use crate::{
    attach::Attach,
    objects::{
        cauldron::Ingredient,
        store::{SecurityCheck, StoreItem},
    },
    physics::{
        slot::{Slot, SlotGracePeriod, SlotSettings, Slottable},
        ColliderBundle, RigidBodyBundle,
    },
    player::grab::{AimPrimitive, AutoAim},
};

use bevy::{
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
};

use bevy_rapier3d::prelude::*;

pub struct SetupPlugin;
impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup);
    }
}

pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn((
            SceneBundle {
                scene: asset_server.load("models/map.gltf#Scene0"),
                ..default()
            },
            NotShadowCaster,
            NotShadowReceiver,
            Name::new("Ground"),
        ))
        .with_children(|children| {
            children
                .spawn(TransformBundle::from_transform(Transform::from_xyz(
                    0.0, -10.0, 0.0,
                )))
                .insert((
                    RigidBody::Fixed,
                    Collider::cuboid(50.0, 10.0, 50.0),
                    Name::new("Plane"),
                    crate::physics::TERRAIN_GROUPING,
                    crate::DEFAULT_FRICTION,
                    NotShadowReceiver,
                ));
        });

    commands.insert_resource(AmbientLight {
        color: Color::ALICE_BLUE,
        brightness: 0.72,
    });

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

    let _cauldron = crate::objects::cauldron::spawn_cauldron(
        &mut commands,
        &*asset_server,
        Transform {
            translation: Vec3::new(-5.0, 2.0, 0.0),
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

    crate::objects::trees::spawn_trees(&mut commands, &*asset_server, &mut meshes);

    let _stone = commands
        .spawn(SceneBundle {
            scene: asset_server.load("models/rock1.glb#Scene0"),
            transform: Transform {
                translation: Vec3::new(-2.0, 5.0, 2.0),
                ..default()
            },
            ..default()
        })
        .insert(RigidBodyBundle {
            rigid_body: RigidBody::Dynamic,
            friction: crate::DEFAULT_FRICTION,
            ..default()
        })
        .insert(ColliderBundle {
            collider: Collider::cuboid(0.3, 0.3, 0.3),
            collision_groups: crate::physics::TERRAIN_GROUPING,
            ..default()
        })
        .insert((
            Ingredient,
            crate::deposit::Value::new(1),
            StoreItem,
            Slottable::default(),
            Name::new("Stone"),
        ))
        .id();

    let _stone = commands
        .spawn(SceneBundle {
            scene: asset_server.load("models/rock1.glb#Scene0"),
            transform: Transform {
                translation: Vec3::new(-2.0, 5.0, 2.0),
                ..default()
            },
            ..default()
        })
        .insert(RigidBodyBundle {
            rigid_body: RigidBody::Dynamic,
            friction: crate::DEFAULT_FRICTION,
            ..default()
        })
        .insert(ColliderBundle {
            collider: Collider::cuboid(0.5, 0.5, 0.5),
            collision_groups: crate::physics::TERRAIN_GROUPING,
            ..default()
        })
        .insert((
            Ingredient,
            crate::deposit::Value::new(1),
            StoreItem,
            Slottable::default(),
            Name::new("Stone"),
        ))
        .id();

    let _cellar = commands
        .spawn((
            SceneBundle {
                scene: asset_server.load("models/cellar.gltf#Scene0"),
                transform: Transform {
                    translation: Vec3::new(-16.5, -3.0, 1.075),
                    ..default()
                },
                ..default()
            },
            crate::SpawnedScene,
        ))
        .id();

    let _cart = commands
        .spawn((
            SpatialBundle {
                transform: Transform {
                    translation: Vec3::new(-10.5, 7.3, -10.),
                    rotation: Quat::from_axis_angle(Vec3::Z, PI / 2.),
                    ..default()
                },
                ..default()
            },
            Name::new("cart collider"),
        ))
        .insert(RigidBodyBundle {
            rigid_body: RigidBody::Dynamic,
            friction: crate::DEFAULT_FRICTION,
            ..default()
        })
        .insert(ColliderBundle {
            collider: Collider::cylinder(1.8, 1.3),
            collision_groups: crate::physics::TERRAIN_GROUPING,
            collider_mass_properties: ColliderMassProperties::Density(2.0),
            ..default()
        })
        .with_children(|commands| {
            commands.spawn(SceneBundle {
                scene: asset_server.load("models/cart.gltf#Scene0"),
                transform: Transform {
                    rotation: Quat::from_axis_angle(Vec3::Z, -PI / 2.),
                    scale: Vec3::splat(2.),
                    ..default()
                },
                ..default()
            });
            commands
                .spawn(SpatialBundle {
                    transform: Transform {
                        translation: Vec3::new(-0.1, 0., -0.5),
                        ..default()
                    },
                    ..default()
                })
                .insert(ColliderBundle {
                    collider: Collider::cuboid(0.1, 1.2, 2.9),
                    ..default()
                });
        });

    let _sky = commands
        .spawn(SceneBundle {
            scene: asset_server.load("models/skybox.gltf#Scene0"),
            transform: Transform {
                translation: Vec3::new(-1.5, 1.3, 1.075),
                scale: Vec3::splat(3.0),
                ..default()
            },
            ..default()
        })
        .insert((NotShadowCaster, NotShadowReceiver))
        .id();

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
           .insert((NotShadowCaster, NotShadowReceiver))
           .id();
    */

    let ball_radius = 0.6;
    let _ball = commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere {
                radius: ball_radius,
                ..default()
            })),
            transform: Transform::from_xyz(1.0, 8.0, -2.0),
            ..default()
        })
        .insert(AutoAim(vec![
            AimPrimitive::Point(Vec3::Z * ball_radius),
            AimPrimitive::Point(-Vec3::Z * ball_radius),
        ]))
        .insert((
            Ingredient,
            crate::deposit::Value::new(5),
            Collider::ball(ball_radius),
            RigidBody::Dynamic,
            Name::new("Ball"),
            Velocity::default(),
            ExternalImpulse::default(),
            Slottable::default(),
            ReadMassProperties::default(),
            crate::DEFAULT_FRICTION,
        ))
        .id();

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
            //Collider::cylinder(1.0, 1.0),
            RigidBody::Dynamic,
            Name::new("Donut"),
            Velocity::default(),
            ExternalImpulse::default(),
            Slottable::default(),
            ReadMassProperties::default(),
            Damping {
                linear_damping: 0.5,
                angular_damping: 0.5,
            },
            crate::DEFAULT_FRICTION,
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
            Slottable::default(),
            ReadMassProperties::default(),
            crate::DEFAULT_FRICTION,
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
            Slottable::default(),
            ReadMassProperties::default(),
            ExternalImpulse::default(),
            Name::new("Thorns"),
            Velocity::default(),
            crate::DEFAULT_FRICTION,
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
            Slottable::default(),
            crate::deposit::Value::new(1),
            Collider::ball(0.2),
            RigidBody::Dynamic,
            Name::new("Weltberry"),
            Velocity::default(),
            ExternalImpulse::default(),
            ExternalForce::default(),
            //ColliderMassProperties::Density(50.0),
            Damping {
                linear_damping: 0.5,
                angular_damping: 0.5,
            },
            ReadMassProperties::default(),
            crate::DEFAULT_FRICTION,
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
            SlotSettings(springy::Spring {
                strength: 1.0,
                damp_ratio: 1.0,
            }),
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

    let col_mesh_mortar: Handle<Mesh> = asset_server.load("models/mortar.gltf#Mesh0/Primitive0");

    let _mortar = commands
        .spawn((
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
            crate::ColliderLoad,
            Name::new("Mortar & Pestle"),
            col_mesh_mortar,
            RigidBody::Dynamic,
            crate::physics::TERRAIN_GROUPING,
        ))
        .id();

    let _stirrer = commands
        .spawn(SceneBundle {
            scene: asset_server.load("models/cauldron_stirrer.glb#Scene0"),
            transform: Transform {
                // translation: Vec3::new(5., 10., -0.075),
                translation: Vec3::new(-10.0, 10.0, -4.0),
                scale: Vec3::splat(1.5),
                ..default()
            },
            ..default()
        })
        .insert(AutoAim(vec![AimPrimitive::Line {
            start: Vec3::new(0.0, 0.3, 0.0),
            end: Vec3::new(0.0, 1.2, 0.0),
        }]))
        .insert((
            //GravityScale(0.0),
            ColliderMassProperties::Density(4.0),
            //RigidBody::KinematicVelocityBased,
            RigidBody::Dynamic,
            Name::new("Stirrer"),
            ExternalImpulse::default(),
            ExternalForce::default(),
            ReadMassProperties::default(),
            Velocity::default(),
            crate::DEFAULT_FRICTION,
            //DecompLoad("stirrer".to_owned()),
            level_collision_mesh3,
        ))
        .with_children(|builder| {
            builder
                .spawn(TransformBundle {
                    local: Transform {
                        translation: Vec3::new(0.0, 1.0, 0.0),
                        ..default()
                    },
                    ..default()
                })
                .insert(Collider::cuboid(0.1, 0.5, 0.1));
        })
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
            crate::DecompLoad("walls_shop1".to_owned()),
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
            Name::new("Door"),
            ImpulseJoint::new(walls, hinge_joint),
            crate::ColliderLoad,
            level_collision_mesh2,
        ))
        .insert(RigidBodyBundle {
            rigid_body: RigidBody::Dynamic,
            friction: crate::DEFAULT_FRICTION,
            ..default()
        })
        .insert(ColliderBundle {
            collider_mass_properties: ColliderMassProperties::Density(10.0),
            collider: Collider::cuboid(0.0, 0.0, 0.0),
            ..default()
        })
        .id();
}
