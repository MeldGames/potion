use std::cmp::Ordering;

use crate::{player::prelude::*, FixedSet, objects::cauldron::Ingredient};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

#[derive(Component, Clone, Debug, Reflect)]
pub struct Inventory {
    pub items: Vec<Option<Entity>>,
}

#[derive(Component, Clone, Debug, Reflect)]
pub struct Storeable;

impl Default for Inventory {
    fn default() -> Self {
        Self {
            items: vec![None; 8],
        }
    }
}

pub struct InventoryPlugin;
impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        // Core
        app.add_system(
            store_item
                .in_schedule(CoreSchedule::FixedUpdate)
                .after(update_local_player_inputs)
                .before(reset_inputs),
        );

        app.add_system(
            transform_stored
                .in_schedule(CoreSchedule::FixedUpdate)
                .after(store_item)
        );


        // Niceties
        app.add_system(
            ingredients_are_storable
                .in_schedule(CoreSchedule::FixedUpdate)
        );
    }
}

pub fn ingredients_are_storable(mut commands: Commands, ingredients: Query<Entity, (With<Ingredient>, Without<Storeable>)>) {
    for entity in &ingredients {
        commands.entity(entity).insert(Storeable);
    }
}

#[derive(Component, Debug, Copy, Clone, Reflect, Default)]
#[reflect(Component)]
pub struct InventoryJoint;

#[derive(Component, Debug, Copy, Clone, Reflect, Default)]
#[reflect(Component)]
pub struct Stored;

pub fn transform_stored(
    mut commands: Commands,
    inventories: Query<(Entity, &Inventory)>,
    stored: Query<&Stored>,
) {
    for (entity, inventory) in &inventories {
        for (index, item) in inventory.items.iter().enumerate() {
            if let Some(item) = item {
                if !stored.contains(*item) {
                    let mut inventory_joint = FixedJointBuilder::new()
                        .local_anchor1(Vec3::new(index as f32, 0.0, 2.0))
                        .build();
                    inventory_joint.set_contacts_enabled(false);

                    commands.entity(*item)
                    .insert(Stored)
                    .with_children(|children| {
                        children
                            .spawn(ImpulseJoint::new(entity, inventory_joint))
                            .insert(InventoryJoint)
                            .insert(Name::new("Inventory Joint"));
                    });
                }
            }
        }
    }
}

pub fn store_item(
    children: Query<&Children>,
    joint_children: Query<&JointChildren>,
    mut inventories: Query<(Entity, &mut Inventory, &mut PlayerInput)>,
    mut hands: Query<(Entity, &LastActive), With<Hand>>,
    mut grabbing: Query<&mut Grabbing>,
    storeable: Query<&Storeable>,

    names: Query<&Name>,
) {
    for (entity, mut inventory, mut input) in &mut inventories {
        let swap_index = if let Some(swap_index) = input.inventory_swap() {
            swap_index as usize
        } else {
            continue;
        };

        let mut hands = find_children_with(&hands, &children, &joint_children, entity);
        hands.sort_by(|(a_entity, a), (b_entity, b)| {
            let a_grab = grabbing
                .get(*a_entity)
                .map(|grabbing| grabbing.trying_grab)
                .unwrap_or(false);
            let b_grab = grabbing
                .get(*b_entity)
                .map(|grabbing| grabbing.trying_grab)
                .unwrap_or(false);

            if a_grab && !b_grab {
                Ordering::Less
            } else if b_grab && !a_grab {
                Ordering::Greater
            } else {
                b.0.cmp(&a.0)
            }
        });

        let target = inventory.items[swap_index];

        // last active hand
        let hand = hands[0].0;

        if let Ok(mut grabbing) = grabbing.get_mut(hand) {
            if let Some(grabbing) = grabbing.grabbing {
                if !storeable.contains(grabbing) {
                    info!("Object is not storeable");
                    continue;
                }
            }

            match (grabbing.grabbing, inventory.items[swap_index]) {
                (Some(grabbing), Some(item)) => {
                    info!("Swapping {:?} and {:?}", grabbing, item);
                }
                (None, Some(item)) => {
                    info!("Grabbing {:?} from inventory", item);
                }
                (Some(grabbing), None) => {
                    info!("Storing {:?} in inventory", grabbing);
                }
                _ => {}
            }

            inventory.items[swap_index] = grabbing.grabbing;
            grabbing.grabbing = target;
        }
    }
}
