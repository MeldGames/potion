
use bevy::{
    prelude::*,
    render::{
    }, window::CursorGrabMode,
};
use bevy_shader_utils::ShaderUtilsPlugin;
use bevy_prototype_debug_lines::*;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    window: WindowDescriptor {
                        cursor_visible: false,
                        cursor_grab_mode: CursorGrabMode::Locked,
                        present_mode: bevy::window::PresentMode::Immediate,
                        ..default()
                    },
                    ..default()
                })
                .set(AssetPlugin {
                    watch_for_changes: true,
                    ..default()
                }),
        )
        .insert_resource(ClearColor(Color::hex("071f3c").unwrap()))
        .add_plugin(DebugLinesPlugin::with_depth_test(true))
        .add_plugin(potion::egui::SetupEguiPlugin)
        .add_plugin(bevy_editor_pls::EditorPlugin)
        .add_plugin(CameraControllerPlugin)
        .add_startup_system(setup)
        .add_system(twist)
        .run();
}

#[derive(Debug, Clone)]
pub struct Twist {
    pub yaw: f32,
    pub pitch: f32,
}

impl Default for Twist {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
        }
    }
}

impl Twist {
    pub fn rotation(&self) -> Quat {
        Quat::from_axis_angle(Vec3::X, self.yaw) * Quat::from_axis_angle(Vec3::Z, self.pitch)
    }

    pub fn extruded(&self) -> Vec3 {
        self.rotation() * (Vec3::X * 1.01)
    }
}

pub fn twist(
    kb: Res<Input<KeyCode>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut twist: Local<Twist>,
    mut lines: ResMut<DebugLines>,
) {
    for delta in mouse_motion.iter().map(|event| event.delta) {
        let previous = twist.clone();
        twist.yaw -= delta.x / 180.0;
        twist.pitch += delta.y / 880.0;

        lines.line_colored(previous.extruded(), twist.extruded(), 3.0, Color::RED);
    }
}

fn setup(
    mut commands: Commands, 
    asset_server: Res<AssetServer>, 
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {

    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane { size: 100.0, ..default() }.into()),
        material: materials.add(Color::rgb(0.2, 0.6, 0.32).into()),
        transform: Transform::from_xyz(0.0, -2.0, 0.0),
        ..default()
    });
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::UVSphere { radius: 1.0, ..default() }.into()),
        material: materials.add(Color::rgb(0.1, 0.1, 0.12).into()),
        ..default()
    });
    commands.insert_resource(AmbientLight {
        color: Color::ALICE_BLUE,
        brightness: 0.72,
    });

    const HALF_SIZE: f32 = 100.0;
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            // Configure the projection to better fit the scene
            shadow_projection: OrthographicProjection {
                left: -HALF_SIZE,
                right: HALF_SIZE,
                bottom: -HALF_SIZE,
                top: HALF_SIZE,
                near: -10.0 * HALF_SIZE,
                far: 1000.0 * HALF_SIZE,
                ..default()
            },
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4),
            ..default()
        },
        ..default()
    });
    
    // camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(2.0, 2.0, 2.0)
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        CameraController {
            orbit_mode: true,
            orbit_focus: Vec3::new(0.0, 0.0, 0.0),
            ..default()
        },
    ));
}

use bevy::{
    input::mouse::{
        MouseMotion, MouseScrollUnit, MouseWheel,
    },
};

/// Provides basic movement functionality to the attached camera
#[derive(Component)]
pub struct CameraController {
    pub enabled: bool,
    pub initialized: bool,
    pub sensitivity: f32,
    pub key_forward: KeyCode,
    pub key_back: KeyCode,
    pub key_left: KeyCode,
    pub key_right: KeyCode,
    pub key_up: KeyCode,
    pub key_down: KeyCode,
    pub key_run: KeyCode,
    pub mouse_key_enable_mouse: MouseButton,
    pub keyboard_key_enable_mouse: KeyCode,
    pub walk_speed: f32,
    pub run_speed: f32,
    pub friction: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub velocity: Vec3,
    pub orbit_focus: Vec3,
    pub orbit_mode: bool,
    pub scroll_wheel_speed: f32,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            enabled: true,
            initialized: false,
            sensitivity: 0.25,
            key_forward: KeyCode::W,
            key_back: KeyCode::S,
            key_left: KeyCode::A,
            key_right: KeyCode::D,
            key_up: KeyCode::F,
            key_down: KeyCode::Q,
            key_run: KeyCode::LShift,
            mouse_key_enable_mouse: MouseButton::Left,
            keyboard_key_enable_mouse: KeyCode::M,
            walk_speed: 5.0,
            run_speed: 15.0,
            friction: 0.5,
            pitch: 0.0,
            yaw: 0.0,
            velocity: Vec3::ZERO,
            orbit_focus: Vec3::ZERO,
            orbit_mode: false,
            scroll_wheel_speed: 0.1,
        }
    }
}

pub fn camera_controller(
    time: Res<Time>,
    mut mouse_events: EventReader<MouseMotion>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut scroll_evr: EventReader<MouseWheel>,
    key_input: Res<Input<KeyCode>>,
    mut move_toggled: Local<bool>,
    mut query: Query<
        (&mut Transform, &mut CameraController),
        With<Camera>,
    >,
) {
    let dt = time.delta_seconds();

    if let Ok((mut transform, mut options)) =
        query.get_single_mut()
    {
        if !options.initialized {
            let (_roll, yaw, pitch) =
                transform.rotation.to_euler(EulerRot::ZYX);
            options.yaw = yaw;
            options.pitch = pitch;
            options.initialized = true;
        }
        if !options.enabled {
            return;
        }

        let mut scroll_distance = 0.0;

        // Handle scroll input
        for ev in scroll_evr.iter() {
            match ev.unit {
                MouseScrollUnit::Line => {
                    scroll_distance = ev.y;
                }
                MouseScrollUnit::Pixel => (),
            }
        }

        // Handle key input
        let mut axis_input = Vec3::ZERO;
        if key_input.pressed(options.key_forward) {
            axis_input.z += 1.0;
        }
        if key_input.pressed(options.key_back) {
            axis_input.z -= 1.0;
        }
        if key_input.pressed(options.key_right) {
            axis_input.x += 1.0;
        }
        if key_input.pressed(options.key_left) {
            axis_input.x -= 1.0;
        }
        if key_input.pressed(options.key_up) {
            axis_input.y += 1.0;
        }
        if key_input.pressed(options.key_down) {
            axis_input.y -= 1.0;
        }
        if key_input
            .just_pressed(options.keyboard_key_enable_mouse)
        {
            *move_toggled = !*move_toggled;
        }

        // Apply movement update
        if axis_input != Vec3::ZERO {
            let max_speed =
                if key_input.pressed(options.key_run) {
                    options.run_speed
                } else {
                    options.walk_speed
                };
            options.velocity =
                axis_input.normalize() * max_speed;
        } else {
            let friction = options.friction.clamp(0.0, 1.0);
            options.velocity *= 1.0 - friction;
            if options.velocity.length_squared() < 1e-6 {
                options.velocity = Vec3::ZERO;
            }
        }
        let forward = transform.forward();
        let right = transform.right();
        let translation_delta =
            options.velocity.x * dt * right
                + options.velocity.y * dt * Vec3::Y
                + options.velocity.z * dt * forward;
        let mut scroll_translation = Vec3::ZERO;
        if options.orbit_mode
            && options.scroll_wheel_speed > 0.0
        {
            scroll_translation = scroll_distance
                * transform
                    .translation
                    .distance(options.orbit_focus)
                * options.scroll_wheel_speed
                * forward;
        }
        transform.translation +=
            translation_delta + scroll_translation;
        options.orbit_focus += translation_delta;

        // Handle mouse input
        let mut mouse_delta = Vec2::ZERO;
        if mouse_button_input
            .pressed(options.mouse_key_enable_mouse)
            || *move_toggled
        {
            for mouse_event in mouse_events.iter() {
                mouse_delta += mouse_event.delta;
            }
        }

        if mouse_delta != Vec2::ZERO {
            let sensitivity = if options.orbit_mode {
                options.sensitivity * 2.0
            } else {
                options.sensitivity
            };
            let (pitch, yaw) = (
                (options.pitch
                    - mouse_delta.y
                        * 0.5
                        * sensitivity
                        * dt)
                    .clamp(
                        -0.99 * std::f32::consts::FRAC_PI_2,
                        0.99 * std::f32::consts::FRAC_PI_2,
                    ),
                options.yaw
                    - mouse_delta.x * sensitivity * dt,
            );

            // Apply look update
            transform.rotation = Quat::from_euler(
                EulerRot::ZYX,
                0.0,
                yaw,
                pitch,
            );
            options.pitch = pitch;
            options.yaw = yaw;

            if options.orbit_mode {
                let rot_matrix =
                    Mat3::from_quat(transform.rotation);
                transform.translation = options.orbit_focus
                    + rot_matrix.mul_vec3(Vec3::new(
                        0.0,
                        0.0,
                        options.orbit_focus.distance(
                            transform.translation,
                        ),
                    ));
            }
        }
    }
}

/// Simple flying camera plugin.
/// In order to function, the [`CameraController`] component should be attached to the camera entity.
#[derive(Default)]
pub struct CameraControllerPlugin;

impl Plugin for CameraControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(camera_controller);
    }
}
