use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use bevy_mod_wanderlust::{ControllerInput, WanderlustPlugin, *};

pub mod controller;
pub mod grab;
pub mod input;
pub mod inventory;
pub mod spawn;

pub mod prelude {
    pub use super::{controller::*, grab::*, input::*, inventory::prelude::*, spawn::*};
    pub use super::{CustomWanderlustPlugin, PlayerBundle, PlayerPlugin};
}

pub struct CustomWanderlustPlugin;
impl Plugin for CustomWanderlustPlugin {
    fn build(&self, app: &mut App) {
        //app.add_plugins(WanderlustPlugin::default());

        //.add_startup_system(bevy_mod_wanderlust::setup_physics_context)
        //app.add_systems(FixedUpdate, bevy_mod_wanderlust::movement);

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
                apply_ground_forces,
            )
                .chain()
                .before(PhysicsSet::SyncBackend),
        );

        app.add_systems(
            Update,
            |casts: Query<&GroundCast>, mut gizmos: Gizmos| {
                for cast in &casts {
                    if let Some((entity, toi, velocity)) = cast.cast {
                        gizmos.sphere(toi.witness, Quat::IDENTITY, 0.3, Color::LIME_GREEN);
                    }
                }
            }
        );
    }
}

#[derive(Bundle)]
pub struct PlayerBundle {
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub player_component: prelude::Player,
    pub name: Name,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ControllerSet;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(CustomWanderlustPlugin);

        app.add_plugin(inventory::InventoryPlugin);
        app.add_plugin(grab::GrabPlugin);
        app.add_plugin(controller::ControllerPlugin);
        app.add_plugin(spawn::PlayerSpawnPlugin);
    }
}
