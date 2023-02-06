use bevy::prelude::*;

use bevy_rapier3d::prelude::*;

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
    fn named<'a>(&'a self, entity: Entity) -> Box<dyn std::fmt::Debug + 'a>;
}

impl<'w, 's> NamedEntity for Query<'w, 's, &Name, ()> {
    fn named<'a>(&'a self, entity: Entity) -> Box<dyn std::fmt::Debug + 'a> {
        match self.get_component::<Name>(entity) {
            Ok(name) => Box::new(name.as_str()),
            _ => Box::new(entity),
        }
    }
}

pub struct CauldronPlugin;
impl Plugin for CauldronPlugin {
    fn build(&self, _app: &mut App) {
        //app.add_network_system(slot_ingredient);
    }
}

pub fn spawn_cauldron(
    commands: &mut Commands,
    asset_server: &AssetServer,
    position: Transform,
    meshes: &mut ResMut<Assets<Mesh>>,
) -> Entity {
    let level_collision_mesh: Handle<Mesh> =
        asset_server.load("models/cauldron.gltf#Mesh0/Primitive0");

    let mut slots = Vec::new();
    let cauldron = commands
        .spawn(SceneBundle {
            scene: asset_server.load("models/cauldron.gltf#Scene0"),
            transform: position,
            ..default()
        })
        .insert((
            ColliderMassProperties::Density(1.0),
            ReadMassProperties::default(),
            RigidBody::Dynamic,
            Velocity::default(),
            ExternalImpulse::default(),
            crate::store::StoreItem,
            Collider::cylinder(0.5, 0.75),
            Name::new("Cauldron"),
            crate::physics::TERRAIN_GROUPING,
        ))
        .insert(crate::DecompLoad(
            "assets/models/cauldron_decomp2.obj".to_owned(),
        ))
        .insert(level_collision_mesh)
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
                                center + Vec3::new(x, 1.5, z) * radius,
                            ),
                            ..default()
                        })
                        .insert(Name::new(format!("Cauldron slot {}", i)))
                        .insert(Velocity::default())
                        .insert(Slot::default())
                        .insert(crate::DebugVisible)
                        .insert(crate::slot::SlotGracePeriod::default())
                        .insert(crate::slot::SlotSettings(springy::SpringState {
                            spring: springy::Spring {
                                strength: 1.00,
                                damp_ratio: 0.2,
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
