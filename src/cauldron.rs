use bevy::{ecs::query::WorldQuery, prelude::*, utils::HashSet};
use bevy_inspector_egui::{Inspectable, RegisterInspectable};
use bevy_rapier3d::prelude::*;
use sabi::stage::NetworkSimulationAppExt;

use crate::{
    attach::Attach,
    slot::{Slot, SlotDeposit},
};

#[derive(Default, Debug, Copy, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct Cauldron;

#[derive(Default, Debug, Copy, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct Ingredient;

pub trait NamedEntity {
    fn named(&self, entity: Entity) -> String;
}

impl<'w, 's, F: WorldQuery> NamedEntity for Query<'w, 's, &Name, F> {
    fn named(&self, entity: Entity) -> String {
        match self.get_component::<Name>(entity) {
            Ok(name) => name.as_str().to_owned(),
            _ => format!("{:?}", entity),
        }
    }
}

pub struct CauldronPlugin;
impl Plugin for CauldronPlugin {
    fn build(&self, app: &mut App) {
        //app.add_network_system(slot_ingredient);
    }
}

pub fn spawn_cauldron(
    commands: &mut Commands,
    asset_server: &AssetServer,
    position: Transform,
    mut meshes: &mut ResMut<Assets<Mesh>>,
) -> Entity {
    let level_collision_mesh: Handle<Mesh> =
        asset_server.load("models/cauldron.glb#Mesh0/Primitive0");

    let mut slots = Vec::new();
    let cauldron = commands
        .spawn_bundle(SceneBundle {
            scene: asset_server.load("models/cauldron.glb#Scene0"),
            transform: position,
            ..default()
        })
        .insert_bundle((
            ColliderMassProperties::Density(100.0),
            ReadMassProperties::default(),
            RigidBody::Dynamic,
            Velocity::default(),
            ExternalImpulse::default(),
            crate::store::StoreItem,
            Collider::cylinder(0.4, 0.75),
            Name::new("Cauldron"),
            crate::physics::TERRAIN_GROUPING,
        ))
        .insert(crate::DecompLoad(
            "assets/models/cauldron_decomp.obj".to_owned(),
        ))
        .insert(level_collision_mesh)
        .with_children(|builder| {
            let center = Vec3::new(0.0, 0.4, 0.0);
            let radius = 0.3;
            let slot_count = 3;

            let slice = std::f32::consts::TAU / slot_count as f32;
            for i in 0..slot_count {
                let x = (slice * i as f32).cos();
                let z = (slice * i as f32).sin();
                slots.push(
                    builder
                        .spawn()
                        .insert_bundle(PbrBundle {
                            mesh: meshes.add(Mesh::from(shape::UVSphere {
                                radius: 0.02,
                                ..default()
                            })),
                            ..default()
                        })
                        .insert_bundle(TransformBundle::from_transform(
                            Transform::from_translation(center + Vec3::new(x, 0.0, z) * radius),
                        ))
                        .insert(Name::new(format!("Cauldron slot {}", i)))
                        .insert(Velocity::default())
                        .insert(Slot::default())
                        .insert(crate::slot::SlotGracePeriod::default())
                        .insert(crate::slot::SlotSettings(springy::SpringState {
                            spring: springy::Spring {
                                strength: 0.85,
                                damp_ratio: 0.7,
                                rest_distance: 0.0,
                                limp_distance: 0.0,
                            },
                            breaking: Some(springy::SpringBreak {
                                tear_force: 3.0,
                                tear_step: 0.02,
                                heal_step: 0.05,
                                ..default()
                            }),
                            ..default()
                        }))
                        .id(),
                );
            }
        })
        .id();

    commands
        .spawn_bundle(TransformBundle::from_transform(position))
        .insert_bundle(Attach::all(cauldron))
        .insert_bundle((
            Name::new("Cauldron Deposit"),
            crate::physics::TERRAIN_GROUPING,
        ))
        .with_children(|children| {
            children
                .spawn_bundle(TransformBundle::from_transform(Transform::from_xyz(
                    0.0, 0.25, 0.0,
                )))
                .insert(ActiveEvents::COLLISION_EVENTS)
                .insert(Collider::cylinder(0.4, 0.55))
                .insert(Cauldron)
                .insert(SlotDeposit::new(slots))
                .insert(Sensor);
        });

    cauldron
}
