use bevy_mod_edge_detection::EdgeDetectionConfig;
use bevy_rapier3d::prelude::*;

use crate::{
    attach::Attach,
    cauldron::{CauldronPlugin, Ingredient},
    debug::DebugVisible,
    player::grab::{AimPrimitive, AutoAim},
    slot::{Slot, SlotGracePeriod, SlotPlugin, SlotSettings, Slottable},
    store::{SecurityCheck, StoreItem},
};

use obj::Obj;

use bevy::{
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
    scene::SceneInstance,
    window::{Cursor, CursorGrabMode},
};

use bevy_prototype_debug_lines::*;

use std::f32::consts::PI;

pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let test_texture = asset_server.load("models/materials/Placeholder.png");
    let test_material = materials.add(StandardMaterial {
        base_color_texture: Some(test_texture.clone()),
        perceptual_roughness: 0.95,
        reflectance: 0.05,
        ..default()
    });

    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube::new(1.0))),
            material: test_material.clone(),
            transform: Transform {
                translation: Vec3::new(0.0, -3.0, 0.0),
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
            // Configure the projection to better fit the scene
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            rotation: Quat::from_rotation_x(-0.2),
            ..default()
        },
        ..default()
    });
}
