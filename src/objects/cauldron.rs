use bevy::prelude::*;

use bevy_rapier3d::prelude::*;

use crate::{
    attach::Attach,
    physics::{
        slot::{Slot, SlotDeposit},
        ColliderBundle, RigidBodyBundle,
    },
};

#[derive(Default, Debug, Copy, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct Cauldron;

#[derive(Default, Debug, Copy, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct Ingredient;

pub fn spawn_cauldron(
    commands: &mut Commands,
    asset_server: &AssetServer,
    position: Transform,
    meshes: &mut ResMut<Assets<Mesh>>,
) -> Entity {
    let mut slots = Vec::new();
    let cauldron = commands
        .spawn(SceneBundle {
            scene: asset_server.load("models/cauldron.gltf#Scene0"),
            transform: position,
            ..default()
        })
        .insert(ColliderBundle {
            collider: Collider::cylinder(0.5, 0.75),
            mass_properties: ColliderMassProperties::Density(5.0),
            collision_groups: crate::physics::TERRAIN_GROUPING,
            ..default()
        })
        .insert(RigidBodyBundle {
            rigid_body: RigidBody::Dynamic,
            ..default()
        })
        .insert(crate::player::inventory::Storeable)
        .insert((crate::objects::store::StoreItem, Name::new("Cauldron")))
        .insert(crate::DecompLoad("cauldron".to_owned()))
        .with_children(|builder| {
            let center = Vec3::new(0.0, 0.5, 0.0);
            let radius = 0.2;
            let slot_count = 3;

            let slice = std::f32::consts::TAU / slot_count as f32;
            for i in 0..slot_count {
                let x = (slice * i as f32).cos();
                let z = (slice * i as f32).sin();
                slots.push(
                    builder
                        .spawn(PbrBundle {
                            mesh: meshes.add(Mesh::from(shape::UVSphere {
                                radius: 0.02,
                                ..default()
                            })),
                            transform: Transform::from_translation(
                                center + Vec3::new(x, 2.5, z) * radius,
                            ),
                            ..default()
                        })
                        .insert(Name::new(format!("Cauldron slot {}", i)))
                        .insert(Velocity::default())
                        .insert(Slot::default())
                        .insert(crate::DebugVisible)
                        .insert(crate::physics::slot::SlotGracePeriod::default())
                        .insert(crate::physics::slot::SlotSettings(springy::Spring {
                            strength: 1.00,
                            damp_ratio: 0.2,
                        }))
                        .id(),
                );
            }
        })
        .id();

    commands
        .spawn(TransformBundle::from_transform(position))
        .insert(Attach::all(cauldron))
        .insert((
            Name::new("Cauldron Deposit"),
            crate::physics::TERRAIN_GROUPING,
        ))
        .with_children(|children| {
            children
                .spawn(TransformBundle::from_transform(Transform::from_xyz(
                    0.0, 0.6, 0.0,
                )))
                .insert(ActiveEvents::COLLISION_EVENTS)
                .insert(Collider::cylinder(0.2, 0.45))
                .insert(Cauldron)
                .insert(SlotDeposit::new(slots))
                .insert(Sensor);
        });

    cauldron
}
