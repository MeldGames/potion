use std::cmp::Ordering;

use crate::{objects::cauldron::Ingredient, player::prelude::*, FixedSet};
use bevy::prelude::*;
use bevy_rapier3d::{
    parry::shape::{
        Cone, ConvexPolyhedron, Cuboid, Cylinder, RoundShape, SharedShape, Triangle, TypedShape,
    },
    prelude::*,
};

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
                .after(store_item),
        );

        // Niceties
        app.add_system(ingredients_are_storable.in_schedule(CoreSchedule::FixedUpdate));
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

#[derive(Component, Debug, Copy, Clone, Reflect, Default)]
#[reflect(Component)]
pub struct InventoryJoint;

#[derive(Component, Debug, Copy, Clone, Reflect, Default)]
#[reflect(Component)]
pub struct Stored {
    pub scale: Vec3,
    pub border_radius: Option<f32>,
}

pub fn transform_stored(
    mut commands: Commands,
    inventories: Query<(Entity, &Inventory)>,
    stored: Query<&Stored>,
    mut rapier: ResMut<RapierContext>,
    mut items: Query<(&mut Transform, &mut Collider, &RapierColliderHandle)>,
) {
    // Scale of item when slotted in inventory hotswaps.
    const NORMALIZED_SCALE: f32 = 0.3;
    // Space inbetween items in inventory hotswaps
    const PADDING: f32 = 0.1;

    for (entity, inventory) in &inventories {
        for (index, item) in inventory.items.iter().enumerate() {
            let Some(item) = item else { continue };
            if stored.contains(*item) {
                continue;
            }
            //let slots = inventory.len();
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

            if let Ok((mut transform, mut collider, collider_handle)) = items.get_mut(*item) {
                let mut rapier_collider = rapier.colliders.get_mut(collider_handle.0).unwrap();

                let mut extents: Vec3 = {
                    let mut collider = rapier_collider.clone();
                    collider.set_rotation(default());
                    collider.set_translation(default());
                    let aabb = collider.compute_aabb();
                    let extents = aabb.extents();
                    extents.into()
                };

                extents *= collider.scale();

                let max = extents.x.max(extents.y).max(extents.z);
                let ratio = NORMALIZED_SCALE / max;

                let prev_scale = transform.scale;
                transform.scale *= ratio;

                let prev_border_radius = match collider.raw.as_typed_shape() {
                    TypedShape::RoundCylinder(RoundShape {
                        inner_shape:
                            Cylinder {
                                radius,
                                half_height,
                            },
                        border_radius,
                    }) => {
                        let prev = *border_radius;
                        let shape = collider.raw.make_mut();
                        if let Some(round) = shape.as_round_cylinder_mut() {
                            round.border_radius = round.border_radius * ratio;
                            *collider = SharedShape::new(round.clone()).into();
                        }
                        Some(prev)
                    }
                    TypedShape::RoundCuboid(RoundShape {
                        inner_shape: Cuboid { half_extents },
                        border_radius,
                    }) => {
                        let prev = *border_radius;
                        *collider = Collider::round_cuboid(
                            half_extents.x,
                            half_extents.y,
                            half_extents.z,
                            border_radius * ratio,
                        );
                        Some(prev)
                    }
                    TypedShape::RoundTriangle(RoundShape {
                        inner_shape: Triangle { a, b, c },
                        border_radius,
                    }) => {
                        let prev = *border_radius;
                        *collider = Collider::round_triangle(
                            (*a).into(),
                            (*b).into(),
                            (*c).into(),
                            border_radius * ratio,
                        );
                        Some(prev)
                    }
                    TypedShape::RoundCone(RoundShape {
                        inner_shape:
                            Cone {
                                half_height,
                                radius,
                            },
                        border_radius,
                    }) => {
                        let prev = *border_radius;
                        *collider =
                            Collider::round_cone(*half_height, *radius, border_radius * ratio);
                        Some(prev)
                    }
                    TypedShape::RoundConvexPolyhedron(RoundShape {
                        inner_shape,
                        border_radius,
                    }) => {
                        let prev = *border_radius;
                        let shape = collider.raw.make_mut();
                        if let Some(round) = shape.as_round_convex_polyhedron_mut() {
                            round.border_radius = round.border_radius * ratio;
                            *collider = SharedShape::new(round.clone()).into();
                        }
                        Some(prev)
                    }
                    _ => None,
                };

                commands
                    .entity(*item)
                    .insert(Stored {
                        scale: prev_scale,
                        border_radius: prev_border_radius,
                    })
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

        let Ok(mut grabbing) = grabbing.get_mut(hand) else { continue };
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
