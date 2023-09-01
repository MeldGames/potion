use crate::prelude::*;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use std::f32::consts::PI;

pub struct SetupPlugin;
impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(super::base_test::SetupPlugin);
        //app.add_systems(Startup, setup);
        app.add_systems(Startup, (simple, compound, children, no_rigidbody));
    }
}

pub fn simple(mut commands: Commands) {}

pub fn no_rigidbody(mut commands: Commands) {
    commands.spawn((
        SpatialBundle {
            transform: Transform {
                translation: Vec3::new(5.0, 1.0, 2.),
                ..default()
            },
            ..default()
        },
        ColliderBundle {
            collider: Collider::compound(vec![
                (
                    Vec3::new(0.5, 0.5, 0.5),
                    Quat::IDENTITY,
                    Collider::cylinder(0.5, 0.5),
                ),
                (
                    Vec3::new(-0.5, 0.5, 0.5),
                    Quat::IDENTITY,
                    Collider::cylinder(0.5, 0.5),
                ),
                (
                    Vec3::new(-0.5, 0.5, -0.5),
                    Quat::IDENTITY,
                    Collider::cylinder(0.5, 0.5),
                ),
                (
                    Vec3::new(0.5, 0.5, -0.5),
                    Quat::IDENTITY,
                    Collider::cylinder(0.5, 0.5),
                ),
            ]),
            ..default()
        },
    ));
}

pub fn compound(mut commands: Commands) {
    commands.spawn((
        SpatialBundle {
            transform: Transform {
                translation: Vec3::new(5.0, 1.0, 0.),
                ..default()
            },
            ..default()
        },
        RigidBodyBundle::fixed(),
        ColliderBundle {
            collider: Collider::compound(vec![
                (
                    Vec3::new(0.5, 0.5, 0.5),
                    Quat::IDENTITY,
                    Collider::cylinder(0.5, 0.5),
                ),
                (
                    Vec3::new(-0.5, 0.5, 0.5),
                    Quat::IDENTITY,
                    Collider::cylinder(0.5, 0.5),
                ),
                (
                    Vec3::new(-0.5, 0.5, -0.5),
                    Quat::IDENTITY,
                    Collider::cylinder(0.5, 0.5),
                ),
                (
                    Vec3::new(0.5, 0.5, -0.5),
                    Quat::IDENTITY,
                    Collider::cylinder(0.5, 0.5),
                ),
            ]),
            ..default()
        },
    ));
}

pub fn children(mut commands: Commands) {
    commands
        .spawn((
            SpatialBundle {
                transform: Transform {
                    translation: Vec3::new(-10.5, 1.0, -10.),
                    rotation: Quat::from_axis_angle(Vec3::Z, PI / 2.),
                    scale: Vec3::splat(1.0),
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

    commands
        .spawn((
            SpatialBundle {
                transform: Transform {
                    translation: Vec3::new(-14.5, 1.0, -10.),
                    rotation: Quat::from_axis_angle(Vec3::Z, PI / 2.),
                    scale: Vec3::splat(2.0),
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
}
