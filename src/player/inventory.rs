use std::cmp::Ordering;

use crate::{objects::cauldron::Ingredient, prelude::*};
use bevy::prelude::*;
use bevy_rapier3d::{
    parry::shape::{Cone, Cuboid, Cylinder, RoundShape, SharedShape, Triangle, TypedShape},
    prelude::*,
    rapier::dynamics::{JointAxesMask, JointAxis},
};

pub mod prelude {
    pub use super::{Inventory, Storeable};
}

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct Inventory {
    pub items: Vec<Option<Grabbed>>,
}

impl Default for Inventory {
    fn default() -> Self {
        Self {
            items: vec![None; 4],
        }
    }
}

impl Inventory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn contains(&self, entity: Entity) -> bool {
        self.items
            .iter()
            .find(|item| item.is_some_and(|item| item.entity == entity))
            .is_some()
    }
}

/// Item is allowed to the scaled and fitted into an [`Inventory`].
#[derive(Component, Clone, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct Storeable;

pub struct InventoryPlugin;
impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Storeable>()
            .register_type::<Ingredient>()
            .register_type::<Inventory>();

        // Core
        app.add_systems(
            FixedUpdate,
            store_item
                .before(reset_inputs),
        );

        app.add_systems(FixedUpdate, transform_stored.after(store_item));

        // Niceties
        app.add_systems(FixedUpdate, ingredients_are_storable);
    }
}

pub fn ingredients_are_storable(
    mut commands: Commands,
    ingredients: Query<Entity, (With<Ingredient>, Without<Storeable>)>,
) {
    for entity in &ingredients {
        commands.entity(entity).insert(Storeable);
    }
}

/// Joint forcing an item into a slot in an [`Inventory`].
#[derive(Component, Debug, Copy, Clone, Reflect, Default)]
#[reflect(Component)]
pub struct InventoryJoint;

/// Contains information for the item to know how much it was scaled down
/// to fit the [`Inventory`]'s requirements.
#[derive(Component, Debug, Copy, Clone)]
pub struct Stored {
    pub inventory: Entity,
    pub scaled_ratio: f32,
}

/// Scaling border radius is awkward and hard to reverse right now
/// so this is just kind of a dead function for now
pub fn scale_border_radius(collider: &mut Collider, ratio: f32) {
    match collider.raw.as_typed_shape() {
        TypedShape::RoundCylinder(RoundShape {
            inner_shape:
                Cylinder {
                    half_height,
                    radius,
                },
            border_radius,
        }) => {
            *collider = Collider::round_cylinder(*half_height, *radius, border_radius * ratio);
            //collider.set_scale(prev_scale * ratio, 32);
        }
        TypedShape::RoundCuboid(RoundShape {
            inner_shape: Cuboid { half_extents },
            border_radius,
        }) => {
            *collider = Collider::round_cuboid(
                half_extents.x,
                half_extents.y,
                half_extents.z,
                border_radius * ratio,
            );
        }
        TypedShape::RoundTriangle(RoundShape {
            inner_shape: Triangle { a, b, c },
            border_radius,
        }) => {
            *collider = Collider::round_triangle(
                (*a).into(),
                (*b).into(),
                (*c).into(),
                border_radius * ratio,
            );
        }
        TypedShape::RoundCone(RoundShape {
            inner_shape: Cone {
                half_height,
                radius,
            },
            border_radius,
        }) => {
            *collider = Collider::round_cone(*half_height, *radius, border_radius * ratio);
        }
        TypedShape::RoundConvexPolyhedron(..) => {
            let shape = collider.raw.make_mut();
            if let Some(round) = shape.as_round_convex_polyhedron_mut() {
                round.border_radius = round.border_radius * ratio;
                *collider = SharedShape::new(round.clone()).into();
            }
        }
        _ => {}
    };
}

/// Scale/apply a joint to stored items so they fit and stay in an inventory.
pub fn transform_stored(
    mut commands: Commands,
    inventories: Query<(Entity, &Inventory)>,
    stored: Query<(Entity, Option<&Children>, &Stored)>,
    inventory_joints: Query<&InventoryJoint>,
    joint_children: Query<&JointChildren>,

    mut impulse_joints: Query<&mut ImpulseJoint>,
    mut multibody_joints: Query<&mut MultibodyJoint>,

    mut transforms: Query<&mut Transform>,
    rapier: Res<RapierContext>,
    colliders: Query<&RapierColliderHandle>,

    names: Query<&Name>,
) {
    // Scale of item when slotted in inventory hotswaps.
    const NORMALIZED_SCALE: f32 = 0.3;
    // Space inbetween items in inventory hotswaps
    const PADDING: f32 = 0.1;

    let debug_name = |entity: Entity| {
        names
            .get(entity)
            .map(|name| name.as_str().to_owned())
            .unwrap_or(format!("{:?}", entity))
    };

    // If item was in an inventory and was removed, then unscale it and unjoint it.
    for (entity, children, stored) in &stored {
        let still_stored = if let Ok((_, inventory)) = inventories.get(stored.inventory) {
            inventory.contains(entity)
        } else {
            false
        };

        if !still_stored {
            info!("removing from storage");

            let Ok(mut transform) = transforms.get_mut(entity) else {
                continue;
            };

            let ratio = 1.0 / stored.scaled_ratio;
            transform.scale *= ratio;
            //scale_border_radius(&mut collider, ratio);

            // Need to figure out how to simplify this in the future.
            let mut joint_attached = Vec::new();
            if let Ok(joint_children) = joint_children.get(entity) {
                joint_attached.extend(joint_children.0.iter());
            }

            if let Ok(joint) = impulse_joints.get(entity) {
                joint_attached.push(joint.parent);
            }

            if let Ok(joint) = multibody_joints.get(entity) {
                joint_attached.push(joint.parent);
            }

            for child in &joint_attached {
                info!("joint child: {:?}", debug_name(*child));

                if let Ok(mut transform) = transforms.get_mut(*child) {
                    transform.scale *= ratio;
                }

                if let Ok(mut impulse) = impulse_joints.get_mut(*child) {
                    let anchor1 = impulse.data.local_anchor1();
                    impulse.data.set_local_anchor1(anchor1 * ratio);

                    let anchor2 = impulse.data.local_anchor2();
                    impulse.data.set_local_anchor2(anchor2 * ratio);
                }

                if let Ok(mut multibody) = multibody_joints.get_mut(*child) {
                    let anchor1 = multibody.data.local_anchor1();
                    multibody.data.set_local_anchor1(anchor1 * ratio);

                    let anchor2 = multibody.data.local_anchor2();
                    multibody.data.set_local_anchor2(anchor2 * ratio);
                }
            }

            // remove joint
            if let Some(children) = children {
                for child in children {
                    if inventory_joints.contains(*child) {
                        commands.entity(*child).despawn_recursive();
                    }
                }
            }

            commands.entity(entity).remove::<Stored>();
        }
    }

    // If item is now in an inventory, then scale it and joint it.
    for (inventory_entity, inventory) in &inventories {
        for (index, item) in inventory.items.iter().enumerate() {
            let Some(item) = item else { continue };
            if stored.contains(item.entity) {
                continue;
            }
            let Ok(mut transform) = transforms.get_mut(item.entity) else {
                continue;
            };

            //let slots = inventory.items.len();
            let slots = 4;
            let left_align = (NORMALIZED_SCALE + PADDING) * ((slots / 2) as f32 - 0.5);

            let mut inventory_joint = FixedJointBuilder::new()
                .local_anchor1(Vec3::new(
                    (index as f32 * (NORMALIZED_SCALE + PADDING)) - left_align,
                    1.0,
                    0.5,
                ))
                .build();
            inventory_joint.set_contacts_enabled(false);

            let extents: Vec3 = {
                if let Ok(collider_handle) = colliders.get(item.entity) {
                    let Some(rapier_collider) = rapier.colliders.get(collider_handle.0) else {
                        continue;
                    };

                    let mut collider = rapier_collider.clone();
                    collider.set_rotation(default());
                    collider.set_translation(default());
                    let aabb = collider.compute_aabb();
                    let extents = aabb.extents();
                    extents.into()
                } else {
                    Vec3::splat(1.)
                }
            };

            let max = extents.x.max(extents.y).max(extents.z);
            let ratio = NORMALIZED_SCALE / max;

            transform.scale *= ratio;
            //scale_border_radius(&mut collider, ratio);

            let mut joint_attached = Vec::new();
            if let Ok(joint_children) = joint_children.get(item.entity) {
                joint_attached.extend(joint_children.0.iter());
            }

            if let Ok(joint) = impulse_joints.get(item.entity) {
                joint_attached.push(joint.parent);
            }

            if let Ok(joint) = multibody_joints.get(item.entity) {
                joint_attached.push(joint.parent);
            }

            for child in &joint_attached {
                info!("joint child: {:?}", debug_name(*child));
                if let Ok(mut transform) = transforms.get_mut(*child) {
                    transform.scale *= ratio;
                }

                if let Ok(mut impulse) = impulse_joints.get_mut(*child) {
                    let anchor1 = impulse.data.local_anchor1();
                    impulse.data.set_local_anchor1(anchor1 * ratio);

                    let anchor2 = impulse.data.local_anchor2();
                    impulse.data.set_local_anchor2(anchor2 * ratio);
                }

                if let Ok(mut multibody) = multibody_joints.get_mut(*child) {
                    let anchor1 = multibody.data.local_anchor1();
                    multibody.data.set_local_anchor1(anchor1 * ratio);

                    let anchor2 = multibody.data.local_anchor2();
                    multibody.data.set_local_anchor2(anchor2 * ratio);
                }
            }

            commands
                .entity(item.entity)
                .insert(Stored {
                    inventory: inventory_entity,
                    scaled_ratio: ratio,
                })
                .with_children(|children| {
                    children
                        .spawn(ImpulseJoint::new(inventory_entity, inventory_joint))
                        .insert(InventoryJoint)
                        .insert(Name::new("Inventory Joint"));
                });
        }
    }
}

/// Manage the meta information about an [`Inventory`] based on input from the player.
pub fn store_item(
    children: Query<&Children>,
    joint_children: Query<&JointChildren>,
    mut inventories: Query<(Entity, &mut Inventory, &PlayerInput)>,
    hands: Query<(Entity, &LastActive), With<Hand>>,
    mut grabbing: Query<&mut Grabbing>,
    storeable: Query<&Storeable>,
) {
    for (entity, mut inventory, input) in &mut inventories {
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

        // last active hand
        let Some(hand) = hands.get(0) else { continue };
        let hand = hand.0;

        let target = inventory.items[swap_index].map(|mut target| {
            target.teleport_entity = true;
            target
        });

        let Ok(mut grabbing) = grabbing.get_mut(hand) else {
            continue;
        };
        if let Some(grabbed) = grabbing.grabbed {
            if !storeable.contains(grabbed.entity) {
                info!("Object is not storeable");
                continue;
            }
        }

        match (grabbing.grabbed, inventory.items[swap_index]) {
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

        inventory.items[swap_index] = grabbing.grabbed;
        grabbing.grabbed = target;
    }
}
