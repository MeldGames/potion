use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        camera::ScalingMode,
        render_resource::{AsBindGroup, ShaderRef},
        renderer::RenderQueue,
    },
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_resource::*,
        Extract, RenderApp, RenderStage,
    },
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle, RenderMaterials2d},
    window::{PresentMode, CursorGrabMode},
};

pub const CLEAR: Color = Color::rgb(0.3, 0.3, 0.3);
pub const HEIGHT: f32 = 900.0;
pub const RESOLUTION: f32 = 16.0 / 9.0;

fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                window: WindowDescriptor {
                    title: "Shader Test".to_owned(),
                    width: 500.,
                    height: 400.,
                    cursor_visible: true,
                    position: WindowPosition::Automatic,
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
        .add_plugin(Material2dPlugin::<DopeMaterial>::default())
        .add_startup_system(spawn_camera)
        .add_plugin(ExtractResourcePlugin::<ExtractedTime>::default())
        .add_startup_system(setup);

    app.sub_app_mut(RenderApp)
        .add_system_to_stage(RenderStage::Extract, extract_health)
        .add_system_to_stage(RenderStage::Prepare, prepare_my_material);
    app.run();
}

fn setup(
    mut commands: Commands,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut my_material_assets: ResMut<Assets<DopeMaterial>>,
    _assets: Res<AssetServer>,
) {
    commands
        .spawn(MaterialMesh2dBundle {
            mesh: mesh_assets.add(Mesh::from(shape::Quad::default())).into(),
            material: my_material_assets.add(DopeMaterial {
                color: Color::rgb(0.0, 1.0, 0.3),
                time: 0.0,
                image: _assets.load("awesome.png"),
            }),
            transform: Transform::from_xyz(-0.6, 0., 0.),
            ..default()
        })
        .insert(Health { value: 0.2 });

    commands
        .spawn(MaterialMesh2dBundle {
            mesh: mesh_assets.add(Mesh::from(shape::Quad::default())).into(),
            material: my_material_assets.add(DopeMaterial {
                color: Color::rgb(0.0, 1.0, 0.3),
                time: 0.0,
                image: _assets.load("awesome.png"),
            }),
            transform: Transform::from_xyz(0.6, 0., 0.),
            ..default()
        })
        .insert(Health { value: 0.8 });
}

fn spawn_camera(mut commands: Commands) {
    let mut camera = Camera2dBundle::default();

    camera.projection.right = 1.0 * RESOLUTION;
    camera.projection.left = -1.0 * RESOLUTION;

    camera.projection.top = 1.0;
    camera.projection.bottom = -1.0;

    camera.projection.scaling_mode = ScalingMode::None;

    commands.spawn(camera);
}

#[derive(AsBindGroup, TypeUuid, Clone)]
#[uuid = "e078ff4b-08e3-49d7-912f-93fe1b247cbb"]
pub struct DopeMaterial {
    #[uniform(0)]
    color: Color,
    #[uniform(0)]
    time: f32,
    #[texture(1)]
    #[sampler(2)]
    image: Handle<Image>,
}

impl Material2d for DopeMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/my_material.wgsl".into()
    }
}

#[derive(Component, Clone, Copy)]
struct Health {
    value: f32,
}

fn prepare_my_material(
    health_query: Query<(&Health, &Handle<DopeMaterial>)>,
    materials: Res<RenderMaterials2d<DopeMaterial>>,
    render_queue: Res<RenderQueue>,
    time: Res<ExtractedTime>,
) {
    for (health, handle) in &health_query {
        if let Some(material) = materials.get(handle) {
            for binding in material.bindings.iter() {
                if let OwnedBindingResource::Buffer(cur_buffer) = binding {
                    let mut buffer = encase::UniformBuffer::new(Vec::new());
                    buffer
                        .write(&DopeMaterialUniformData {
                            color: Color::rgb(health.value, 0., 0.),
                            time: time.seconds_since_startup % 1.0,
                        })
                        .unwrap();
                    render_queue.write_buffer(cur_buffer, 0, buffer.as_ref());
                }
            }
        }
    }
}

#[derive(Clone, ShaderType)]
struct DopeMaterialUniformData {
    color: Color,
    time: f32,
}

#[derive(Resource)]
struct ExtractedTime {
    seconds_since_startup: f32,
}

impl ExtractResource for ExtractedTime {
    type Source = Time;

    fn extract_resource(time: &Self::Source) -> Self {
        ExtractedTime {
            seconds_since_startup: time.elapsed_seconds() as f32,
        }
    }
}

fn extract_health(
    mut commands: Commands,
    health_query: Extract<Query<(Entity, &Health, &Handle<DopeMaterial>)>>,
) {
    for (entity, health, handle) in health_query.iter() {
        commands
            .get_or_spawn(entity)
            .insert(*health)
            .insert(handle.clone());
    }
}
