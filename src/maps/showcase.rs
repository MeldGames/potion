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

use crate::prelude::*;

use bevy::{
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
};

use bevy_mod_billboard::prelude::*;
use bevy_rapier3d::prelude::*;

pub struct SetupPlugin;
impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BillboardPlugin);
        app.add_systems(Startup, (setup, moving_ground));
        app.add_systems(FixedUpdate, circle_velocity);
        app.add_systems(Startup, ramps);
        app.add_systems(Startup, potions);
    }
}

pub fn moving_ground(mut commands: Commands) {
    commands
        .spawn(SpatialBundle {
            transform: Transform {
                translation: Vec3::new(-25.0, 1.0, 15.0),
                ..default()
            },
            ..default()
        })
        .insert(RigidBodyBundle {
            velocity: Velocity {
                linvel: Vec3::new(-0.3, 0.0, 0.0),
                angvel: Vec3::new(0.0, 0.3, 0.0),
                //angvel: Vec3::new(0.0, 0.0, 0.0),
            },
            ..RigidBodyBundle::kinematic_velocity()
        })
        .insert(CircleVelocity)
        .insert(ColliderBundle::collider(Collider::cuboid(3.0, 0.1, 3.0)));
}

pub fn ramps(mut commands: Commands) {
    let steps = 9;
    let angle_step = (std::f32::consts::PI / 2.0) / steps as f32;
    let full_width = 5.0;
    let width = full_width / steps as f32;
    for i in 1..=steps {
        commands
            .spawn(SpatialBundle {
                transform: Transform {
                    translation: Vec3::new(-30.0 + i as f32 * width * 2.0, -1.0, -10.0),
                    rotation: Quat::from_axis_angle(Vec3::X, angle_step * i as f32),
                    ..default()
                },
                ..default()
            })
            .insert(RigidBodyBundle {
                ..RigidBodyBundle::kinematic_velocity()
            })
            .insert(ColliderBundle::collider(Collider::cuboid(width, 1.0, 5.0)));
    }
}

#[derive(Component, Debug, Copy, Clone)]
pub struct CircleVelocity;

pub fn circle_velocity(mut t: Local<f32>, mut query: Query<&mut Velocity, With<CircleVelocity>>) {
    for mut vel in &mut query {
        vel.linvel = Vec3::new(t.sin(), 0.0, t.cos());
    }

    *t += 0.01;
}

pub fn potions(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let location = Vec3::new(-15.0, 2.0, 0.0);
    commands
        .spawn((
            SceneBundle {
                scene: asset_server.load("models/potion_square.glb#Scene0"),
                transform: Transform {
                    translation: location + Vec3::new(0.0, 0.0, 0.0),
                    scale: Vec3::new(0.5, 0.5, 0.5),
                    ..default()
                },
                ..default()
            },
            Name::new("potion"),
        ))
        .insert(Storeable)
        .insert(RigidBodyBundle::dynamic())
        .insert(crate::objects::potion::PotionBundle::default())
        .with_children(|children| {
            children
                .spawn(TransformBundle {
                    local: Transform {
                        //translation: Vec3::new(0., 0.5, 0.),
                        scale: Vec3::new(1.15, 2.0, 1.15),
                        ..default()
                    },
                    ..default()
                })
                .insert(crate::objects::potion::PotionColliderBundle::default())
                .insert(ColliderBundle {
                    collider: Collider::cuboid(0.5, 0.5, 0.5),
                    collision_groups: crate::physics::TERRAIN_GROUPING,
                    ..default()
                });
        });

    commands
        .spawn((
            SceneBundle {
                scene: asset_server.load("models/potion_coil.glb#Scene0"),
                transform: Transform {
                    translation: location + Vec3::new(1.0, 0.0, 0.0),
                    scale: Vec3::new(0.5, 0.5, 0.5),
                    ..default()
                },
                ..default()
            },
            Name::new("potion 3"),
        ))
        .insert(Storeable)
        .insert(RigidBodyBundle::dynamic())
        .insert(crate::objects::potion::PotionBundle::default())
        .insert(crate::objects::potion::PotionColliderBundle::default())
        .insert(ColliderBundle {
            collider: Collider::cuboid(0.5, 0.5, 0.5),
            collision_groups: crate::physics::TERRAIN_GROUPING,
            ..default()
        });

    commands
        .spawn((
            SceneBundle {
                scene: asset_server.load("models/potion_flask.glb#Scene0"),
                transform: Transform {
                    translation: location + Vec3::new(2.0, 0.0, 0.0),
                    scale: Vec3::new(0.5, 0.5, 0.5),
                    ..default()
                },
                ..default()
            },
            Name::new("potion 2"),
        ))
        .insert(Storeable)
        .insert(crate::objects::potion::PotionBundle::default())
        .insert(crate::objects::potion::PotionColliderBundle::default())
        .insert(RigidBodyBundle::dynamic())
        .insert(ColliderBundle {
            collider: Collider::cuboid(0.5, 0.5, 0.5),
            collision_groups: crate::physics::TERRAIN_GROUPING,
            ..default()
        });

}

pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands
        .spawn((
            SceneBundle {
                //scene: asset_server.load("models/map.gltf#Scene0"),
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
                    Collider::cuboid(30.0, 10.0, 30.0),
                    Name::new("Plane"),
                    crate::physics::TERRAIN_GROUPING,
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

    //crate::objects::trees::spawn_trees(&mut commands, &*asset_server, &mut meshes);
    let billboard = commands.spawn(BillboardTextBundle {
        transform: Transform {
            translation: Vec3::new(-5.0, 2.0, 5.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::splat(0.0085),
        },
        text: Text::from_sections([
            TextSection {
                value: "IMPORTANT".to_string(),
                style: TextStyle {
                    font_size: 60.0,
                    color: Color::ORANGE,
                    ..default()
                },
            },
            TextSection {
                value: " text".to_string(),
                style: TextStyle {
                    font_size: 60.0,
                    color: Color::WHITE,
                    ..default()
                },
            },
        ])
        .with_alignment(TextAlignment::Center),
        ..default()
    });

    let _stone = commands
        .spawn(SceneBundle {
            scene: asset_server.load("models/rock1.glb#Scene0"),
            transform: Transform {
                translation: Vec3::new(-2.0, 5.0, 2.0),
                ..default()
            },
            ..default()
        })
        .insert(RigidBodyBundle::dynamic())
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
        .insert(RigidBodyBundle::dynamic())
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
                    translation: Vec3::new(-20.6, -3.7, 1.0),
                    scale: Vec3::splat(1.5),
                    ..default()
                },
                ..default()
            },
            crate::SpawnedScene,
        ))
        .insert(Name::new("Cellar"))
        .insert(RigidBodyBundle::fixed())
        .insert(ColliderBundle {
            collider: Collider::compound(vec![
                // Floor
                (
                    Vec3::new(-11., -1., 0.),
                    Quat::IDENTITY,
                    Collider::cuboid(10., 1., 10.),
                ),
                // Walls
                (
                    Vec3::new(-11., -1., 25.4),
                    Quat::IDENTITY,
                    Collider::cuboid(20., 3.425, 20.),
                ),
                (
                    Vec3::new(-11., -1., -24.625),
                    Quat::IDENTITY,
                    Collider::cuboid(20., 3.425, 20.),
                ),
                (
                    Vec3::new(-36.26, -1., 0.),
                    Quat::IDENTITY,
                    Collider::cuboid(20., 3.425, 20.),
                ),
            ]),
            ..default()
        })
        .with_children(|children| {
            children
                .spawn(ColliderBundle {
                    collider: Collider::cuboid(0.5, 0.5, 0.5),
                    ..default()
                })
                .insert(TransformBundle::default());
        })
        .id();

    let _cart = commands
        .spawn((
            SpatialBundle {
                transform: Transform {
                    translation: Vec3::new(-10.5, 0.3, -10.),
                    rotation: Quat::from_axis_angle(Vec3::Z, PI / 2.),
                    scale: Vec3::splat(1.5),
                    ..default()
                },
                ..default()
            },
            Name::new("cart collider"),
        ))
        .insert(RigidBodyBundle {
            //rigid_body: RigidBody::Dynamic,
            rigid_body: RigidBody::Fixed,
            ..default()
        })
        .insert(ColliderBundle {
            collider: Collider::cylinder(1.8, 1.3),
            collision_groups: crate::physics::TERRAIN_GROUPING,
            mass_properties: ColliderMassProperties::Density(2.0),
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
        .insert(RigidBodyBundle::dynamic())
        .insert(ColliderBundle::collider(Collider::ball(ball_radius)))
        .insert((
            Ingredient,
            crate::deposit::Value::new(5),
            Name::new("Ball"),
            Slottable::default(),
        ))
        .id();

    let _donut = commands
        .spawn(PbrBundle {
            /*
                       mesh: meshes.add(Mesh::from(shape::Torus {
                           radius: 0.4,
                           ring_radius: 0.2,
                           ..default()
                       })),
            */
            transform: Transform::from_xyz(2.0, 6.0, -2.0),
            ..default()
        })
        .insert(RigidBodyBundle::dynamic())
        .insert(ColliderBundle::collider(Collider::round_cylinder(
            0.025, 0.4, 0.1,
        )))
        .insert((
            Ingredient,
            crate::deposit::Value::new(5),
            Name::new("Donut 2"),
            Slottable::default(),
        ))
        .id();

    let _donut = commands
        .spawn(PbrBundle {
            /*
                       mesh: meshes.add(Mesh::from(shape::Torus {
                           radius: 0.4,
                           ring_radius: 0.2,
                           ..default()
                       })),
            */
            transform: Transform::from_xyz(1.0, 6.0, -2.0),
            ..default()
        })
        .insert(RigidBodyBundle::dynamic())
        .insert(ColliderBundle::collider(Collider::cylinder(0.025, 0.4)))
        .insert((
            Ingredient,
            crate::deposit::Value::new(5),
            Name::new("Donut"),
            Slottable::default(),
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
        .insert(RigidBodyBundle::dynamic())
        .insert(ColliderBundle::collider(Collider::cuboid(0.3, 0.3, 0.3)))
        .insert((
            Ingredient,
            crate::deposit::Value::new(1),
            Name::new("Prallet"),
            Slottable::default(),
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
        .insert(RigidBodyBundle::dynamic())
        .insert(ColliderBundle::collider(Collider::cuboid(0.3, 0.3, 0.3)))
        .insert((
            Ingredient,
            crate::deposit::Value::new(1),
            StoreItem,
            Slottable::default(),
            Name::new("Thorns"),
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
        .insert(RigidBodyBundle::default())
        .insert(ColliderBundle {
            collider: Collider::ball(0.2),
            mass_properties: ColliderMassProperties::Density(1.0),
            ..default()
        })
        .insert((
            Ingredient,
            Slottable::default(),
            crate::deposit::Value::new(1),
            Name::new("Weltberry"),
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
        .insert(RigidBodyBundle {
            rigid_body: RigidBody::KinematicPositionBased,
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
        ))
        .insert(RigidBodyBundle::dynamic())
        .insert(ColliderBundle {
            collision_groups: TERRAIN_GROUPING,
            ..default()
        })
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
        .insert(RigidBodyBundle::dynamic())
        .insert((Name::new("Stirrer"), level_collision_mesh3))
        .with_children(|builder| {
            builder
                .spawn(TransformBundle {
                    local: Transform {
                        translation: Vec3::new(0.0, 1.0, 0.0),
                        ..default()
                    },
                    ..default()
                })
                //.insert(DecompLoad("stirrer".to_owned()))
                .insert(ColliderBundle::collider(Collider::cuboid(0.1, 0.5, 0.1)));
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
        .insert(RigidBodyBundle::fixed())
        .insert(ColliderBundle::collider(Collider::cuboid(1.0, 1.0, 1.0)))
        .insert((
            Name::new("Walls Shop"),
            crate::DecompLoad("walls_shop1".to_owned()),
            level_collision_mesh,
        ))
        .id();

    let security_check = commands
        .spawn(TransformBundle::from_transform(Transform::from_xyz(
            1.1, 1.0, 0.5,
        )))
        .insert(RigidBodyBundle::fixed())
        .insert(ColliderBundle::collider(Collider::cuboid(0.5, 1.0, 0.5)))
        .insert(Sensor)
        .insert((
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
            //scene: asset_server.load("models/door.glb#Scene0"),
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
            ..default()
        })
        .insert(ColliderBundle {
            mass_properties: ColliderMassProperties::Density(10.0),
            collider: Collider::cuboid(0.0, 0.0, 0.0),
            ..default()
        })
        .id();
}
