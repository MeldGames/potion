use std::fmt::Debug;

use bevy::prelude::*;

use bevy_rapier3d::prelude::*;
use springy::RapierParticleQuery;

pub struct MusclePlugin;
impl Plugin for MusclePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Muscle>();
        app.add_systems(FixedUpdate, muscle_target);
    }
}

#[derive(Debug, Component, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct Muscle {
    pub target: Option<Entity>,
    pub strength: f32,
    pub tense: bool,
}

impl Muscle {
    pub fn new(target: Entity) -> Self {
        Self {
            target: Some(target),
            ..default()
        }
    }
}

impl Default for Muscle {
    fn default() -> Self {
        Self {
            target: None,
            strength: 0.3,
            tense: false,
        }
    }
}

pub fn muscle_target(
    targets: Query<(Entity, &Muscle)>,
    mut impulses: Query<Option<&mut ExternalImpulse>>,
    particles: Query<RapierParticleQuery>,
) {
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

        let displacement = particle_a
            .angular(Vec3::Y)
            .instant(&particle_b.angular(Vec3::Y))
            .displacement;
        let displacement_dir = displacement.normalize_or_zero();

        let mut angular_impulse = muscle.strength * displacement_dir;

        // Cap out the angular impulse to not be greater than the displacement so we don't constantly overshoot.
        // TODO: simplify this
        if (displacement.x > 0.0 && angular_impulse.x > displacement.x)
            || (displacement.x < 0.0 && angular_impulse.x < displacement.x)
        {
            angular_impulse.x = displacement.x;
        }

        if (displacement.y > 0.0 && angular_impulse.y > displacement.y)
            || (displacement.y < 0.0 && angular_impulse.y < displacement.y)
        {
            angular_impulse.y = displacement.y;
        }

        if (displacement.z > 0.0 && angular_impulse.z > displacement.z)
            || (displacement.z < 0.0 && angular_impulse.z < displacement.z)
        {
            angular_impulse.z = displacement.z;
        }

        if let Some(mut impulse_a) = impulse_a {
            impulse_a.torque_impulse += angular_impulse;
        }

        if let Some(mut impulse_b) = impulse_b {
            impulse_b.torque_impulse -= angular_impulse;
        }
    }
}
