use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey, NotShadowReceiver},
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::{MeshVertexBufferLayout, VertexAttributeValues},
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
        },
    }, window::CursorGrabMode,
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
        .add_plugin(potion::egui::SetupEguiPlugin)
        .add_plugin(bevy_editor_pls::EditorPlugin)
        .add_plugin(MaterialPlugin::<LeafMaterial>::default())
        .add_plugin(ShaderUtilsPlugin)
        .add_plugin(CameraControllerPlugin)
        .add_startup_system(setup)
        .add_system_to_stage(CoreStage::PostUpdate, mod_scene)
        .run();
}

#[derive(Component)]
struct GLTFScene;

#[derive(Component)]
struct Inserted;

/// set up a simple 3D scene
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(AmbientLight {
        color: Color::ALICE_BLUE,
        brightness: 0.72,
    });

    const HALF_SIZE: f32 = 10.0;
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            // Configure the projection to better fit the scene
            shadow_projection: OrthographicProjection {
                left: -HALF_SIZE,
                right: HALF_SIZE,
                bottom: -HALF_SIZE,
                top: HALF_SIZE,
                near: -10.0 * HALF_SIZE,
                far: 10.0 * HALF_SIZE,
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
    commands
        .spawn(SceneBundle {
            scene: asset_server.load("models/tree2.gltf#Scene0"),
            ..default()
        })
        .insert(GLTFScene);
    
    // camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 8.0, 15.0)
                .looking_at(Vec3::ZERO, Vec3::Y),
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
impl Material for LeafMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/leaf_material.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
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
#[uuid = "f690fdae-d598-45ab-8225-97e2a3f056e0"]
pub struct LeafMaterial {
    #[uniform(0)]
    color: Color,
    #[texture(1)]
    #[sampler(2)]
    color_texture: Option<Handle<Image>>,
    #[texture(3)]
    #[sampler(4)]
    alpha_texture: Option<Handle<Image>>,
    alpha_mode: AlphaMode,
}

fn mod_scene(
    mut commands: Commands,
    spheres: Query<(Entity, &Handle<Mesh>, &Name), Without<Inserted>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut custom_materials: ResMut<Assets<LeafMaterial>>,
    asset_server: Res<AssetServer>,
) {
    for (e, hand, name) in spheres.iter() {
        if name.as_str().contains("leaves") {
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
                color: Color::DARK_GREEN,
                color_texture: Some(asset_server.load("shaders/leaves.png")),
                alpha_texture: Some(asset_server.load("shaders/leaves_mask.png")),
                alpha_mode: AlphaMode::Blend,
            });
            commands.entity(e).remove::<Handle<StandardMaterial>>();
            commands
                .entity(e)
                .insert((custom_material, NotShadowReceiver, Inserted));
        }
    }
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
