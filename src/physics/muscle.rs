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
    //pub spring: Spring,
    pub strength: f32,
    pub tense: bool,
}

impl Muscle {
    pub fn new(target: Entity) -> Self {
        Self {
            target: Some(target),
            /*
            spring: Spring {
                strength: 0.75,
                damp_ratio: 0.05,
            },
            */
            strength: 0.3,
            tense: true,
        }
    }
}

pub fn muscle_target(
    ctx: Res<RapierContext>,
    targets: Query<(Entity, &Muscle)>,
    mut impulses: Query<Option<&mut ExternalImpulse>>,
    particles: Query<RapierParticleQuery>,

    names: Query<&Name>,
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
            .instant(&particle_b.angular(Vec3::Y));

        let displacement = angular_instant.displacement;
        let displacement_dir = displacement.normalize_or_zero();

        let falloff = 0.2;
        let strength = if displacement.length() < falloff {
            let t = displacement.length() / falloff;

            let falloff_percent = 1.0 - ((-t * 0.9 + 1.0).log10() + 1.0);
            muscle.strength * falloff_percent
        } else {
            muscle.strength
        };
        let angular_impulse = strength * displacement_dir;

        if let Some(mut impulse_a) = impulse_a {
            impulse_a.torque_impulse += angular_impulse;
        }

        if let Some(mut impulse_b) = impulse_b {
            impulse_b.torque_impulse -= angular_impulse;
        }
    }
}
