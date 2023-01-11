use std::fmt::Debug;

use bevy::ecs::entity::Entities;

use sabi::stage::NetworkSimulationAppExt;

use bevy::prelude::*;
use bevy::utils::HashSet;
use bevy_prototype_debug_lines::DebugLines;

use bevy_mod_wanderlust::Spring;
use bevy_rapier3d::prelude::*;
use bevy_rapier3d::rapier::prelude::{JointAxis, MotorModel};

use crate::physics::{GRAB_GROUPING, REST_GROUPING};
use crate::cauldron::NamedEntity;

pub struct MusclePlugin;
impl Plugin for MusclePlugin {
    fn build(&self, app: &mut App) {
        app.add_network_system(muscle_target);
    }
}

#[derive(Debug, Component, Clone, Copy)]
pub struct MuscleTarget(pub Entity);

pub fn muscle_target(
    ctx: Res<RapierContext>,
    globals: Query<&GlobalTransform>,
    mut targets: Query<(Entity, &MuscleTarget, &mut ExternalImpulse)>,
) {
    let dt = ctx.integration_parameters.dt;

    for (current_entity, target_rotation, mut impulse) in &mut targets {
        let [target_global, current_global] =
            if let Ok(globals) = globals.get_many([target_rotation.0, current_entity]) {
                globals
            } else {
                continue;
            };

        let current_transform = current_global.compute_transform();
        let target_transform = target_global.compute_transform();
        let current_dir = current_transform.rotation * -Vec3::Y;
        let target_dir = target_transform.rotation * -Vec3::Y;

        // Not normalizing this doubles as a strength of the difference
        // if we normalize we tend to get jitters so uh... don't do that
        let target_axis = current_dir.normalize().cross(target_dir.normalize());

        //let local_angular_velocity = hand_velocity.angvel - arm_velocity.angvel;
        //let local_angular_velocity = hand_velocity.angvel;

        //let mass = mass.0.mass;
        let muscle = Spring {
            strength: 30.0,
            damping: 0.3,
        };

        let wrist_force = (target_axis * muscle.strength);
        //- (local_angular_velocity * wrist_spring.damp_coefficient(hand_mass));
        let torque = wrist_force.clamp_length_max(muscle.strength) * dt;
        impulse.torque_impulse += torque;
    }
}
