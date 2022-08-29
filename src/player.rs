use std::fmt::Debug;

use bevy::input::mouse::MouseWheel;
use bevy::utils::HashSet;
use bevy::{input::mouse::MouseMotion, prelude::*};
use bevy_prototype_debug_lines::DebugLines;
use std::f32::consts::{PI, TAU};

use bevy_egui::EguiContext;
use bevy_inspector_egui::{Inspectable, RegisterInspectable};
use bevy_mod_wanderlust::{
    CharacterControllerBundle, ControllerInput, ControllerPhysicsBundle, ControllerSettings,
    ControllerState,
};
use bevy_rapier3d::prelude::*;
use bevy_rapier3d::rapier::prelude::{JointAxis, MotorModel};
use bevy_renet::renet::RenetServer;
use sabi::prelude::*;

use sabi::stage::{NetworkCoreStage, NetworkSimulationAppExt};

use serde::{Deserialize, Serialize};

use iyes_loopless::{condition::IntoConditionalSystem, prelude::*};
use smooth_bevy_cameras::{
    controllers::orbit::{OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin},
    LookTransformPlugin,
};

use crate::follow::{Follow, FollowPlugin};
use crate::physics::{GRAB_GROUPING, REST_GROUPING};

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

#[derive(Default, Debug, Component, Reflect)]
#[reflect(Component)]
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

#[derive(Clone, Copy, Default, Component, Serialize, Deserialize)]
pub struct PlayerInput {
    /// Movement inputs
    pub binary_inputs: PlayerInputSet,
    /// Grabby hands by index.
    pub grabby_hands: [bool; 8],
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
            grabby_hands: [false; 8],
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

    pub fn set_jump(&mut self, jump: bool) {
        self.binary_inputs.set(PlayerInputSet::JUMP, jump);
    }

    pub fn set_grabby_hands(&mut self, index: usize, grabby_hands: bool) {
        self.grabby_hands[index] = grabby_hands;
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
}

#[derive(Component, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Arm;

#[derive(Component, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Hand;

#[derive(Component, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GrabJoint;

#[derive(Component, Debug)]
pub struct Neck;

#[derive(Component, Debug)]
pub struct Reticle {
    pub max_distance: f32,
    pub from_height: f32,
}

#[derive(Component, Debug)]
pub struct FromCamera(pub Entity);

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
    //Replicate,
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
        app.add_system(
            player_binary_inputs
                .run_if(window_focused)
                .label("binary_inputs"),
        )
        .add_system(
            zoom_on_scroll
                .run_in_state(MouseState::Locked)
                .run_if(window_focused)
                .label("zoom_scroll"),
        )
        .add_system(
            zoom_scroll_for_toi
                .label("zoom_scroll_for_toi")
                .after("zoom_scroll"),
        )
        .add_system(
            player_mouse_inputs
                .run_in_state(MouseState::Locked)
                .run_if(window_focused)
                .label("player_mouse_input"),
        )
        .add_system(initial_mouse_click.label("initial_mouse_click"))
        .add_system(
            toggle_mouse_lock
                .run_if(window_focused)
                .label("toggle_mouse_lock"),
        )
        .add_system(mouse_lock.run_if(window_focused).label("toggle_mouse_lock"));

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
        app.add_plugin(FollowPlugin);
        app.register_type::<Speed>();
        app.register_inspectable::<Speed>();
        app.register_type::<Player>();

        app.insert_resource(Events::<PlayerEvent>::default());

        //app.add_plugin(ReplicatePlugin::<Speed>::default());

        app.add_network_system(
            player_movement
                .label("player_movement")
                .before(bevy_mod_wanderlust::movement)
                .after("update_player_inputs")
                .after("player_swivel_and_tilt"),
        )
        .add_network_system(
            player_grabby_hands
                .label("player_grabby_hands")
                .after(bevy_mod_wanderlust::movement)
                .after("update_player_inputs")
                .after("player_movement"),
        )
        .add_network_system(
            target_position
                .label("target_position")
                .after("update_player_inputs")
                .after("player_grabby_hands"),
        )
        .add_system_to_network_stage(
            NetworkCoreStage::PostUpdate,
            avoid_intersecting.label("avoid_intersecting"),
        )
        .add_network_system(
            character_crouch
                .label("character_crouch")
                .before(bevy_mod_wanderlust::movement)
                .after("update_player_inputs"),
        )
        //.add_network_system(pull_up.label("pull_up").after("update_player_inputs"))
        .add_network_system(
            grab_collider
                .label("grab_collider")
                .after(bevy_mod_wanderlust::movement)
                .after("target_position"),
        )
        .add_network_system(
            player_swivel_and_tilt
                .label("player_swivel_and_tilt")
                .after("update_player_inputs"),
        )
        .add_meta_network_system(setup_player)
        .add_meta_network_system(Events::<PlayerEvent>::update_system);
    }
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
        warn!("no player to provide input for");
    }
}

#[derive(Default, Debug, Clone, Component)]
pub struct LookTransform(pub Transform);

impl LookTransform {
    pub fn rotation(&self) -> Quat {
        self.0.rotation
    }

    pub fn translation(&self) -> Vec3 {
        self.0.translation
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

#[derive(Debug, Clone)]
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
    if kb.just_pressed(KeyCode::Delete) {
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

pub fn mouse_lock(mut windows: ResMut<Windows>, state: Res<CurrentState<MouseState>>) {
    let locked = state.0 == MouseState::Locked;

    if let Some(window) = windows.get_primary_mut() {
        window.set_cursor_visibility(!locked);
        window.set_cursor_lock_mode(locked);
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
    mut mouse_scroll: EventReader<MouseWheel>,
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

    player_input.pitch -= sensitivity.0 * cumulative_delta.y * 1.0 / 89.759789 / 2.0;

    player_input.pitch = player_input.pitch.clamp(-PI / 2.0, PI / 2.0);

    // We want approximately 5142.8571 dots per 360 I think? At least according to mouse-sensitivity.com's 1 sensitivity 600 DPI valorant measurements.
    player_input.yaw -= sensitivity.0 * cumulative_delta.x * 1.0 / 89.759789 / 2.0;
    player_input.yaw = player_input.yaw.rem_euclid(std::f32::consts::TAU);
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PlayerEvent {
    Spawn { id: u64 },
    SetupLocal { id: u64 },
}

pub fn setup_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut asset_server: ResMut<AssetServer>,
    mut player_reader: EventReader<PlayerEvent>,

    mut lobby: ResMut<Lobby>,
    mut server: Option<ResMut<RenetServer>>,
) {
    for (event, id) in player_reader.iter_with_id() {
        info!("player event {:?}: {:?}", id, event);
        match event {
            &PlayerEvent::SetupLocal { id } => {
                let player_entity = *lobby.players.get(&id).expect("Expected a player");

                let reticle_cube =
                    meshes.add(Mesh::from(bevy::render::mesh::shape::Cube { size: 0.2 }));

                let camera = commands
                    .spawn_bundle(Camera3dBundle {
                        transform: Transform::from_translation(Vec3::new(0., 0., 4.))
                            .looking_at(Vec3::ZERO, Vec3::Y),
                        ..Default::default()
                    })
                    .insert(AvoidIntersecting {
                        dir: Vec3::Z,
                        max_toi: 4.0,
                        buffer: 0.075,
                    })
                    .insert(ZoomScroll {
                        current: 4.0,
                        scroll_sensitivity: -0.5,
                        min: 2.0,
                        max: 8.0,
                    })
                    .insert(ZoomScrollForToi)
                    .insert(Name::new("Player Camera"))
                    .id();

                let reticle = commands
                    .spawn_bundle((
                        Transform {
                            translation: Vec3::new(0., 0., 0.),
                            ..Default::default()
                        },
                        GlobalTransform::identity(),
                        Reticle {
                            max_distance: 6.0,
                            from_height: 4.0,
                        },
                        Name::new("Reticle"),
                        FromCamera(camera),
                    ))
                    .id();

                let neck = commands
                    .spawn_bundle((
                        Transform {
                            translation: Vec3::new(0., 1., 0.),
                            ..Default::default()
                        },
                        GlobalTransform::identity(),
                        Neck,
                        Name::new("Neck"),
                    ))
                    .insert_bundle(Follow::translation(player_entity))
                    .id();

                commands.entity(neck).push_children(&[camera]);

                let mut material = StandardMaterial::default();
                material.base_color = Color::hex("800000").unwrap().into();
                material.perceptual_roughness = 0.97;
                material.reflectance = 0.0;
                let red = materials.add(material);

                let ret_mesh = commands
                    .spawn_bundle(PbrBundle {
                        material: red.clone(),
                        mesh: reticle_cube.clone(),
                        ..Default::default()
                    })
                    .id();

                commands.entity(reticle).push_children(&[ret_mesh]);

                commands
                    .entity(player_entity)
                    .insert(PlayerInput::default())
                    .insert(LookTransform::default())
                    .push_children(&[reticle]);
            }
            &PlayerEvent::Spawn { id } => {
                info!("spawning player {}", id);
                let global_transform = GlobalTransform::from(Transform::from_xyz(0.0, 100.0, 0.0));
                // Spawn player cube
                let player_entity = commands
                    .spawn_bundle(CharacterControllerBundle {
                        settings: ControllerSettings {
                            acceleration: 4.0,
                            max_speed: 5.0,
                            max_acceleration_force: 1.0,
                            up_vector: Vec3::Y,
                            gravity: 9.8,
                            max_ground_angle: 45.0 * (PI / 180.0),
                            min_float_offset: -0.3,
                            max_float_offset: 0.05,
                            jump_time: 0.5,
                            jump_initial_force: 4.0,
                            jump_stop_force: 0.3,
                            jump_decay_function: |x| (1.0 - x).sqrt(),
                            jump_skip_ground_check_duration: 0.5,
                            coyote_time_duration: 0.16,
                            jump_buffer_duration: 0.16,
                            force_scale: Vec3::new(1.0, 0.0, 1.0),
                            float_cast_length: 1.0,
                            float_cast_collider: Collider::ball(0.45),
                            float_distance: 0.25,
                            float_strength: 10.0,
                            float_dampen: 1.0,
                            upright_spring_strength: 10.0,
                            upright_spring_damping: 2.0,
                            ..default()
                        },
                        transform: global_transform.compute_transform(),
                        global_transform: global_transform,
                        ..default()
                    })
                    //.insert(crate::deposit::Value::new(500))
                    .insert(Speed::default())
                    .insert(PlayerInput::default())
                    .insert(Player { id: id })
                    .insert(Name::new(format!("Player {}", id.to_string())))
                    //.insert(Owned)
                    //.insert(Loader::<Mesh>::new("scenes/gltfs/boi.glb#Mesh0/Primitive0"))
                    .insert(crate::physics::PLAYER_GROUPING)
                    .id();

                let distance_from_body = 0.7;
                attach_arm(
                    &mut commands,
                    player_entity,
                    global_transform.compute_transform(),
                    Vec3::new(distance_from_body, 0.5, 0.0),
                    0,
                );
                attach_arm(
                    &mut commands,
                    player_entity,
                    global_transform.compute_transform(),
                    Vec3::new(-distance_from_body, 0.5, 0.0),
                    1,
                );

                // for some body horror
                /*
                attach_arm(
                    &mut commands,
                    player_entity,
                    Vec3::new(0.0, 0.5, distance_from_body),
                );

                attach_arm(
                    &mut commands,
                    player_entity,
                    Vec3::new(0.0, 0.5, -distance_from_body),
                );
                */

                // We could send an InitState with all the players id and positions for the client
                // but this is easier to do.

                lobby.players.insert(id, player_entity);
                /*
                               if let Some(ref mut server) = server {
                                   for (existing_id, existing_entity) in lobby.players.iter() {
                                       let message = bincode::serialize(&ServerMessage::PlayerConnected {
                                           id: *existing_id,
                                           entity: (*existing_entity).into(),
                                       })
                                       .unwrap();

                                       server.send_message(id, ServerChannel::Message.id(), message);
                                   }
                               }


                               if let Some(ref mut server) = server {
                                   let message = bincode::serialize(&ServerMessage::PlayerConnected {
                                       id: id,
                                       entity: player_entity.into(),
                                   })
                                   .unwrap();
                                   server.broadcast_message(ServerChannel::Message.id(), message);

                                   let message = bincode::serialize(&ServerMessage::AssignOwnership {
                                       entity: player_entity.into(),
                                   })
                                   .unwrap();
                                   server.send_message(id, ServerChannel::Message.id(), message);

                                   let message = bincode::serialize(&ServerMessage::SetPlayer { id: id }).unwrap();
                                   server.send_message(id, ServerChannel::Message.id(), message);
                               }
                */
            }
        }
    }
}

#[derive(Debug, Clone, Component)]
pub struct ArmId(usize);

pub fn attach_arm(
    commands: &mut Commands,
    to: Entity,
    to_transform: Transform,
    at: Vec3,
    index: usize,
) {
    let max_force = 1000.0;
    let twist_stiffness = 2.0;
    let twist_damping = 0.2;
    let resting_stiffness = 2.0;
    let resting_damping = 0.2;
    let arm_radius = 0.15;
    let hand_radius = 0.175;
    let motor_model = MotorModel::ForceBased;

    let arm_height = Vec3::new(0.0, (1.0 / 1.25) - (hand_radius * 2.0), 0.0);

    let mut arm_joint = SphericalJointBuilder::new()
        .local_anchor1(at) // body local
        .local_anchor2(arm_height)
        .motor_model(JointAxis::AngX, motor_model)
        .motor_model(JointAxis::AngY, motor_model)
        .motor_model(JointAxis::AngZ, motor_model)
        .motor_max_force(JointAxis::AngX, max_force)
        .motor_max_force(JointAxis::AngY, max_force)
        .motor_max_force(JointAxis::AngZ, max_force)
        .motor_position(JointAxis::AngX, 0.0, resting_stiffness, resting_damping)
        .motor_position(JointAxis::AngZ, 0.0, resting_stiffness, resting_damping)
        .motor_position(JointAxis::AngY, 0.0, twist_stiffness, twist_damping)
        .build();
    arm_joint.set_contacts_enabled(false);

    let arm_entity = commands
        .spawn_bundle(TransformBundle::from_transform(to_transform))
        .insert(Name::new("Arm"))
        .insert(Arm)
        .insert(RigidBody::Dynamic)
        .insert(crate::physics::PLAYER_GROUPING)
        .insert(Collider::capsule(Vec3::ZERO, arm_height, arm_radius))
        .insert(ImpulseJoint::new(to, arm_joint))
        .insert(ArmId(index))
        .id();

    let hand_joint = SphericalJointBuilder::new()
        .local_anchor2(Vec3::new(0.0, arm_radius * 2.0 + 0.15, 0.0))
        .motor_model(JointAxis::AngX, motor_model)
        .motor_model(JointAxis::AngY, motor_model)
        .motor_model(JointAxis::AngZ, motor_model)
        .motor_max_force(JointAxis::AngX, max_force)
        .motor_max_force(JointAxis::AngY, max_force)
        .motor_max_force(JointAxis::AngZ, max_force)
        .motor_position(JointAxis::AngX, 0.0, resting_stiffness, resting_damping)
        .motor_position(JointAxis::AngZ, 0.0, resting_stiffness, resting_damping)
        .motor_position(JointAxis::AngY, 0.0, twist_stiffness, twist_damping);
    let mut hand_joint = hand_joint.build();
    hand_joint.set_contacts_enabled(false);

    let hand_entity = commands
        .spawn_bundle(TransformBundle::from_transform(to_transform))
        .insert(Name::new("Hand"))
        .insert(Hand)
        .insert(TargetPosition(None))
        .insert(Grabbing(false))
        .insert(ExternalImpulse::default())
        .insert(RigidBody::Dynamic)
        .insert(crate::physics::PLAYER_GROUPING)
        .insert(Collider::ball(hand_radius))
        .insert(ImpulseJoint::new(arm_entity, hand_joint))
        .insert(ArmId(index))
        .id();
}

pub fn player_swivel_and_tilt(
    mut inputs: Query<(&mut LookTransform, &PlayerInput)>,
    mut necks: Query<(&mut Transform, &Follow), (With<Neck>, Without<Player>)>,
) {
    for (mut neck_transform, follow) in &mut necks {
        if let Ok((mut look_transform, input)) = inputs.get_mut(follow.get()) {
            let rotation = (Quat::from_axis_angle(Vec3::Y, input.yaw as f32)
                * Quat::from_axis_angle(Vec3::X, input.pitch as f32))
            .into();

            neck_transform.rotation = rotation;
            look_transform.0 = *neck_transform;
        }
    }
}

#[derive(Debug, Component, Clone, Copy)]
pub struct TargetPosition(Option<Vec3>);

#[derive(Debug, Component, Clone, Copy)]
pub struct Grabbing(bool);

pub fn player_grabby_hands(
    inputs: Query<(&GlobalTransform, &LookTransform, &PlayerInput)>,
    joints: Query<&ImpulseJoint>,
    mut hands: Query<
        (
            Entity,
            &mut TargetPosition,
            &mut Grabbing,
            &mut CollisionGroups,
            &ArmId,
        ),
        With<Hand>,
    >,
) {
    for (hand, mut target_position, mut grabbing, mut collision_groups, arm_id) in &mut hands {
        target_position.0 = None;

        let arm_entity = if let Ok(joint) = joints.get(hand) {
            joint.parent
        } else {
            continue;
        };

        let player_entity = if let Ok(joint) = joints.get(arm_entity) {
            joint.parent
        } else {
            continue;
        };

        let (global, direction, input) =
            if let Ok((global, direction, input)) = inputs.get(player_entity) {
                (global, direction, input)
            } else {
                continue;
            };

        if input.grabby_hands(arm_id.0) {
            grabbing.0 = true;
            target_position.0 =
                Some(global.translation() + (direction.rotation() * -Vec3::Z * 2.) + Vec3::Y);
            *collision_groups = GRAB_GROUPING;
        } else {
            grabbing.0 = false;
            *collision_groups = REST_GROUPING;
        }
    }
}

pub fn target_position(
    mut hands: Query<(&TargetPosition, &GlobalTransform, &mut ExternalImpulse), With<Hand>>,
) {
    for (target, global, mut impulse) in &mut hands {
        let current = global.translation();
        if let Some(target) = target.0 {
            impulse.impulse = (target - current) * 0.02;
            //info!("impulse: {:?}", impulse);
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct AvoidIntersecting {
    pub dir: Vec3,
    pub max_toi: f32,
    pub buffer: f32,
}

pub fn avoid_intersecting(
    rapier_context: Res<RapierContext>,
    global: Query<&GlobalTransform>,
    mut avoid: Query<(&mut Transform, &Parent, &AvoidIntersecting)>,
) {
    let filter = QueryFilter::default().exclude_dynamic().exclude_sensors();

    for (mut transform, parent, avoid) in &mut avoid {
        let global_transform = if let Ok(global) = global.get(parent.get()) {
            global.compute_transform()
        } else {
            Transform::default()
        };

        let toi = if let Some((_entity, toi)) = rapier_context.cast_ray(
            global_transform.translation,
            global_transform.rotation * avoid.dir,
            avoid.max_toi + avoid.buffer,
            true,
            filter,
        ) {
            toi
        } else {
            avoid.max_toi + avoid.buffer
        };

        transform.translation = avoid.dir * (toi - avoid.buffer);
    }
}

pub fn character_crouch(mut controllers: Query<(&PlayerInput, &mut ControllerSettings)>) {
    let crouch_height = 0.05;
    let full_height = 0.45;
    let threshold = -0.3;
    for (input, mut controller) in &mut controllers {
        // Are we looking sufficiently down?
        if input.pitch < threshold {
            // interpolate between crouch and full based on how far we are pitched downwards
            let crouch_coefficient = input.pitch.abs() / ((PI / 2.0) - threshold.abs());
            let interpolated =
                full_height * (1.0 - crouch_coefficient) + crouch_height * crouch_coefficient;
            controller.float_distance = interpolated;
        } else {
            controller.float_distance = full_height;
        }
    }
}

pub fn pull_up(
    grab_joints: Query<&GrabJoint>,
    hands: Query<(Entity, &Children), With<Hand>>,
    impulse_joints: Query<&ImpulseJoint>,
    mut controllers: Query<(&mut ControllerInput, &PlayerInput)>,
) {
    for (hand, children) in &hands {
        let should_pull_up = children.iter().any(|child| grab_joints.contains(*child));
        if should_pull_up {
            let mut child_entity = hand;
            while let Ok(joint) = impulse_joints.get(child_entity) {
                child_entity = joint.parent;
                if let Ok((mut controller, input)) = controllers.get_mut(child_entity) {
                    let power = 1.0 - ((input.pitch + PI / 2.) / PI);
                    controller.custom_impulse += Vec3::Y * 1.5 * power;
                    break;
                }
            }
        }
    }
}

pub fn grab_collider(
    mut commands: Commands,
    name: Query<&Name>,
    rapier_context: Res<RapierContext>,
    globals: Query<&GlobalTransform>,
    hands: Query<(Entity, &Grabbing, &GlobalTransform, Option<&Children>), With<Hand>>,
    impulse_joints: Query<&ImpulseJoint>,
    grab_joints: Query<&GrabJoint>,
) {
    for (hand, grabbing, global, children) in &hands {
        if grabbing.0 {
            let mut already_grabbing = false;
            let mut related_entities = HashSet::new();

            if let Some(children) = children {
                for child in children.iter() {
                    related_entities.insert(*child);
                    if let Ok(impulse) = impulse_joints.get(*child) {
                        // We are already grabbing something so just skip this hand.
                        already_grabbing = true;
                        related_entities.insert(impulse.parent);
                    }
                }
            }

            // Walk up chain of impulse joints to make sure we aren't grabbing ourselves*
            // *TODO: should also walk down the hierarchy to check that.
            let mut child_entity = hand;
            while let Ok(impulse) = impulse_joints.get(child_entity) {
                related_entities.insert(impulse.parent);
                child_entity = impulse.parent;
            }

            if already_grabbing {
                continue;
            }

            for contact_pair in rapier_context.contacts_with(hand) {
                let other_collider = if contact_pair.collider1() == hand {
                    contact_pair.collider2()
                } else {
                    contact_pair.collider1()
                };

                if related_entities.contains(&other_collider) {
                    continue;
                }

                let contact_points = contact_pair
                    .manifolds()
                    .map(|manifold| {
                        manifold
                            .solver_contacts()
                            .map(|contact| contact.point())
                            .collect::<Vec<_>>()
                    })
                    .flatten()
                    .collect::<Vec<_>>();
                if contact_points.len() == 0 {
                    continue;
                }

                let mut closest_point = Vec3::ZERO;
                let mut closest_distance = f32::MAX;
                for point in &contact_points {
                    let dist = point.distance(global.translation());
                    if dist < closest_distance {
                        closest_point = *point;
                        closest_distance = dist;
                    }
                }

                if let Ok(other_global) = globals.get(other_collider) {
                    // convert back to local space.
                    let other_transform = other_global.compute_transform();
                    let other_matrix = other_global.compute_matrix();
                    let anchor1 = other_matrix.inverse().project_point3(closest_point)
                        * other_transform.scale;
                    let transform = global.compute_transform();
                    let matrix = global.compute_matrix();
                    let anchor2 = matrix.inverse().project_point3(closest_point) * transform.scale;

                    if let Ok(name) = name.get(other_collider) {
                        info!("grabbing {:?}", name.as_str());
                    } else {
                        info!("grabbing entity {:?}", other_collider);
                    }

                    let motor_model = MotorModel::ForceBased;
                    let max_force = 1000.0;
                    let stiffness = 10.0;
                    let damping = 1.0;
                    let grab_joint = SphericalJointBuilder::new()
                        .local_anchor1(anchor1)
                        .local_anchor2(anchor2)
                        .motor_model(JointAxis::AngX, motor_model)
                        .motor_model(JointAxis::AngY, motor_model)
                        .motor_model(JointAxis::AngZ, motor_model)
                        .motor_max_force(JointAxis::AngX, max_force)
                        .motor_max_force(JointAxis::AngY, max_force)
                        .motor_max_force(JointAxis::AngZ, max_force)
                        .motor_position(JointAxis::AngX, 0.0, stiffness, damping)
                        .motor_position(JointAxis::AngZ, 0.0, stiffness, damping)
                        .motor_position(JointAxis::AngY, 0.0, stiffness, damping);
                    let mut grab_joint = grab_joint.build();
                    grab_joint.set_contacts_enabled(false);

                    commands.entity(hand).add_children(|children| {
                        children
                            .spawn()
                            .insert(ImpulseJoint::new(other_collider, grab_joint))
                            .insert(GrabJoint);
                    });
                }
            }
        } else {
            // clean up joints if we aren't grabbing anymore
            if let Some(children) = children {
                for child in children.iter() {
                    if grab_joints.get(*child).is_ok() {
                        commands.entity(*child).despawn_recursive();
                    }
                }
            }
        }
    }
}

pub fn player_movement(
    mut query: Query<
        (
            &GlobalTransform,
            &mut ControllerInput,
            &LookTransform,
            &PlayerInput,
        ),
        //With<Owned>,
    >,
    mut lines: ResMut<DebugLines>,
) {
    for (global, mut controller, look_transform, player_input) in query.iter_mut() {
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

        // we only take into account horizontal rotation so looking down doesn't
        // slow the character down.
        let rotation = Quat::from_axis_angle(Vec3::Y, player_input.yaw as f32);
        let dir = (rotation * dir).normalize_or_zero();

        controller.movement = dir;
        controller.jumping = player_input.jump();

        let current_dir = Vec2::new(global.forward().x, global.forward().z);
        let mut desired_dir = Vec2::new(dir.x, dir.z);

        lines.line(
            global.translation(),
            global.translation() + Vec3::new(current_dir.x, 0.0, current_dir.y),
            0.0,
        );
        lines.line(
            global.translation(),
            global.translation() + Vec3::new(desired_dir.x, 0.0, desired_dir.y),
            0.0,
        );

        // If we are grabby then make the character face the way we are grabbing.
        if player_input.any_grabby_hands() {
            let camera_dir = rotation * -Vec3::Z;
            desired_dir = Vec2::new(camera_dir.x, camera_dir.z);
        }

        if desired_dir.length() > 0.0 && current_dir.length() > 0.0 {
            let y = desired_dir.angle_between(current_dir);
            controller.custom_torque.y = y * 0.1; // avoid overshooting
        }
    }
}

pub fn teleport_player_back(mut players: Query<&mut Transform, With<Player>>) {
    for mut transform in &mut players {
        if transform.translation.y < -100.0 {
            transform.translation = Vec3::new(0.0, 10.0, 0.0);
        }
    }
}
