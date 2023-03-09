use std::{collections::VecDeque, time::Duration};

use bevy::{ecs::entity::Entities, prelude::*};
use bevy_prototype_debug_lines::DebugLines;
use bevy_rapier3d::prelude::*;
use sabi::{prelude::ReplicatePlugin, stage::NetworkSimulationAppExt};

use crate::cauldron::NamedEntity;

#[derive(Default, Debug, Copy, Clone, Component, Reflect, FromReflect)]
#[reflect(Component)]
pub struct Slot {
    /// Entity this slot contains.
    #[reflect(default)]
    pub containing: Option<Entity>,
}

#[derive(Debug, Clone, Bundle)]
pub struct SlotBundle {
    pub slot: Slot,
    pub settings: SlotSettings,
    pub grace: SlotGracePeriod,
}

#[derive(Default, Debug, Clone, Component, Reflect, FromReflect)]
#[reflect(Component)]
pub struct SlotSettings(pub springy::SpringState<Vec3>);

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Component, Reflect, FromReflect)]
#[reflect(Component)]
pub enum Slottable {
    #[default]
    Free,

    Slotted,
}

#[derive(Debug, Clone, Component, Reflect, FromReflect)]
#[reflect(Component)]
pub struct SlotGracePeriod(Timer);

impl Default for SlotGracePeriod {
    fn default() -> Self {
        Self(Timer::new(Duration::from_secs(1), TimerMode::Once))
    }
}

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

    pub fn contains(&self, _entity: Entity) -> Option<usize> {
        self.attempting
            .iter()
            .enumerate()
            .find(|(_index, entity)| entity == entity)
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
    names: Query<&Name>,
    mut slotters: Query<(Entity, &mut SlotDeposit)>,
    slottable: Query<(Entity, &Slottable)>,
    mut collision_events: EventReader<CollisionEvent>,
) {
    for collision_event in collision_events.iter() {
        let ((_slotter_entity, mut slotter), (ingredient_entity, slottable), colliding) =
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

        if *slottable == Slottable::Slotted {
            continue;
        }

        if colliding {
            slotter.attempt(ingredient_entity);
        } else {
            slotter.stop_attempt(ingredient_entity);
        }

        info!(
            "attempting: {:?}",
            slotter
                .attempting
                .iter()
                .map(|entity| names.named(*entity))
                .collect::<Vec<_>>()
        );
    }
}

pub fn tick_grace_period(mut slots: Query<&mut SlotGracePeriod>) {
    for mut period in &mut slots {
        period.0.tick(crate::TICK_RATE);
    }
}

pub fn insert_slot(
    mut slotted: Query<&mut Slottable>,
    mut slots: Query<(&mut Slot, &mut SlotGracePeriod)>,
    mut deposits: Query<&mut SlotDeposit>,
    names: Query<&Name>,
) {
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

            if let Ok((mut slot, mut grace_period)) = slots.get_mut(*slot_entity) {
                if slot.containing.is_none() {
                    while let Some(next_item) = attempting.pop_front() {
                        if let Ok(mut slottable) = slotted.get_mut(next_item) {
                            if *slottable == Slottable::Free {
                                info!("slotting {:?}", names.named(next_item));
                                slot.containing = Some(next_item);
                                grace_period.0 = Timer::new(Duration::from_secs(1), TimerMode::Once);
                                *slottable = Slottable::Slotted;
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Keep the item in the slot with spring forces.
///
/// This adds/removes the spring force to the item.
pub fn spring_slot(
    entities: &Entities,
    particles: Query<springy::RapierParticleQuery>,
    mut impulses: Query<Option<&mut ExternalImpulse>>,
    mut slottables: Query<&mut Slottable>,
    mut slots: Query<(Entity, &mut Slot, &mut SlotSettings, &SlotGracePeriod)>,
    names: Query<&Name>,
    mut lines: ResMut<DebugLines>,
) {
    let timestep = crate::TICK_RATE.as_secs_f32();

    for (slot_entity, mut slot, mut slot_settings, grace_period) in &mut slots {
        if let Some(particle_entity) = slot.containing {
            if !entities.contains(particle_entity) {
                warn!(
                    "contained entity is no longer alive: {:?}",
                    names.named(particle_entity)
                );
                slot.containing = None;
                continue;
            }

            if particle_entity == slot_entity {
                warn!("Slot cannot contain itself: {:?}", names.named(slot_entity));
                slot.containing = None;
                continue;
            }

            let [particle_a, particle_b] =
                if let Ok(particles) = particles.get_many([slot_entity, particle_entity]) {
                    particles
                } else {
                    warn!("Particle does not contain all necessary components");
                    continue;
                };

            let translation_a = particle_a.global_transform.translation();
            let translation_b = particle_b.global_transform.translation();

            let rigid_body_a = particle_a.rigid_body.cloned();
            let rigid_body_b = particle_a.rigid_body.cloned();

            let impulse = match slot_settings.0.impulse(timestep, particle_a, particle_b) {
                springy::SpringResult::Impulse(impulse) => impulse,
                springy::SpringResult::Broke(impulse) => {
                    if grace_period.0.finished() {
                        if let Ok(mut slottable) = slottables.get_mut(particle_entity) {
                            *slottable = Slottable::Free;
                        }

                            info!(
                                "Removing from slot {:?}: {:?}",
                                names.named(slot_entity),
                                names.named(particle_entity)
                            );
                            slot.containing = None;
                            continue;
                    } else {
                        impulse
                    }
                }
            };

            let [slot_impulse, particle_impulse] =
                impulses.many_mut([slot_entity, particle_entity]);

            let impulse_error = |entity: Entity, rigid_body: Option<RigidBody>| {
                if let Some(RigidBody::Dynamic) = rigid_body {
                    warn!(
                        "Particle {:?} does not have an `ExternalImpulse` component",
                        names.named(entity)
                    );
                }
            };

            if let Some(mut slot_impulse) = slot_impulse {
                slot_impulse.impulse = -impulse;

                lines.line_colored(
                    translation_a,
                    translation_a - impulse,
                    crate::TICK_RATE.as_secs_f32(),
                    Color::BLUE,
                );
            } else {
                impulse_error(slot_entity, rigid_body_a);
            }

            if let Some(mut particle_impulse) = particle_impulse {
                particle_impulse.impulse = impulse;

                lines.line_colored(
                    translation_b,
                    translation_b + impulse,
                    crate::TICK_RATE.as_secs_f32(),
                    Color::RED,
                );
            } else {
                impulse_error(particle_entity, rigid_body_b);
            }
        }
    }
}

pub struct SlotPlugin;
impl Plugin for SlotPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Slot>()
            //.register_type::<VecDeque<Entity>>()
            .register_type::<springy::SpringState<Vec3>>()
            .register_type::<Option<springy::SpringBreak>>()
            .register_type::<Option<Entity>>()
            .register_type::<springy::Spring>()
            .register_type::<springy::SpringBreak>()
            .register_type::<bevy::time::TimerMode>()
            .register_type::<SlotSettings>();

        app.add_plugin(ReplicatePlugin::<Slot>::default());
        app.add_plugin(ReplicatePlugin::<SlotSettings>::default());
        app.add_plugin(ReplicatePlugin::<Slottable>::default());
        app.add_plugin(ReplicatePlugin::<SlotGracePeriod>::default());
        //app.add_plugin(ReplicatePlugin::<SlotDeposit>::default());

        app.add_network_system(pending_slot);
        app.add_network_system(insert_slot.after(pending_slot));
        app.add_network_system(tick_grace_period.before(insert_slot));
        app.add_network_system(spring_slot.after(insert_slot));
    }
}
