use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use bevy_mod_wanderlust::{ControllerInput, WanderlustPlugin, *};


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
                apply_gravity,
                movement_force,
                jump_force,
                upright_force,
                float_force,
                accumulate_forces,
                apply_forces,
                //apply_ground_forces,
                //custom_apply_ground_forces,
            )
                .chain()
                .before(PhysicsSet::SyncBackend),
        );

        app.add_systems(FixedUpdate, custom_apply_ground_forces.after(apply_forces));

        app.add_systems(Update, |casts: Query<&GroundCast>, mut gizmos: Gizmos| {
            for cast in &casts {
                if let Some((_, toi, _)) = cast.cast {
                    gizmos.sphere(toi.witness, Quat::IDENTITY, 0.3, Color::LIME_GREEN);
                }
            }
        });
    }
}


pub fn custom_apply_ground_forces(
    mut grounds: Query<(&mut ExternalImpulse, Option<&ReadMassProperties>)>,
    ground_forces: Query<(&GroundForce, &GroundCast)>,
    ctx: Res<RapierContext>,
) {
    let dt = ctx.integration_parameters.dt;
    for (force, cast) in &ground_forces {
        if let Some((ground, _, _)) = cast.cast {
            if let Ok((mut impulse, mass)) = grounds.get_mut(ground) {
                let mass = if let Some(mass) = mass {
                    mass.0.mass
                } else {
                    0.0
                };

                info!("mass: {:?}", mass);

                if mass >= 2.0 {
                    impulse.impulse += force.linear * dt;
                    impulse.torque_impulse += force.angular * dt;
                }
            }
        }
    }
}
