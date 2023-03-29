use bevy::{prelude::*, window::PrimaryWindow};

use bevy_editor_pls::EditorState;
use bevy_mod_wanderlust::{ControllerInput, ControllerSettings, ControllerState};

use crate::attach::AttachPlugin;

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
            //.add_startup_system(bevy_mod_wanderlust::setup_physics_context)
            .add_system(bevy_mod_wanderlust::movement);
    }
}

#[derive(Bundle)]
pub struct PlayerBundle {
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub player_component: prelude::Player,
    pub name: Name,
}

pub fn window_focused(windows: Option<Query<&Window, With<PrimaryWindow>>>) -> bool {
    match windows.and_then(|windows| windows.get_single().ok().map(|window| window.focused)) {
        Some(focused) => focused,
        _ => false,
    }
}

pub fn editor_active(editor: Option<Res<EditorState>>) -> bool {
    if let Some(editor) = editor {
        editor.active
    } else {
        false
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ControllerSet;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(AttachPlugin);
        app.add_plugin(grab::GrabPlugin);
        app.register_type::<spawn::Player>();

        app.insert_resource(Events::<spawn::PlayerEvent>::default());

        //app.configure_set(ControllerSet.in_schedule(CoreSchedule::FixedUpdate));
        app.add_system(grab::player_grabby_hands.in_set(ControllerSet));
        app.add_system(grab::joint_children.in_set(ControllerSet));
        app.add_system(grab::tense_arms.in_set(ControllerSet));
        app.add_system(grab::grab_collider.in_set(ControllerSet));

        app.add_system(controller::player_movement.in_set(ControllerSet));
        app.add_system(controller::avoid_intersecting.in_set(ControllerSet));
        app.add_system(controller::character_crouch.in_set(ControllerSet));
        app.add_system(controller::controller_exclude.in_set(ControllerSet));
        app.add_system(controller::player_swivel_and_tilt.in_set(ControllerSet));
        app.add_system(controller::teleport_player_back.in_set(ControllerSet));

        app.add_system(spawn::connected_entities);
        app.add_system(spawn::contact_filter);
        app.add_system(spawn::connected_mass);
        app.add_system(spawn::extended_mass);
        app.add_system(spawn::setup_player);
        app.add_system(spawn::setup_ik);
        app.add_system(Events::<spawn::PlayerEvent>::update_system);
    }
}
