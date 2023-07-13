use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use bevy_mod_wanderlust::{ControllerInput, WanderlustPlugin, *};

pub mod controller;
pub mod grab;
pub mod input;
pub mod inventory;
pub mod spawn;
pub mod wanderlust;

pub mod prelude {
    pub use super::{controller::*, grab::*, input::*, inventory::prelude::*, spawn::*, wanderlust::*};
    pub use super::{PlayerBundle, PlayerPlugin};
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
        app.add_plugins(wanderlust::CustomWanderlustPlugin);

        app.add_plugins(inventory::InventoryPlugin);
        app.add_plugins(grab::GrabPlugin);
        app.add_plugins(controller::ControllerPlugin);
        app.add_plugins(spawn::PlayerSpawnPlugin);
    }
}
