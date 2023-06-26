use std::fmt::Debug;

use bevy::input::mouse::MouseWheel;
use bevy::{input::mouse::MouseMotion, prelude::*, window::PrimaryWindow};
use bevy_editor_pls::editor::Editor;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

use super::prelude::*;

#[derive(Resource, Debug)]
pub struct MouseSensitivity(f32);

impl Default for MouseSensitivity {
    fn default() -> Self {
        Self(1.0)
    }
}

bitflags::bitflags! {
    #[derive(Default, Serialize, Deserialize)]
    pub struct PlayerInputSet: u16 {
        const FORWARD = 1 << 1;
        const BACK = 1 << 2;
        const LEFT = 1 << 3;
        const RIGHT = 1 << 4;
        const JUMP = 1 << 5;
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

        keys += match self.contains(Self::JUMP) {
            true => "+",
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

#[derive(Resource, Clone, Copy, Default, Component, Serialize, Deserialize)]
pub struct PlayerInput {
    /// Movement inputs
    pub binary_inputs: PlayerInputSet,
    /// Arm should extend by index.
    pub extend_arm: [bool; 8],
    /// Vertical rotation of camera
    pub pitch: f32,
    /// Horizontal rotation of camera
    pub yaw: f32,
    /// Modifier for grabbing
    pub twist: bool,
}

impl Debug for PlayerInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlayerInput")
            .field("keys", &self.binary_inputs.shorthand_display())
            .field("pitch", &Radians(self.pitch))
            .field("yaw", &Radians(self.yaw))
            .field("twist", &self.twist)
            .field(
                "extend_arm",
                &self
                    .extend_arm
                    .iter()
                    .enumerate()
                    .filter(|(_, grabbing)| **grabbing)
                    .map(|(index, _)| index)
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

#[derive(Debug, Clone, Copy, Default, Component, Serialize, Deserialize)]
pub struct CastInput {}

impl PlayerInput {
    pub fn new() -> Self {
        Self {
            binary_inputs: PlayerInputSet::empty(),
            extend_arm: [false; 8],
            pitch: 0.0,
            yaw: 0.0,
            twist: false,
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

    pub fn set_jump(&mut self, jump: bool) {
        self.binary_inputs.set(PlayerInputSet::JUMP, jump);
    }

    pub fn set_extend_arm(&mut self, index: usize, extend_arm: bool) {
        self.extend_arm[index] = extend_arm;
    }

    pub fn set_twist(&mut self, twist: bool) {
        self.twist = twist;
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

    pub fn jump(&self) -> bool {
        self.binary_inputs.contains(PlayerInputSet::JUMP)
    }

    pub fn any_extend_arm(&self) -> bool {
        self.extend_arm.iter().any(|grabby| *grabby)
    }

    pub fn extend_arm(&self, index: usize) -> bool {
        self.extend_arm[index]
    }

    pub fn twist(&self) -> bool {
        self.twist
    }
}

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum MouseState {
    Free,
    #[default]
    Locked,
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LockToggle(bool);

impl Default for LockToggle {
    fn default() -> Self {
        Self(true)
    }
}

#[derive(Resource, Debug, Clone)]
pub struct InitialClick;

pub fn initial_mouse_click(
    mut commands: Commands,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mouse_input: Res<Input<MouseButton>>,
    mut toggle: ResMut<LockToggle>,
    initial_click: Option<Res<InitialClick>>,
) {
    let primary_focused = primary_window
        .get_single()
        .and_then(|window| Ok(window.focused))
        .unwrap_or(false);
    if !primary_focused {
        toggle.0 = true;
        commands.remove_resource::<InitialClick>();
    }

    if let None = initial_click {
        if mouse_input.any_pressed([MouseButton::Left, MouseButton::Right]) {
            info!("initial click");
            commands.insert_resource(InitialClick);
        }
    }
}

pub fn toggle_mouse_lock(
    windows: Query<&Window, With<PrimaryWindow>>,
    kb: Res<Input<KeyCode>>,
    state: Res<State<MouseState>>,
    mut next_state: ResMut<NextState<MouseState>>,
    mut toggle: ResMut<LockToggle>,
    _initial_click: Option<Res<InitialClick>>,
) {
    if kb.just_pressed(KeyCode::Escape) || kb.just_pressed(KeyCode::Delete) {
        toggle.0 = !toggle.0;
    }

    let primary_focused = windows
        .get_single()
        .and_then(|window| Ok(window.focused))
        .unwrap_or(false);

    let should_lock = (kb.pressed(KeyCode::LAlt) || toggle.0) && primary_focused; // && initial_click.is_some();

    match &state.0 {
        MouseState::Free if should_lock => next_state.set(MouseState::Locked),
        MouseState::Locked if !should_lock => next_state.set(MouseState::Free),
        _ => {}
    }
}

pub fn mouse_lock(
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    editor: Option<Res<Editor>>,
    state: Res<State<MouseState>>,
) {
    let editor_active = editor.map(|state| state.active()).unwrap_or(false);
    let locked = state.0 == MouseState::Locked && !editor_active;

    if let Ok(mut window) = windows.get_single_mut() {
        window.cursor.visible = !locked;

        if locked {
            let oob = match window.cursor_position() {
                Some(position) => {
                    position.x > window.width()
                        || position.x < 0.0
                        || position.y > window.height()
                        || position.y < 0.0
                }
                None => true,
            };
            if oob {
                info!("position: {:?}", window.cursor_position());
                let center_cursor = Vec2::new(window.width() / 2.0, window.height() / 2.0);
                window.set_cursor_position(Some(center_cursor));
            }
            window.cursor.grab_mode = bevy::window::CursorGrabMode::Locked;
        } else {
            window.cursor.grab_mode = bevy::window::CursorGrabMode::None;
        }
    }
}

pub fn player_binary_inputs(
    keyboard_input: Res<Input<KeyCode>>,
    mouse_input: Res<Input<MouseButton>>,
    mut player_input: ResMut<PlayerInput>,
) {
    player_input
        .set_left(keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left));
    player_input
        .set_right(keyboard_input.pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right));
    player_input
        .set_forward(keyboard_input.pressed(KeyCode::W) || keyboard_input.pressed(KeyCode::Up));
    player_input
        .set_back(keyboard_input.pressed(KeyCode::S) || keyboard_input.pressed(KeyCode::Down));
    player_input
        .set_jump(keyboard_input.pressed(KeyCode::Space) || keyboard_input.pressed(KeyCode::Back));
    player_input.set_extend_arm(
        0,
        mouse_input.pressed(MouseButton::Right)
            || keyboard_input.pressed(KeyCode::K)
            || keyboard_input.pressed(KeyCode::LShift),
    );
    player_input.set_extend_arm(
        1,
        mouse_input.pressed(MouseButton::Left)
            || keyboard_input.pressed(KeyCode::J)
            || keyboard_input.pressed(KeyCode::LShift),
    );

    player_input.set_twist(keyboard_input.pressed(KeyCode::LControl));
}

#[derive(Debug, Clone, Component)]
pub struct ZoomScroll {
    pub current: f32,
    pub scroll_sensitivity: f32,
    pub min: f32,
    pub max: f32,
}

pub fn zoom_on_scroll(
    mut mouse_scroll: EventReader<MouseWheel>,
    mut zooms: Query<&mut ZoomScroll>,
) {
    let mut cumulative_scroll = 0.0;
    for event in mouse_scroll.iter() {
        cumulative_scroll += event.y;
    }

    for mut zoom in &mut zooms {
        zoom.current =
            (zoom.current + cumulative_scroll * zoom.scroll_sensitivity).clamp(zoom.min, zoom.max);
    }
}

#[derive(Debug, Clone, Component)]

pub struct ZoomScrollForToi;

pub fn zoom_scroll_for_toi(
    _mouse_scroll: EventReader<MouseWheel>,
    mut zooms: Query<(&ZoomScroll, &mut AvoidIntersecting)>,
) {
    for (zoom, mut avoid) in &mut zooms {
        avoid.max_toi = zoom.current;
    }
}

pub fn player_mouse_inputs(
    sensitivity: Res<MouseSensitivity>,
    mut ev_mouse: EventReader<MouseMotion>,
    mut player_input: ResMut<PlayerInput>,
    kb: Res<Input<KeyCode>>,
) {
    let mut cumulative_delta = Vec2::ZERO;
    for ev in ev_mouse.iter() {
        cumulative_delta += ev.delta;
    }

    if kb.pressed(KeyCode::RControl) {
        return;
    }

    player_input.pitch -= sensitivity.0 * cumulative_delta.y / 180.0;
    player_input.pitch = player_input.pitch.clamp(-PI / 2.0, PI / 2.0);

    player_input.yaw -= sensitivity.0 * cumulative_delta.x / 180.0;
    player_input.yaw = player_input.yaw.rem_euclid(std::f32::consts::TAU);
}

pub fn update_local_player_inputs(
    player_input: Res<PlayerInput>,
    //mut query: Query<&mut PlayerInput, With<Owned>>,
    mut query: Query<&mut PlayerInput>,
) {
    if let Ok(mut input) = query.get_single_mut() {
        //info!("setting local player inputs: {:?}", player_input);
        *input = player_input.clone();
    } else {
        //warn!("no player to provide input for");
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InputSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CollectInputs;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MetaInputs;

pub struct PlayerInputPlugin;
impl Plugin for PlayerInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<MouseState>();
        app.insert_resource(LockToggle::default());
        app.insert_resource(MouseSensitivity::default());
        app.insert_resource(PlayerInput::default());
        app.configure_sets((CollectInputs, MetaInputs).in_set(InputSet));
        app.configure_set(
            CollectInputs
                .run_if(crate::mouse_locked)
                .run_if(crate::window_focused)
                .run_if(not(crate::editor_active)),
        );
        app.add_systems(
            (
                player_binary_inputs,
                zoom_on_scroll,
                zoom_scroll_for_toi,
                player_mouse_inputs,
            )
                .in_set(CollectInputs),
        )
        .add_systems(
            (
                initial_mouse_click,
                toggle_mouse_lock,
                mouse_lock,
                update_local_player_inputs,
            )
                .in_set(MetaInputs),
        );
    }
}
