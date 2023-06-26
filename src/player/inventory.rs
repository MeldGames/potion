use std::cmp::Ordering;

use crate::player::prelude::*;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

#[derive(Component, Clone, Debug, Reflect)]
pub struct Inventory {
    pub items: Vec<Option<Entity>>,
}

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
        app.add_system(store_item);
    }
}

pub fn store_item(
    keycode: Res<Input<KeyCode>>,
    children: Query<&Children>,
    joint_children: Query<&JointChildren>,
    mut inventories: Query<(Entity, &mut Inventory)>,
    mut hands: Query<(Entity, &LastActive), With<Hand>>,
    mut grabbing: Query<&mut Grabbing>,

    names: Query<&Name>,
) {
    let swap = if keycode.just_pressed(KeyCode::Key1) {
        Some(1)
    } else {
        None
    };

    for (entity, mut inventory) in &mut inventories {
        let swap_index = if let Some(swap_index) = swap {
            swap_index
        } else {
            continue;
        };

        let mut hands = find_children_with(&hands, &children, &joint_children, entity);
        hands.sort_by(|(a_entity, a), (b_entity, b)| {
            let a_grab = grabbing.get(*a_entity).map(|grabbing| grabbing.trying_grab).unwrap_or(false);
            let b_grab = grabbing.get(*b_entity).map(|grabbing| grabbing.trying_grab).unwrap_or(false);

if a_grab && !b_grab {
    Ordering::Less
} else if b_grab && !a_grab {
    Ordering::Greater
} else {
    b.0.cmp(&a.0)
}
        });

        let target = inventory.items[swap_index];

        let prioritize_storing = target.is_some();

        // last active hand
        let hand = hands[0].0;

        info!("last active hand: {:?}", names.get(hand).unwrap().as_str());
        if let Ok(mut grabbing) = grabbing.get_mut(hand) {
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
