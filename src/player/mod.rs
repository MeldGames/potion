use std::fmt::Debug;

use bevy::input::mouse::MouseWheel;
use bevy::utils::HashSet;
use bevy::{input::mouse::MouseMotion, prelude::*};
use bevy_prototype_debug_lines::DebugLines;
use std::f32::consts::PI;

use bevy_inspector_egui::{Inspectable, RegisterInspectable};
use bevy_mod_wanderlust::{
    CharacterControllerBundle, CharacterControllerPreset, ControllerInput, ControllerPhysicsBundle,
    ControllerSettings, ControllerState, Spring,
};
use bevy_rapier3d::prelude::*;
use bevy_rapier3d::rapier::prelude::{JointAxis, MotorModel};
use bevy_renet::renet::RenetServer;
use sabi::prelude::*;

use sabi::stage::{NetworkCoreStage, NetworkSimulationAppExt};

use serde::{Deserialize, Serialize};

use iyes_loopless::{condition::IntoConditionalSystem, prelude::*};

use crate::attach::{Attach, AttachPlugin};
use crate::physics::{GRAB_GROUPING, REST_GROUPING};

pub mod controller;
pub mod grab;
pub mod input;
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
            .add_startup_system(bevy_mod_wanderlust::setup_physics_context)
            .add_network_system(bevy_mod_wanderlust::movement);
    }
}

#[derive(Bundle)]
pub struct PlayerBundle {
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub player_component: prelude::Player,
    pub name: Name,
}

pub fn window_focused(windows: Option<Res<Windows>>) -> bool {
    if let Some(windows) = windows {
        if let Some(window) = windows.get_primary() {
            return window.is_focused();
        }
    }

    false
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(AttachPlugin);
        app.register_type::<spawn::Player>();

        app.insert_resource(Events::<spawn::PlayerEvent>::default());

        app.add_network_system(
            controller::player_movement
                .label("player_movement")
                .before(bevy_mod_wanderlust::movement)
                .after("update_player_inputs")
                .after("player_swivel_and_tilt"),
        );
        app.add_network_system(
            grab::player_grabby_hands
                .label("player_grabby_hands")
                .after(bevy_mod_wanderlust::movement)
                .after("update_player_inputs")
                .after("player_movement"),
        );
        app.add_system_to_network_stage(
            NetworkCoreStage::PostUpdate,
            controller::avoid_intersecting.label("avoid_intersecting"),
        );
        app.add_network_system(
            controller::character_crouch
                .label("character_crouch")
                .before(bevy_mod_wanderlust::movement)
                .after("update_player_inputs"),
        );
        app.add_network_system(
            controller::pull_up
                .label("pull_up")
                .before(bevy_mod_wanderlust::movement)
                .after("update_player_inputs")
                .after("player_swivel_and_tilt")
                .after("player_movement"),
        );
        app.add_network_system(grab::joint_children.label("joint_children"));
        app.add_network_system(
            spawn::connected_entities
                .label("connected_entities")
                .after("joint_children")
                .before("related_entities"),
        );
        app.add_network_system(
            controller::controller_exclude
                .label("controller_exclude")
                .after("joint_children"),
        );
        app.add_network_system(
            grab::grab_collider
                .label("grab_collider")
                .after(bevy_mod_wanderlust::movement)
                .after("target_position")
                .after("related_entities"),
        );
        app.add_network_system(
            controller::player_swivel_and_tilt
                .label("player_swivel_and_tilt")
                .after("update_player_inputs"),
        );
        app.add_meta_network_system(spawn::setup_player);
        app.add_meta_network_system(Events::<spawn::PlayerEvent>::update_system);

        app.add_network_system(controller::teleport_player_back);
    }
}
