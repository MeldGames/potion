use std::fmt::Debug;

use bevy::prelude::*;

use bevy_mod_wanderlust::Spring;
use bevy_rapier3d::prelude::*;

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
    pub strength: f32,
    pub tense: bool,
}

impl Muscle {
    pub fn new(target: Entity) -> Self {
        Self {
            target: Some(target),
            strength: 100.0,
            tense: true,
        }
    }
}

pub fn muscle_target(
    ctx: Res<RapierContext>,
    globals: Query<&GlobalTransform>,
    mut targets: Query<(
        Entity,
        &Muscle,
        &mut ExternalImpulse,
        &Velocity,
        &ReadMassProperties,
    )>,
) {
    let dt = ctx.integration_parameters.dt;

    for (current_entity, muscle, mut impulse, velocity, mass_properties) in &mut targets {
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

        let current_transform = current_global.compute_transform();
        let target_transform = target_global.compute_transform();
        let current_dir = current_transform.rotation * -Vec3::Y;
        let target_dir = target_transform.rotation * -Vec3::Y;

        // Not normalizing this doubles as a strength of the difference
        let target_axis = current_dir.normalize().cross(target_dir.normalize());

        let local_angular_velocity = velocity.angvel;

        let mass = mass_properties.0.mass;
        let spring = Spring {
            strength: muscle.strength,
            damping: 0.2,
        };

        let mut torque = (target_axis * spring.strength)
            - (local_angular_velocity * spring.damp_coefficient(mass));
        torque = torque.clamp_length_max(spring.strength) * dt;
        impulse.torque_impulse += torque;
    }
}
