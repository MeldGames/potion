use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{deposit::Value};

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
pub struct SecurityCheck {
    pub push: Vec3,
}

/// Buy a store item here.
#[derive(Debug, Component, Clone, Copy)]
pub struct Register;

#[derive(Debug, Component, Clone, Copy)]
pub struct CaughtItem {
    pub caught_time: f32,
    pub by: Entity,
}

impl CaughtItem {
    pub fn new(by: Entity) -> Self {
        CaughtItem {
            caught_time: 0.0,
            by: by,
        }
    }
}

pub fn push_item_back(
    mut commands: Commands,
    name: Query<DebugName>,
    rapier_context: Res<RapierContext>,
    security_checks: Query<(Entity, &GlobalTransform, &SecurityCheck, Option<&Children>)>,
    mut store_items: Query<(
        Entity,
        &GlobalTransform,
        Option<&mut ExternalImpulse>,
        &StoreItem,
    )>,
) {
    for (security_entity, security_transform, security_check, _children) in &security_checks {
        for (collider1, collider2, intersecting) in
            rapier_context.intersections_with(security_entity)
        {
            let potential = if collider1 == security_entity {
                collider2
            } else {
                collider1
            };

            if intersecting {
                if let Ok((item_entity, item_transform, impulse, _store_item)) =
                    store_items.get_mut(potential)
                {
                    info!("Player is trying to steal {:?}", name.get(potential).unwrap());
                    commands
                        .entity(item_entity)
                        .insert(CaughtItem::new(security_entity));

                    // Push object tangential to the push direction as well, to avoid
                    // getting stuck on walls hopefully.
                    let center_dir = (security_transform.translation()
                        - item_transform.translation())
                    .normalize_or_zero();

                    let push_dir = security_check.push.normalize_or_zero();
                    let (tangent1, tangent2) = push_dir.any_orthonormal_pair();
                    let tangent = (tangent1.abs() + tangent2.abs()).normalize_or_zero();
                    let tangential_push = tangent * center_dir;

                    let new_impulse = ExternalImpulse {
                        impulse: security_check.push * 0.1,
                        torque_impulse: tangential_push * 0.1,
                    };

                    match impulse {
                        Some(mut impulse) => {
                            impulse.impulse += new_impulse.impulse;
                            impulse.torque_impulse += new_impulse.torque_impulse;
                        }
                        None => {
                            commands.entity(item_entity).insert(new_impulse);
                        }
                    }
                }
            }
        }
    }
}

pub fn buy_item(
    mut commands: Commands,
    name: Query<DebugName>,
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
                        info!("Player buying {:?}", name.get(potential).unwrap());
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
    fn build(&self, _app: &mut App) {
        //app.add_network_system(push_item_back);
        //app.add_network_system(buy_item);
    }
}
