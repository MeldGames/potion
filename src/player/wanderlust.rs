use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use bevy_mod_wanderlust::{ControllerInput, WanderlustPlugin, *};

use crate::prelude::*;

pub struct CustomWanderlustPlugin;
impl Plugin for CustomWanderlustPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                get_mass_from_rapier,
                get_velocity_from_rapier,
                find_ground,
                determine_groundedness,
                gravity_force,
                movement_force,
                jump_force,
                upright_force,
                float_force,
                accumulate_forces,
                apply_forces,
                //apply_ground_forces,
                custom_apply_ground_forces,
            )
                .chain()
                .before(PhysicsSet::SyncBackend),
        );

        //app.add_systems(FixedUpdate, custom_apply_ground_forces.after(apply_forces));

        app.add_systems(Update, |casts: Query<(&GroundCast, &GroundForce, &JumpForce, &FloatForce)>, mut gizmos: Gizmos| {
            for (cast, opposing, jump, float) in &casts {
                if let Some((_, toi, _)) = cast.cast {
                    //gizmos.sphere(toi.witness, Quat::IDENTITY, 0.3, Color::LIME_GREEN);
                    //gizmos.ray_gradient(toi.witness, opposing.linear, Color::LIME_GREEN, Color::LIME_GREEN);
                    //gizmos.ray_gradient(toi.witness, -jump.linear, Color::RED, Color::RED);
                    //gizmos.ray_gradient(toi.witness, -float.linear, Color::BLUE, Color::BLUE);
                }
            }
        });
    }
}

pub fn custom_apply_ground_forces(
    mut grounds: Query<(&mut ExternalImpulse, Option<&ReadMassProperties>)>,
    ground_forces: Query<(Entity, &GroundForce, &GroundCast)>,
    ctx: Res<RapierContext>,

    grabbing: Query<&Grabbing>,
    children: Query<&Children>,
    joint_children: Query<&JointChildren>,
) {
    let dt = ctx.integration_parameters.dt;
    for (entity, force, cast) in &ground_forces {
        if let Some((ground, _, _)) = cast.cast {
            if let Ok((mut impulse, mass)) = grounds.get_mut(ground) {
                let mass = if let Some(mass) = mass {
                    mass.0.mass
                } else {
                    0.0
                };

                let is_grabbed =
                    find_children_with(&grabbing, &children, &joint_children, entity)
                        .iter()
                        .filter_map(|grabbing| grabbing.grabbed)
                        .any(|grabbed| grabbed.entity == ground);

                let is_small_object = mass < 2.0;
                let force_multiplier = if is_small_object && !is_grabbed {
                    0.01
                } else {
                    1.0
                };

                impulse.impulse += force.linear * force_multiplier * dt;
                impulse.torque_impulse += force.angular * force_multiplier * dt;
            }
        }
    }
}
