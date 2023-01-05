use std::fmt::Debug;

use bevy_editor_pls::EditorState;
use bevy::input::mouse::MouseWheel;
use bevy::{input::mouse::MouseMotion, prelude::*};
use std::f32::consts::PI;
use sabi::stage::NetworkSimulationAppExt;
use serde::{Deserialize, Serialize};
use iyes_loopless::{condition::IntoConditionalSystem, prelude::*};

use super::prelude::*;
use crate::player::editor_active;

use super::window_focused;

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
    /// Grabby hands by index.
    pub grabby_hands: [bool; 8],
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
                "grabby_hands",
                &self
                    .grabby_hands
                    .iter()
                    .enumerate()
                    .filter(|(_, grabbing)| **grabbing)
                    .map(|(index, _)| index),
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
            grabby_hands: [false; 8],
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

    pub fn set_grabby_hands(&mut self, index: usize, grabby_hands: bool) {
        self.grabby_hands[index] = grabby_hands;
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

    pub fn any_grabby_hands(&self) -> bool {
        self.grabby_hands.iter().any(|grabby| *grabby)
    }

    pub fn grabby_hands(&self, index: usize) -> bool {
        self.grabby_hands[index]
    }

    pub fn twist(&self) -> bool {
        self.twist
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MouseState {
    Free,
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
    mouse_input: Res<Input<MouseButton>>,
    initial_click: Option<Res<InitialClick>>,
) {
    if let None = initial_click {
        if mouse_input.any_pressed([MouseButton::Left, MouseButton::Right]) {
            info!("initial click");
            commands.insert_resource(InitialClick);
        }
    }
}

pub fn toggle_mouse_lock(
    mut commands: Commands,
    windows: Res<Windows>,
    kb: Res<Input<KeyCode>>,
    state: Res<CurrentState<MouseState>>,
    mut toggle: ResMut<LockToggle>,
    initial_click: Option<Res<InitialClick>>,
) {
    if kb.just_pressed(KeyCode::Escape) || kb.just_pressed(KeyCode::Delete) {
        toggle.0 = !toggle.0;
    }

    let should_lock = (kb.pressed(KeyCode::LAlt) || toggle.0)
        && windows
            .get_primary()
            .and_then(|window| Some(window.is_focused()))
            .unwrap_or(false)
        && initial_click.is_some();

    match &state.0 {
        MouseState::Free if should_lock => commands.insert_resource(NextState(MouseState::Locked)),
        MouseState::Locked if !should_lock => commands.insert_resource(NextState(MouseState::Free)),
        _ => {}
    }
}

pub fn mouse_lock(mut windows: ResMut<Windows>, editor: Option<Res<EditorState>>, state: Res<CurrentState<MouseState>>) {
    let editor_active = editor.map(|state| state.active).unwrap_or(false);
    let locked = state.0 == MouseState::Locked && !editor_active;


    if let Some(window) = windows.get_primary_mut() {
        window.set_cursor_visibility(!locked);
        if locked {
            window.set_cursor_grab_mode(bevy::window::CursorGrabMode::Locked);
        } else {
            window.set_cursor_grab_mode(bevy::window::CursorGrabMode::None);
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
    player_input.set_grabby_hands(
        0,
        mouse_input.pressed(MouseButton::Right) || keyboard_input.pressed(KeyCode::LShift),
    );
    player_input.set_grabby_hands(
        1,
        mouse_input.pressed(MouseButton::Left) || keyboard_input.pressed(KeyCode::LShift),
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
) {
    let mut cumulative_delta = Vec2::ZERO;
    for ev in ev_mouse.iter() {
        cumulative_delta += ev.delta;
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

pub struct PlayerInputPlugin;
impl Plugin for PlayerInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_loopless_state(MouseState::Locked);

        app.insert_resource(LockToggle::default());
        app.insert_resource(MouseSensitivity::default());
        app.insert_resource(PlayerInput::default());
        app.add_system(
            player_binary_inputs
                .run_if(window_focused)
                .run_if_not(editor_active)
                .label("binary_inputs"),
        )
        .add_system(
            zoom_on_scroll
                .run_in_state(MouseState::Locked)
                .run_if(window_focused)
                .run_if_not(editor_active)
                .label("zoom_scroll"),
        )
        .add_system(
            zoom_scroll_for_toi
                .run_if(window_focused)
                .run_if_not(editor_active)
                .label("zoom_scroll_for_toi")
                .after("zoom_scroll"),
        )
        .add_system(
            player_mouse_inputs
                .run_in_state(MouseState::Locked)
                .run_if(window_focused)
                .run_if_not(editor_active)
                .label("player_mouse_input"),
        )
        .add_system(initial_mouse_click.label("initial_mouse_click"))
        .add_system(
            toggle_mouse_lock
                .run_if(window_focused)
                .run_if_not(editor_active)
                .label("toggle_mouse_lock"),
        )
        .add_system(mouse_lock.run_if(window_focused).label("set_mouse_lock"));

        app.add_network_system(
            update_local_player_inputs
                .label("update_player_inputs")
                .before("player_movement"),
        );
    }
}
