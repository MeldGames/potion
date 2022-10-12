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

#[derive(Default, Debug, Copy, Clone, Component, Reflect, Inspectable)]
#[reflect(Component)]
pub struct SlotSettings {
    /// Strength of the spring-like force of the slot. Ranged between 0..1
    #[inspectable(min = 0.0, max = 1.0)]
    pub strength: f32,
    /// Damping of the spring-like force of the slot. Ranged between 0..1
    #[inspectable(min = 0.0, max = 1.0)]
    pub damping: f32,

    pub rest_distance: f32,
    pub limp_distance: f32,
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

#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct ParticleQuery<'a> {
    pub mass: Option<&'a ReadMassProperties>,
    pub rigid_body: Option<&'a RigidBody>,
    pub global_transform: &'a GlobalTransform,
    pub velocity: &'a Velocity,
    pub impulse: Option<&'a mut ExternalImpulse>,
}

impl<'w, 's> ParticleQueryItem<'w, 's> {
    pub fn mass(&self) -> f32 {
        match self.mass {
            Some(mass_properties) => mass_properties.0.mass,
            None => {
                if let Some(RigidBody::Dynamic) = self.rigid_body {
                    1.0
                } else {
                    f32::INFINITY
                }
            }
        }
    }

    pub fn inverse_mass(&self) -> f32 {
        let mass = self.mass();
        if mass.is_infinite() {
            0.0
        } else {
            1.0 / mass
        }
    }

    pub fn apply_impulse(&mut self, impulse: Vec3) {
        if let Some(physics_impulse) = self.impulse.as_mut() {
            physics_impulse.impulse = impulse;
        }
    }
}

/// Keep the item in the slot with spring forces.
///
/// This adds/removes the spring force to the item.
pub fn spring_slot(
    time: Res<Time>,
    mut particle: Query<ParticleQuery>,
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

            let [mut particle_a, mut particle_b] =
                if let Ok(particles) = particle.get_many_mut([slot_entity, particle_entity]) {
                    particles
                } else {
                    warn!("Particle does not contain all necessary components");
                    continue;
                };

            let strength = slot_settings.strength;
            let damping = slot_settings.damping;
            let rest_distance = slot_settings.rest_distance;

            let distance = particle_b.global_transform.translation()
                - particle_a.global_transform.translation();
            let velocity = particle_b.velocity.linvel - particle_a.velocity.linvel;

            let unit_vector = distance.normalize_or_zero();

            let distance_error = unit_vector
                * if slot_settings.rest_distance > distance.length() {
                    0.0
                } else {
                    distance.length() - slot_settings.rest_distance
                };
            let velocity_error = velocity;

            let reduced_mass = 1.0 / (particle_a.inverse_mass() + particle_b.inverse_mass());
            let strength_max = reduced_mass / timestep;
            let damping_max = reduced_mass;

            let distance_impulse = strength * distance_error * inverse_timestep * reduced_mass;
            let velocity_impulse = damping * velocity_error * reduced_mass;

            let impulse = -(distance_impulse + velocity_impulse);

            particle_a.apply_impulse(-impulse);
            particle_b.apply_impulse(impulse);
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
