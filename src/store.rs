use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{cauldron::NamedEntity, deposit::Value};

#[derive(Debug, Component, Clone, Copy)]
pub struct Store;

#[derive(Debug, Component, Clone, Copy)]
pub struct StoreSlot {
    pub store: Entity,
}

#[derive(Debug, Component, Clone, Copy)]
pub struct StoreItem {
    pub store: Entity,
    pub location: Transform,
}

/// Teleports store items back into the store they should be in.
#[derive(Debug, Component, Clone, Copy)]
pub struct SecurityCheck;

/// Buy a store item here.
#[derive(Debug, Component, Clone, Copy)]
pub struct Register;

pub fn teleport_item_back(
    mut commands: Commands,
    name: Query<&Name>,
    rapier_context: Res<RapierContext>,
    security_checks: Query<(Entity, Option<&Children>), With<SecurityCheck>>,
    store_items: Query<&StoreItem>,
    mut transforms: Query<&mut Transform>,
) {
    for (entity, children) in &security_checks {
        for (collider1, collider2, intersecting) in rapier_context.intersections_with(entity) {
            let potential = if collider1 == entity {
                collider2
            } else {
                collider1
            };

            if intersecting {
                if let Ok(store_item) = store_items.get(potential) {
                    info!("Player tried to steal {:?}", name.named(potential));
                    let mut transform = transforms
                        .get_mut(potential)
                        .expect("Store item should have a transform");
                    *transform = store_item.location;
                }
            }
        }
    }
}

pub fn buy_item(
    mut commands: Commands,
    name: Query<&Name>,
    rapier_context: Res<RapierContext>,
    registers: Query<(Entity, Option<&Children>), With<Register>>,
    store_items: Query<(&Value, &StoreItem)>,
    mut player_value: ResMut<Value>,
) {
    for (entity, children) in &registers {
        for (collider1, collider2, intersecting) in rapier_context.intersections_with(entity) {
            let potential = if collider1 == entity {
                collider2
            } else {
                collider1
            };

            if intersecting {
                if let Ok((value, _)) = store_items.get(potential) {
                    if player_value.enough(value) {
                        info!("Player buying {:?}", name.named(potential));
                        *player_value -= *value;
                        commands.entity(entity).remove::<StoreItem>();
                    }
                }
            }
        }
    }
}
