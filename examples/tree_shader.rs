use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey, NotShadowReceiver},
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::{MeshVertexBufferLayout, VertexAttributeValues},
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
        },
    },
    window::CursorGrabMode,
};
use bevy_shader_utils::ShaderUtilsPlugin;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    window: WindowDescriptor {
                        cursor_visible: true,
                        cursor_grab_mode: CursorGrabMode::None,
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
        .add_plugins(potion::egui::SetupEguiPlugin)
        .add_plugins(bevy_editor_pls::EditorPlugin)
        .add_plugins(MaterialPlugin::<LeafMaterial>::default())
        .add_plugins(MaterialPlugin::<WaterMaterial>::default())
        .add_plugins(ShaderUtilsPlugin)
        .add_plugins(CameraControllerPlugin)
        .add_startup_system(setup)
        .add_system_to_stage(CoreStage::PostUpdate, mod_scene)
        .add_system(movement)
        .run();
}

#[derive(Component)]
struct Movable;

#[derive(Component)]
struct Inserted;

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut water: ResMut<Assets<WaterMaterial>>,
) {
    // plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane { size: 100.0 }.into()),
        material: materials.add(Color::rgb(0.1, 0.1, 0.12).into()),
        ..default()
    });
    // water
    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(shape::Plane { size: 5.0 }.into()),
        material: water.add(WaterMaterial {
            color: Color::CYAN,
            color_texture: Some(asset_server.load("shaders/leaves.png")),
        }),
        transform: Transform::from_xyz(5.0, 2.0, 6.0),
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
    commands.spawn(SceneBundle {
        scene: asset_server.load("models/tree.gltf#Scene0"),
        ..default()
    });

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere {
                radius: 2.00,
                ..default()
            })),
            material: materials.add(Color::rgb(1.0, 1.0, 0.1).into()),
            transform: Transform::from_xyz(5.0, 2.0, -3.0),
            ..default()
        },
        Name::new("sphere"),
    ));
    commands.spawn((PbrBundle {
        mesh: meshes.add(Mesh::from(shape::UVSphere {
            radius: 2.00,
            ..default()
        })),
        material: materials.add(Color::rgb(1.0, 1.0, 0.1).into()),
        transform: Transform::from_xyz(-5.0, 2.0, -3.0),
        ..default()
    },));

    // camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(18.0, 16.0, 18.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        CameraController {
            orbit_mode: true,
            orbit_focus: Vec3::new(0.0, 0.5, 0.0),
            ..default()
        },
    ));
}
/// The Material trait is very configurable, but comes with sensible defaults for all methods.
/// You only need to implement functions for features that need non-default behavior. See the Material api docs for details!
impl Material for WaterMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/water.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        if let Some(label) = &mut descriptor.label {
            *label = format!("water__{}", *label).into();
        }
        Ok(())
    }
}

// This is the struct that will be passed to your shader
#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "f690fdae-d598-45ab-8225-97e2a3f053e0"]
pub struct WaterMaterial {
    #[uniform(0)]
    color: Color,
    #[texture(1)]
    #[sampler(2)]
    color_texture: Option<Handle<Image>>,
}

/// The Material trait is very configurable, but comes with sensible defaults for all methods.
/// You only need to implement functions for features that need non-default behavior. See the Material api docs for details!
impl Material for LeafMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/leaf_material2.wgsl".into()
    }
    fn vertex_shader() -> ShaderRef {
        "shaders/leaf_material2.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        if let Some(label) = &mut descriptor.label {
            *label = format!("tree__{}", *label).into();
        }
        Ok(())
    }
}

// This is the struct that will be passed to your shader
#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "dac0f52c-b570-11ed-afa1-0242ac120002"]
pub struct LeafMaterial {
    #[uniform(0)]
    color: Color,
    #[texture(1)]
    #[sampler(2)]
    color_texture: Option<Handle<Image>>,
}

fn mod_scene(
    mut commands: Commands,
    spheres: Query<(Entity, &Handle<Mesh>, &Name), Without<Inserted>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut custom_materials: ResMut<Assets<LeafMaterial>>,
    asset_server: Res<AssetServer>,
) {
    for (e, hand, name) in spheres.iter() {
        if name.as_str().contains("leaves") || name.as_str().contains("sphere") {
            let mesh = meshes.get_mut(hand).unwrap();
            if let Some(VertexAttributeValues::Float32x3(positions)) =
                mesh.attribute(Mesh::ATTRIBUTE_POSITION)
            {
                let colors: Vec<[f32; 4]> = positions
                    .iter()
                    .map(|[r, g, b]| [(1. - *r) / 2., (1. - *g) / 2., (1. - *b) / 2., 1.])
                    .collect();
                mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
            }
            let custom_material = custom_materials.add(LeafMaterial {
                color: Color::YELLOW_GREEN,
                color_texture: Some(asset_server.load("shaders/leaves.png")),
            });
            commands.entity(e).remove::<Handle<StandardMaterial>>();
            commands
                .entity(e)
                .insert((custom_material, NotShadowReceiver, Inserted));
        }
    }
}

fn movement(
    mut movers: Query<(&mut Transform, &Movable)>,
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    for (mut transform, _movable) in &mut movers {
        let mut direction = Vec3::ZERO;
        if input.pressed(KeyCode::W) {
            direction.z -= 1.0;
        }
        if input.pressed(KeyCode::S) {
            direction.z += 1.0;
        }
        if input.pressed(KeyCode::A) {
            direction.x -= 1.0;
        }
        if input.pressed(KeyCode::D) {
            direction.x += 1.0;
        }
        if input.pressed(KeyCode::V) {
            direction.y -= 1.0;
        }
        if input.pressed(KeyCode::C) {
            direction.y += 1.0;
        }

        transform.translation += time.delta_seconds() * 2.0 * direction;
    }
}

use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};

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
    mut query: Query<(&mut Transform, &mut CameraController), With<Camera>>,
) {
    let dt = time.delta_seconds();

    if let Ok((mut transform, mut options)) = query.get_single_mut() {
        if !options.initialized {
            let (_roll, yaw, pitch) = transform.rotation.to_euler(EulerRot::ZYX);
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
        if key_input.just_pressed(options.keyboard_key_enable_mouse) {
            *move_toggled = !*move_toggled;
        }

        // Apply movement update
        if axis_input != Vec3::ZERO {
            let max_speed = if key_input.pressed(options.key_run) {
                options.run_speed
            } else {
                options.walk_speed
            };
            options.velocity = axis_input.normalize() * max_speed;
        } else {
            let friction = options.friction.clamp(0.0, 1.0);
            options.velocity *= 1.0 - friction;
            if options.velocity.length_squared() < 1e-6 {
                options.velocity = Vec3::ZERO;
            }
        }
        let forward = transform.forward();
        let right = transform.right();
        let translation_delta = options.velocity.x * dt * right
            + options.velocity.y * dt * Vec3::Y
            + options.velocity.z * dt * forward;
        let mut scroll_translation = Vec3::ZERO;
        if options.orbit_mode && options.scroll_wheel_speed > 0.0 {
            scroll_translation = scroll_distance
                * transform.translation.distance(options.orbit_focus)
                * options.scroll_wheel_speed
                * forward;
        }
        transform.translation += translation_delta + scroll_translation;
        options.orbit_focus += translation_delta;

        // Handle mouse input
        let mut mouse_delta = Vec2::ZERO;
        if mouse_button_input.pressed(options.mouse_key_enable_mouse) || *move_toggled {
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
                (options.pitch - mouse_delta.y * 0.5 * sensitivity * dt).clamp(
                    -0.99 * std::f32::consts::FRAC_PI_2,
                    0.99 * std::f32::consts::FRAC_PI_2,
                ),
                options.yaw - mouse_delta.x * sensitivity * dt,
            );

            // Apply look update
            transform.rotation = Quat::from_euler(EulerRot::ZYX, 0.0, yaw, pitch);
            options.pitch = pitch;
            options.yaw = yaw;

            if options.orbit_mode {
                let rot_matrix = Mat3::from_quat(transform.rotation);
                transform.translation = options.orbit_focus
                    + rot_matrix.mul_vec3(Vec3::new(
                        0.0,
                        0.0,
                        options.orbit_focus.distance(transform.translation),
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
