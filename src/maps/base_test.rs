use std::f32::consts::PI;

use crate::{
    attach::Attach,
    objects::{
        cauldron::Ingredient,
        store::{SecurityCheck, StoreItem},
    },
    physics::slot::{Slot, SlotGracePeriod, SlotSettings, Slottable},
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
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube::new(1.0))),
            transform: Transform {
                translation: Vec3::new(0.0, -3.5, 0.0),
                scale: Vec3::new(100.0, 2.0, 100.0),
                ..default()
            },
            ..default()
        })
        .insert(Name::new("Ground"))
        .insert((
            RigidBody::Fixed,
            Collider::cuboid(0.5, 0.5, 0.5),
            crate::physics::TERRAIN_GROUPING,
            crate::DEFAULT_FRICTION,
        ));

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            rotation: Quat::from_rotation_x(-0.2),
            ..default()
        },
        ..default()
    });

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 5000.0,
            range: 50.0,
            ..default()
        },
        ..default()
    });
}
