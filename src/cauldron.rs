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
) -> Entity {
    let level_collision_mesh: Handle<Mesh> =
        asset_server.load("models/cauldron.glb#Mesh0/Primitive0");
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
        .id();

    let mut slots = Vec::new();
    slots.push(commands.spawn().insert(Slot::default()).id());
    slots.push(commands.spawn().insert(Slot::default()).id());
    slots.push(commands.spawn().insert(Slot::default()).id());

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
