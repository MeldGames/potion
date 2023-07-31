use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use bevy_mod_wanderlust::*;

use crate::prelude::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WanderlustSet;

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
                float_force,
                upright_force,
                jump_force,
                accumulate_forces,
                apply_forces,
                //apply_ground_forces,
                custom_apply_ground_forces,
            )
                .chain()
                .in_set(WanderlustSet)
                .before(PhysicsSet::SyncBackend),
        );

        app.add_systems(
            Update,
            |casts: Query<(&GroundCast, &GroundForce, &JumpForce, &FloatForce)>,
             mut gizmos: Gizmos,
             mut config: ResMut<GizmoConfig>| {
                config.depth_bias = -0.01;
                for (cast, opposing, _jump, _float) in &casts {
                    if let GroundCast::Touching(ground) = cast {
                        /*gizmos.ray(
                            ground.cast.witness,
                            Vec3::Y,
                            Color::LIME_GREEN,
                        );*/
                        gizmos.ray(
                            ground.cast.witness,
                            opposing.linear * crate::TICK_RATE.as_secs_f32(),
                            Color::LIME_GREEN,
                        );
                        //gizmos.ray_gradient(toi.witness, -jump.linear, Color::RED, Color::RED);
                        //gizmos.ray_gradient(toi.witness, -float.linear, Color::BLUE, Color::BLUE);
                    }
                }
            },
        );
    }
}

pub fn custom_apply_ground_forces(
    mut grounds: Query<(&mut ExternalImpulse, Option<&ReadMassProperties>)>,
    ground_forces: Query<(Entity, &GroundForce, &GroundCast)>,
    ctx: Res<RapierContext>,
    /*
    grabbing: Query<&Grabbing>,
    children: Query<&Children>,
    joint_children: Query<&JointChildren>,
    */
) {
    let dt = ctx.integration_parameters.dt;
    for (_entity, force, cast) in &ground_forces {
        if let GroundCast::Touching(ground) = cast {
            if let Ok((mut impulse, _mass)) = grounds.get_mut(ground.entity) {
                /*
                let mass = if let Some(mass) = mass {
                    mass.0.mass
                } else {
                    0.0
                };

                let is_grabbed = find_children_with(&grabbing, &children, &joint_children, entity)
                    .iter()
                    .filter_map(|grabbing| grabbing.grabbed)
                    .any(|grabbed| grabbed.entity == ground.entity);

                let is_small_object = mass < 2.0;
                let force_multiplier = if is_small_object && !is_grabbed {
                    0.01
                } else {
                    1.0
                };
                */

                impulse.impulse += force.linear * dt;
                impulse.torque_impulse += force.angular * dt;
            }
        }
    }
}
