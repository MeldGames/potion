use std::fmt::Debug;

use bevy::prelude::*;

use bevy_rapier3d::prelude::*;
use springy::{RapierParticleQuery, Spring};

pub struct MusclePlugin;
impl Plugin for MusclePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Muscle>();
        app.add_system(muscle_target.in_schedule(CoreSchedule::FixedUpdate));
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
            spring: Spring {
                strength: 0.75,
                damp_ratio: 0.05,
            },
            tense: true,
        }
    }
}

pub fn muscle_target(
    ctx: Res<RapierContext>,
    targets: Query<(Entity, &Muscle)>,
    mut impulses: Query<Option<&mut ExternalImpulse>>,
    particles: Query<RapierParticleQuery>,
) {
    let dt = ctx.integration_parameters.dt;

    for (current_entity, muscle) in &targets {
        if !muscle.tense {
            continue;
        }

        let target = if let Some(target) = muscle.target {
            target
        } else {
            continue;
        };

        let [particle_a, particle_b] =
            if let Ok(particles) = particles.get_many([target, current_entity]) {
                particles
            } else {
                continue;
            };

        let [impulse_a, impulse_b] =
            if let Ok(impulses) = impulses.get_many_mut([target, current_entity]) {
                impulses
            } else {
                continue;
            };

        let angular_instant = particle_a
            .angular(Vec3::Y)
            .instant(&particle_b.angular(-Vec3::Y));
        let angular_impulse = muscle.spring.impulse(dt, angular_instant);

        if let Some(mut impulse_a) = impulse_a {
            impulse_a.torque_impulse += angular_impulse;
        }

        if let Some(mut impulse_b) = impulse_b {
            impulse_b.torque_impulse -= angular_impulse;
        }
    }
}
