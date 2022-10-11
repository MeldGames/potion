use std::collections::VecDeque;

use bevy::{ecs::query::WorldQuery, prelude::*, utils::HashSet};
use bevy_inspector_egui::{Inspectable, RegisterInspectable};
use bevy_prototype_debug_lines::DebugLines;
use bevy_rapier3d::prelude::*;
use sabi::stage::NetworkSimulationAppExt;

use crate::{
    attach::Attach,
    cauldron::{Ingredient, NamedEntity},
};

#[derive(Default, Debug, Copy, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct Slot {
    pub containing: Option<Entity>,
}

#[derive(Default, Debug, Copy, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct Slottable;

#[derive(Debug, Clone, Component)]
pub struct SlotDeposit {
    pub slots: Vec<Entity>,
    pub attempting: VecDeque<Entity>,
}

impl SlotDeposit {
    pub fn new(slots: Vec<Entity>) -> Self {
        Self {
            slots,
            attempting: VecDeque::new(),
        }
    }

    pub fn contains(&self, entity: Entity) -> Option<usize> {
        self.attempting
            .iter()
            .enumerate()
            .find(|(index, entity)| entity == entity)
            .map(|(index, _)| index)
    }

    pub fn attempt(&mut self, entity: Entity) {
        if let None = self.contains(entity) {
            self.attempting.push_back(entity);
        }
    }

    pub fn stop_attempt(&mut self, entity: Entity) {
        if let Some(index) = self.contains(entity) {
            let removed = self.attempting.remove(index);
            assert_eq!(removed, Some(entity));
        }
    }

    pub fn pop_attempt(&mut self) -> Option<Entity> {
        self.attempting.pop_front()
    }
}

pub fn pending_slot(
    mut commands: Commands,
    name: Query<&Name>,
    mut slotters: Query<(Entity, &mut SlotDeposit)>,
    slottable: Query<(Entity, &Slottable)>,
    mut collision_events: EventReader<CollisionEvent>,
) {
    for collision_event in collision_events.iter() {
        let ((slotter_entity, mut slotter), (ingredient_entity, ingredient), colliding) =
            match collision_event {
                &CollisionEvent::Started(collider1, collider2, _flags) => {
                    let (slotter, potential) = if let Ok(slotter) = slotters.get_mut(collider1) {
                        (slotter, collider2)
                    } else if let Ok(slotter) = slotters.get_mut(collider2) {
                        (slotter, collider1)
                    } else {
                        continue;
                    };

                    if let Ok(ingredient) = slottable.get(potential) {
                        (slotter, ingredient, true)
                    } else {
                        continue;
                    }
                }
                &CollisionEvent::Stopped(collider1, collider2, _flags) => {
                    let (slotter, potential) = if let Ok(slotter) = slotters.get_mut(collider1) {
                        (slotter, collider2)
                    } else if let Ok(slotter) = slotters.get_mut(collider2) {
                        (slotter, collider1)
                    } else {
                        continue;
                    };

                    if let Ok(ingredient) = slottable.get(potential) {
                        (slotter, ingredient, false)
                    } else {
                        continue;
                    }
                }
            };

        if colliding {
            slotter.attempt(ingredient_entity);
        } else {
            slotter.stop_attempt(ingredient_entity);
        }
    }
}

pub fn insert_slot(mut slots: Query<&mut Slot>, mut deposits: Query<&mut SlotDeposit>) {
    for mut deposit in &mut deposits {
        if deposit.slots.len() == 0 {
            warn!("no slots specified in slot deposit");
        }

        let SlotDeposit {
            slots: deposit_slots,
            attempting,
        } = deposit.as_mut();

        for slot_entity in deposit_slots {
            if attempting.len() == 0 {
                break;
            }

            if let Ok(mut slot) = slots.get_mut(*slot_entity) {
                if slot.containing.is_none() {
                    slot.containing = attempting.pop_front();
                }
            }
        }
    }
}

/// Keep the item in the slot with spring forces.
///
/// This adds/removes the spring force to the item.
pub fn spring_slot(
    time: Res<Time>,
    mut items: Query<(
        &GlobalTransform,
        &Velocity,
        &ReadMassProperties,
        &mut ExternalImpulse,
        //&mut ExternalForce,
    )>,
    slots: Query<(&GlobalTransform, &Slot)>,
    names: Query<&Name>,
    mut lines: ResMut<DebugLines>,
) {
    let dt = time.delta_seconds();

    if dt == 0.0 {
        return;
    }

    for (slot_transform, slot) in &slots {
        if let Some(contained) = slot.containing {
            let strength: f32 = 0.5;
            let damp_ratio: f32 = 0.5;

            let (item_transform, item_velocity, mass_properties, mut impulse) = if let Ok(item) =
                items.get_mut(contained)
            {
                item
            } else {
                warn!("Contained entity {:?}, does not have an `ExternalImpulse` and/or `ReadMassProperties` component.", names.named(contained));
                continue;
            };

            let strength = strength.clamp(0.0, 1.0);
            let damp_ratio = damp_ratio.clamp(0.0, 1.0);
            let (mass, center) = (
                mass_properties.0.mass,
                mass_properties.0.local_center_of_mass,
            );

            if mass <= 0.0 || strength <= 0.0 {
                continue;
            }

            // should calculate the reduced mass between the 2 objects here.
            let t = crate::TICK_RATE.as_secs_f32();
            let kmax = mass / t;

            //let critical_damping = 2.0 * (mass * strength).sqrt();
            //let damp_coefficient = damp_ratio * critical_damping;

            let offset = item_transform.translation() - slot_transform.translation();
            let offset_impulse = -kmax * strength * offset;
            let vel = item_velocity.linvel + item_velocity.angvel.cross(Vec3::ZERO - center);

            let damp_impulse = -damp_ratio * vel;

            // don't let the damping force accelerate it
            //damp_force = damp_force.clamp_length_max(vel.length());

            let spring_impulse = offset_impulse + damp_impulse;

            impulse.impulse = spring_impulse;

            let lightness = (spring_impulse.length() / (strength * kmax)).clamp(0.0, 1.0);
            let color = Color::Hsla {
                hue: 0.0,
                saturation: 1.0,
                lightness: lightness,
                alpha: 0.7,
            };

            lines.line_colored(
                item_transform.translation(),
                item_transform.translation() + spring_impulse,
                crate::TICK_RATE.as_secs_f32(),
                color,
            );
        }
    }
}

pub struct SlotPlugin;
impl Plugin for SlotPlugin {
    fn build(&self, app: &mut App) {
        app.add_network_system(pending_slot);
        app.add_network_system(insert_slot);
        app.add_network_system(spring_slot);
    }
}
