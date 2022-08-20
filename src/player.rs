use std::fmt::Debug;

use bevy::{input::mouse::MouseMotion, prelude::*};

use bevy_egui::EguiContext;
use bevy_inspector_egui::{Inspectable, RegisterInspectable};
use bevy_rapier3d::prelude::*;
use sabi::{prelude::*, Replicate};

use sabi::stage::NetworkSimulationAppExt;

use serde::{Deserialize, Serialize};

use iyes_loopless::{condition::IntoConditionalSystem, prelude::*};

#[derive(Debug, Component)]
pub struct Player {
    pub id: u64,
}

#[derive(Debug)]
pub struct MouseSensitivity(f32);

impl Default for MouseSensitivity {
    fn default() -> Self {
        Self(1.0)
    }
}

#[derive(Component, Debug)]
pub struct LocalPlayer;

bitflags::bitflags! {
    #[derive(Default, Serialize, Deserialize)]
    pub struct PlayerInputSet: u16 {
        const FORWARD = 1 << 1;
        const BACK = 1 << 2;
        const LEFT = 1 << 3;
        const RIGHT = 1 << 4;
    }
}

impl PlayerInputSet {
    pub fn shorthand_display(self) -> String {
        let mut keys = "".to_owned();

        keys += match self.contains(Self::LEFT) {
            true => "<",
            false => "-",
        };

        keys += match self.contains(Self::FORWARD) {
            true => "^",
            false => "-",
        };

        keys += match self.contains(Self::BACK) {
            true => "v",
            false => "-",
        };

        keys += match self.contains(Self::RIGHT) {
            true => ">",
            false => "-",
        };

        keys
    }
}

#[derive(Clone, Copy, Default, Component, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Radians(f32);

impl Debug for Radians {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{value:.precision$?}°",
            precision = f.precision().unwrap_or(2),
            value = self.0 * 360.0 / std::f32::consts::TAU,
        ))
    }
}

#[derive(Clone, Copy, Default, Component, Serialize, Deserialize)]
pub struct PlayerInput {
    /// Movement inputs
    pub binary_inputs: PlayerInputSet,
    /// Vertical rotation of camera
    pub pitch: f32,
    /// Horizontal rotation of camera
    pub yaw: f32,
    //pub casted: [Option<CastInput>; 4],
}

impl Debug for PlayerInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlayerInput")
            .field("keys", &self.binary_inputs.shorthand_display())
            .field("pitch", &Radians(self.pitch))
            .field("yaw", &Radians(self.yaw))
            .finish()
    }
}

#[derive(Debug, Clone, Copy, Default, Component, Serialize, Deserialize)]
pub struct CastInput {}

impl PlayerInput {
    pub fn new() -> Self {
        Self {
            binary_inputs: PlayerInputSet::empty(),
            pitch: 0.0,
            yaw: 0.0,
            //casted: [None; 4],
        }
    }

    pub fn set_forward(&mut self, forward: bool) {
        self.binary_inputs.set(PlayerInputSet::FORWARD, forward);
    }

    pub fn set_back(&mut self, back: bool) {
        self.binary_inputs.set(PlayerInputSet::BACK, back);
    }

    pub fn set_left(&mut self, left: bool) {
        self.binary_inputs.set(PlayerInputSet::LEFT, left);
    }

    pub fn set_right(&mut self, right: bool) {
        self.binary_inputs.set(PlayerInputSet::RIGHT, right);
    }

    pub fn forward(&self) -> bool {
        self.binary_inputs.contains(PlayerInputSet::FORWARD)
    }

    pub fn back(&self) -> bool {
        self.binary_inputs.contains(PlayerInputSet::BACK)
    }

    pub fn left(&self) -> bool {
        self.binary_inputs.contains(PlayerInputSet::LEFT)
    }

    pub fn right(&self) -> bool {
        self.binary_inputs.contains(PlayerInputSet::RIGHT)
    }
}

#[derive(Component, Debug)]
pub struct Neck;

#[derive(Component, Debug)]
pub struct Reticle {
    pub max_distance: f32,
    pub from_height: f32,
}

#[derive(Component, Debug)]
pub struct FromCamera(pub Entity);

#[derive(Component, Debug)]
pub struct PlayerCam;

#[derive(
    Component,
    Debug,
    Clone,
    Reflect,
    PartialEq,
    PartialOrd,
    Inspectable,
    Serialize,
    Deserialize,
    Replicate,
)]
#[reflect(Component)]
pub struct Speed(pub f32);
impl Default for Speed {
    fn default() -> Self {
        Self(3.)
    }
}

#[derive(Bundle)]
pub struct PlayerBundle {
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub player_component: Player,
    pub speed: Speed,
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

pub struct PlayerInputPlugin;
impl Plugin for PlayerInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_loopless_state(MouseState::Locked);
        app.insert_resource(LockToggle::default());
        app.insert_resource(MouseSensitivity::default());
        app.insert_resource(PlayerInput::default());
        app.add_system(player_input.label("player_inputs"))
            .add_system(
                player_mouse_input
                    .run_in_state(MouseState::Locked)
                    .run_if(window_focused)
                    .label("player_mouse_input"),
            )
            .add_system(
                toggle_mouse_lock
                    .run_if(window_focused)
                    .label("toggle_mouse_lock"),
            )
            .add_system(mouse_lock.run_if(window_focused).label("toggle_mouse_lock"))
            .add_system_to_stage(
                CoreStage::PostUpdate,
                camera_above_ground
                    .label("camera_above_ground")
                    .after(bevy::transform::TransformSystem::TransformPropagate),
            );

        app.add_network_system(
            update_local_player_inputs
                .label("update_player_inputs")
                .before("player_movement"),
        );
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Speed>();
        app.register_inspectable::<Speed>();

        app.add_plugin(ReplicatePlugin::<Speed>::default());

        app.add_network_system(
            player_swivel_and_tilt
                .label("player_swivel_and_tilt")
                .after("update_player_inputs"),
        )
        .add_network_system(
            player_movement
                .label("player_movement")
                .after("update_player_inputs")
                .after("player_swivel_and_tilt"),
        )
        .add_network_system(
            reticle_move
                .label("reticle_move")
                .after("update_player_inputs")
                .after("player_movement"),
        );
    }
}

pub fn update_local_player_inputs(
    player_input: Res<PlayerInput>,
    mut query: Query<&mut PlayerInput, With<Owned>>,
) {
    if let Ok(mut input) = query.get_single_mut() {
        //info!("setting local player inputs: {:?}", player_input);
        *input = player_input.clone();
    } else {
        warn!("no player to provide input for");
    }
}

pub fn player_movement(mut query: Query<(&Speed, &mut Velocity, &PlayerInput), With<Owned>>) {
    for (speed, mut velocity, player_input) in query.iter_mut() {
        let mut dir = Vec3::new(0.0, 0.0, 0.0);
        if player_input.left() {
            dir.x += -1.;
        }
        if player_input.right() {
            dir.x += 1.;
        }

        if player_input.back() {
            dir.z += 1.;
        }
        if player_input.forward() {
            dir.z += -1.;
        }

        let dir = dir.normalize_or_zero();

        let movement_vector =
            Quat::from_axis_angle(Vec3::Y, player_input.yaw as f32) * dir * speed.0;

        // don't effect the y direction since you can't move in that direction.
        velocity.linvel.x = movement_vector.x;
        velocity.linvel.z = movement_vector.z;
    }
}

pub fn reticle_move(
    players: Query<(&PlayerInput, Option<&Children>), (With<Owned>, With<Player>)>,
    mut reticles: Query<(&mut Transform, &Reticle), Without<Player>>,
) {
    for (player_input, children) in players.iter() {
        if let Some(children) = children {
            for child in children.iter() {
                if let Ok((mut transform, reticle)) = reticles.get_mut(*child) {
                    let current_angle = player_input.pitch.clamp(-1.57, 0.);
                    // new poggers way
                    transform.translation.z = (1.57 + current_angle).tan() * -reticle.from_height;
                    transform.translation.z =
                        transform.translation.z.clamp(-reticle.max_distance, 0.);
                }
            }
        }
    }
}

pub fn player_mouse_input(
    sensitivity: Res<MouseSensitivity>,
    mut ev_mouse: EventReader<MouseMotion>,
    mut player_input: ResMut<PlayerInput>,
) {
    let mut cumulative_delta = Vec2::ZERO;
    for ev in ev_mouse.iter() {
        cumulative_delta += ev.delta;
    }

    player_input.pitch -= sensitivity.0 * cumulative_delta.y * 1.0 / 89.759789 / 2.0;

    player_input.pitch = player_input.pitch.clamp(-1.57, 1.57);

    // We want approximately 5142.8571 dots per 360 I think? At least according to mouse-sensitivity.com's 1 sensitivity 600 DPI valorant measurements.
    player_input.yaw -= sensitivity.0 * cumulative_delta.x * 1.0 / 89.759789 / 2.0;
    player_input.yaw = player_input.yaw.rem_euclid(std::f32::consts::TAU);
}

pub fn camera_above_ground(mut cameras: Query<&mut Transform, With<PlayerCam>>) {
    for mut transform in cameras.iter_mut() {
        transform.translation.y = transform.translation.y.max(-0.5);
    }
}

pub fn player_swivel_and_tilt(
    mut players: Query<
        (&mut Transform, &PlayerInput, Option<&Children>),
        (With<Owned>, With<Player>),
    >,
    mut necks: Query<&mut Transform, (With<Neck>, Without<Player>)>,
) {
    for (mut player_transform, player_inputs, children) in players.iter_mut() {
        player_transform.rotation = Quat::from_axis_angle(Vec3::Y, player_inputs.yaw as f32).into();

        if let Some(children) = children {
            for child in children.iter() {
                if let Ok(mut neck_transform) = necks.get_mut(*child) {
                    neck_transform.rotation =
                        Quat::from_axis_angle(Vec3::X, player_inputs.pitch as f32).into();
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MouseState {
    Free,
    Locked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LockToggle(bool);

impl Default for LockToggle {
    fn default() -> Self {
        Self(true)
    }
}

pub fn toggle_mouse_lock(
    mut commands: Commands,
    windows: Res<Windows>,
    kb: Res<Input<KeyCode>>,
    state: Res<CurrentState<MouseState>>,
    mut toggle: ResMut<LockToggle>,
) {
    if kb.just_pressed(KeyCode::Delete) {
        toggle.0 = !toggle.0;
    }

    let should_free = (kb.pressed(KeyCode::LAlt) || toggle.0)
        && windows
            .get_primary()
            .and_then(|window| Some(window.is_focused()))
            .unwrap_or(true);

    match &state.0 {
        MouseState::Free if !should_free => commands.insert_resource(NextState(MouseState::Locked)),
        MouseState::Locked if should_free => commands.insert_resource(NextState(MouseState::Free)),
        _ => {}
    }
}

pub fn mouse_lock(mut windows: ResMut<Windows>, state: Res<CurrentState<MouseState>>) {
    let locked = state.0 == MouseState::Locked;

    if let Some(window) = windows.get_primary_mut() {
        window.set_cursor_visibility(!locked);
        window.set_cursor_lock_mode(locked);
    }
}

pub fn player_input(keyboard_input: Res<Input<KeyCode>>, mut player_input: ResMut<PlayerInput>) {
    player_input
        .set_left(keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left));
    player_input
        .set_right(keyboard_input.pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right));
    player_input
        .set_forward(keyboard_input.pressed(KeyCode::W) || keyboard_input.pressed(KeyCode::Up));
    player_input
        .set_back(keyboard_input.pressed(KeyCode::S) || keyboard_input.pressed(KeyCode::Down));
}
