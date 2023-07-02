use std::cmp::Ordering;

use crate::{objects::cauldron::Ingredient, player::prelude::*};
use bevy::prelude::*;
use bevy_rapier3d::{
    parry::shape::{Cone, Cuboid, Cylinder, RoundShape, SharedShape, Triangle, TypedShape},
    prelude::*,
    rapier::dynamics::{JointAxesMask, JointAxis},
};

#[derive(Component, Clone, Debug, Reflect, FromReflect)]
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

#[derive(Component, Clone, Debug, Reflect)]
pub struct Storeable;

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

#[derive(Component, Debug, Copy, Clone)]
pub struct Stored {
    pub inventory: Entity,
    pub scaled_ratio: f32,
}

// scaling border radius is awkward and hard to reverse right now
// so this is just kind of a dead function for now
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
        TypedShape::RoundConvexPolyhedron(RoundShape {
            inner_shape,
            border_radius,
        }) => {
            let shape = collider.raw.make_mut();
            if let Some(round) = shape.as_round_convex_polyhedron_mut() {
                round.border_radius = round.border_radius * ratio;
                *collider = SharedShape::new(round.clone()).into();
            }
        }
        _ => {}
    };
}

pub fn transform_stored(
    mut commands: Commands,
    inventories: Query<(Entity, &Inventory)>,
    stored: Query<(Entity, Option<&Children>, &Stored)>,
    inventory_joints: Query<&InventoryJoint>,

    mut transforms: Query<&mut Transform>,
    mut rapier: ResMut<RapierContext>,
    mut colliders: Query<(&mut Collider, &RapierColliderHandle)>,
) {
    // Scale of item when slotted in inventory hotswaps.
    const NORMALIZED_SCALE: f32 = 0.3;
    // Space inbetween items in inventory hotswaps
    const PADDING: f32 = 0.1;

    // If item was in an inventory and was removed, then unscale it and unjoint it.
    for (entity, children, stored) in &stored {
        let still_stored = if let Ok((_, inventory)) = inventories.get(stored.inventory) {
            inventory.contains(entity)
        } else {
            false
        };

        if !still_stored {
            let Ok(mut transform) = transforms.get_mut(entity) else { continue };
            let Ok((mut collider, collider_handle)) = colliders.get_mut(entity) else { continue };

            let ratio = 1.0 / stored.scaled_ratio;
            transform.scale *= ratio;
            //scale_border_radius(&mut collider, ratio);

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
    for (entity, inventory) in &inventories {
        for (index, item) in inventory.items.iter().enumerate() {
            let Some(item) = item else { continue };
            if stored.contains(item.entity) {
                continue;
            }
            let Ok(mut transform) = transforms.get_mut(item.entity) else { continue };
            let Ok((mut collider, collider_handle)) = colliders.get_mut(item.entity) else { continue };

            //let slots = inventory.len();
            let slots = 4;
            let left_align = (NORMALIZED_SCALE + PADDING) * ((slots / 2) as f32 - 0.5);

            let strength = 5000.0;
            let damping = 5.0;
            let mut inventory_joint = GenericJointBuilder::new(JointAxesMask::empty())
                .local_anchor1(Vec3::new(
                    (index as f32 * (NORMALIZED_SCALE + PADDING)) - left_align,
                    1.0,
                    0.5,
                ))
                .motor_position(JointAxis::X, 0.0, strength, damping)
                .motor_position(JointAxis::Y, 0.0, strength, damping)
                .motor_position(JointAxis::Z, 0.0, strength, damping)
                .motor_position(JointAxis::AngX, 0.0, strength, damping)
                .motor_position(JointAxis::AngY, 0.0, strength, damping)
                .motor_position(JointAxis::AngZ, 0.0, strength, damping)
                .build();
            inventory_joint.set_contacts_enabled(false);

            let mut rapier_collider = rapier.colliders.get_mut(collider_handle.0).unwrap();

            let mut extents: Vec3 = {
                let mut collider = rapier_collider.clone();
                collider.set_rotation(default());
                collider.set_translation(default());
                let aabb = collider.compute_aabb();
                let extents = aabb.extents();
                extents.into()
            };

            let max = extents.x.max(extents.y).max(extents.z);
            let ratio = NORMALIZED_SCALE / max;

            transform.scale *= ratio;
            //scale_border_radius(&mut collider, ratio);

            commands
                .entity(item.entity)
                .insert(Stored {
                    inventory: entity,
                    scaled_ratio: ratio,
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
