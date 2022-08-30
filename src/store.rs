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
pub struct StoreItem;

/// Teleports store items back into the store they should be in.
#[derive(Debug, Component, Clone, Copy)]
pub struct SecurityCheck;

/// Buy a store item here.
#[derive(Debug, Component, Clone, Copy)]
pub struct Register;

pub fn push_item_back(
    mut commands: Commands,
    name: Query<&Name>,
    rapier_context: Res<RapierContext>,
    security_checks: Query<(Entity, Option<&Children>), With<SecurityCheck>>,
    mut store_items: Query<(Entity, Option<&mut ExternalImpulse>, &StoreItem)>,
) {
    for (entity, _children) in &security_checks {
        for (collider1, collider2, intersecting) in rapier_context.intersections_with(entity) {
            let potential = if collider1 == entity {
                collider2
            } else {
                collider1
            };

            if intersecting {
                if let Ok((item_entity, impulse, _store_item)) = store_items.get_mut(potential) {
                    info!("Player is trying to steal {:?}", name.named(potential));
                    let push_direction = Vec3::Z * 0.01;
                    match impulse {
                        Some(mut impulse) => {
                            impulse.impulse += push_direction;
                        }
                        None => {
                            commands.entity(item_entity).insert(ExternalImpulse {
                                impulse: push_direction,
                                ..default()
                            });
                        }
                    }
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
    for (entity, _children) in &registers {
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

pub struct StorePlugin;
impl Plugin for StorePlugin {
    fn build(&self, app: &mut App) {
        use sabi::stage::NetworkSimulationAppExt;
        app.add_network_system(push_item_back);
        app.add_network_system(buy_item);
    }
}
