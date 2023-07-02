use bevy::prelude::*;

use bevy_mod_wanderlust::{ControllerInput, ControllerSettings, ControllerState};

use crate::{
    attach::AttachPlugin,
    player::{inventory::InventoryPlugin, prelude::GrabJoint},
    FixedSet,
};

use self::prelude::{CharacterEntities, ConnectedEntities};

pub mod controller;
pub mod grab;
pub mod input;
pub mod inventory;
pub mod spawn;

pub mod prelude {
    pub use super::{controller::*, grab::*, input::*, spawn::*};
    pub use super::{CustomWanderlustPlugin, PlayerBundle, PlayerPlugin};
}

pub struct CustomWanderlustPlugin;

impl Plugin for CustomWanderlustPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ControllerState>()
            .register_type::<ControllerSettings>()
            .register_type::<ControllerInput>()
            //.add_startup_system(bevy_mod_wanderlust::setup_physics_context)
            .add_system(bevy_mod_wanderlust::movement.in_schedule(CoreSchedule::FixedUpdate));
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
        app.add_plugin(AttachPlugin);
        app.add_plugin(InventoryPlugin);
        app.add_plugin(grab::GrabPlugin);
        app.add_plugin(controller::ControllerPlugin);
        app.add_plugin(spawn::PlayerSpawnPlugin);

        /*
               app.register_type::<spawn::Player>();
               app.insert_resource(Events::<spawn::PlayerEvent>::default());

               app.add_system(spawn::related_entities::<CharacterEntities, Without<GrabJoint>>);
               app.add_system(spawn::related_entities::<ConnectedEntities, ()>);
               app.add_system(spawn::contact_filter);
               app.add_system(spawn::setup_player);
               app.add_system(spawn::setup_ik);
               app.add_system(Events::<spawn::PlayerEvent>::update_system);
        */
    }
}
