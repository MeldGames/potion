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

#[derive(Default, Debug, Copy, Clone, Component, Reflect, Inspectable)]
#[reflect(Component)]
pub struct Slot {
    /// Entity this slot contains.
    #[inspectable(read_only)]
    pub containing: Option<Entity>,
}

#[derive(Default, Debug, Copy, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct SlotSettings(pub springy::Spring);

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
    particles: Query<springy::RapierParticleQuery>,
    mut impulses: Query<Option<&mut ExternalImpulse>>,
    slots: Query<(Entity, &Slot, &SlotSettings)>,
    names: Query<&Name>,
    mut lines: ResMut<DebugLines>,
) {
    if time.delta_seconds() == 0.0 {
        return;
    }

    let timestep = crate::TICK_RATE.as_secs_f32();
    let inverse_timestep = 1.0 / timestep;

    for (slot_entity, slot, slot_settings) in &slots {
        if let Some(particle_entity) = slot.containing {
            if particle_entity == slot_entity {
                continue;
            }

            let [particle_a, particle_b] =
                if let Ok(particles) = particles.get_many([slot_entity, particle_entity]) {
                    particles
                } else {
                    warn!("Particle does not contain all necessary components");
                    continue;
                };

            let impulse = slot_settings.0.impulse(timestep, particle_a, particle_b);

            let [slot_impulse, particle_impulse] =
                if let Ok(impulses) = impulses.get_many_mut([slot_entity, particle_entity]) {
                    impulses
                } else {
                    warn!("Particle does not contain all necessary components");
                    continue;
                };

            if let Some(mut slot_impulse) = slot_impulse {
                slot_impulse.impulse = -impulse;
            }

            if let Some(mut particle_impulse) = particle_impulse {
                particle_impulse.impulse = impulse;
            }
        }
    }
}

pub struct SlotPlugin;
impl Plugin for SlotPlugin {
    fn build(&self, app: &mut App) {
        //app.register_type::<Slot>();
        //app.register_inspectable::<Slot>();

        app.add_network_system(pending_slot);
        app.add_network_system(insert_slot);
        app.add_network_system(spring_slot);
    }
}
