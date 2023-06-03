use std::fmt::Debug;

use bevy::prelude::*;

use bevy_rapier3d::prelude::*;
use springy::{Particle, RapierParticleQuery, Spring};

pub struct MusclePlugin;
impl Plugin for MusclePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Muscle>();
        app.add_system(muscle_target);
    }
}

#[derive(Default, Debug, Component, Clone, Copy, Reflect, FromReflect)]
#[reflect(Component)]
pub struct Muscle {
    pub target: Option<Entity>,
    pub spring: Spring,
    pub tense: bool,
}

impl Muscle {
    pub fn new(target: Entity) -> Self {
        Self {
            target: Some(target),
            spring: Spring::default(),
            tense: true,
        }
    }
}

pub fn muscle_target(
    ctx: Res<RapierContext>,
    globals: Query<&GlobalTransform>,
    mut targets: Query<(Entity, &Muscle, &mut ExternalImpulse, RapierParticleQuery)>,
) {
    let dt = ctx.integration_parameters.dt;

    for (current_entity, muscle, mut impulse, particle) in &mut targets {
        if !muscle.tense {
            continue;
        }

        let target = if let Some(target) = muscle.target {
            target
        } else {
            continue;
        };

        let [target_global, current_global] =
            if let Ok(globals) = globals.get_many([target, current_entity]) {
                globals
            } else {
                continue;
            };

        let current = current_global.compute_transform().up();
        let target = target_global.compute_transform().up();

        // Not normalizing this doubles as a strength of the difference
        let difference = current.cross(target);

        let particle_b = Particle::default();

        /*
               let mut torque = (target_axis * muscle.spring.strength)
                   - (local_angular_velocity * spring.damp_coefficient(mass));
               torque = torque.clamp_length_max(spring.strength) * dt;
               impulse.torque_impulse += torque;
        */
    }
}
