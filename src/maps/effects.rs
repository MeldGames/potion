use crate::objects::vine::VineEffect;
use crate::prelude::*;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use std::f32::consts::PI;

type Effect = VineEffect;

pub struct SetupPlugin;
impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(super::base_test::SetupPlugin);
        app.add_systems(Startup, (ramps, flat));
        app.add_systems(Last, frame_delay_effect);
    }
}

#[derive(Component, Default)]
pub struct DelayedEffect;

fn frame_delay_effect(
    time: Res<Time>,
    mut wait: Local<f32>,
    mut commands: Commands,
    delayed: Query<Entity, With<DelayedEffect>>,
) {
    *wait += time.delta_seconds();
    if *wait <= 1.0 {
        return;
    }

    for entity in &delayed {
        commands
            .entity(entity)
            .insert(Effect::default())
            .remove::<DelayedEffect>();
    }
}

fn flat(mut commands: Commands) {
    let point = Vec3::new(-8.0, 1.0, 2.0);
    commands.spawn((
        SpatialBundle {
            transform: Transform {
                translation: point,
                scale: Vec3::new(3.0, 1.0, 3.0),
                ..default()
            },
            ..default()
        },
        RigidBodyBundle {
            rigid_body: RigidBody::Fixed,
            ..default()
        },
        ColliderBundle {
            collider: Collider::cuboid(0.5, 0.5, 0.5),
            collision_groups: crate::physics::TERRAIN_GROUPING,
            ..default()
        },
    ));

    commands.spawn((
        //Effect::default(),
        DelayedEffect,
        SpatialBundle {
            transform: Transform {
                translation: point + Vec3::Y,
                ..default()
            },
            ..default()
        },
    ));
}

fn effect(commands: &mut Commands, translation: Vec3) {
    commands.spawn((
        DelayedEffect,
        SpatialBundle {
            transform: Transform {
                translation: translation + Vec3::Y,
                ..default()
            },
            ..default()
        },
    ));
}

fn ramps(mut commands: Commands) {
    let center = Vec3::new(-10.0, 1.0, 10.0);
    effect(&mut commands, center);

    let radius = 3.0;
    let steps = 8;
    for step in 1..=steps {
        let per = step as f32 / steps as f32;
        let radians = per * PI * 2.0;

        ramp(
            &mut commands,
            center + Vec3::new(radians.sin(), 0.0, radians.cos()) * radius,
            Quat::from_rotation_y(radians) * Quat::from_rotation_x(45f32.to_radians()),
        //Quat::IDENTITY,
        );
    }
}

fn ramp(commands: &mut Commands, translation: Vec3, rotation: Quat) {
    commands.spawn((
        SpatialBundle {
            transform: Transform {
                translation,
                rotation,
                scale: Vec3::new(1.0, 5.0, 1.0),
            },
            ..default()
        },
        RigidBodyBundle {
            rigid_body: RigidBody::Fixed,
            ..default()
        },
        ColliderBundle {
            collider: Collider::cuboid(0.5, 0.5, 0.5),
            collision_groups: crate::physics::TERRAIN_GROUPING,
            ..default()
        },
    ));

    effect(commands, translation);
}
