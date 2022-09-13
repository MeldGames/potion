use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::{
            MeshVertexBufferLayout, VertexAttributeValues,
        },
        render_resource::{
            AsBindGroup, Face, RenderPipelineDescriptor,
            ShaderRef, SpecializedMeshPipelineError,
        },
    },
    scene::SceneInstance,
};
use bevy_shader_utils::ShaderUtilsPlugin;

fn main() {
    App::new()
        .insert_resource(ClearColor(
            Color::hex("071f3c").unwrap(),
        ))
        .add_plugins(DefaultPlugins)
        .add_plugin(ShaderUtilsPlugin)
        .add_plugin(
            MaterialPlugin::<CustomMaterial>::default(),
        )
        .add_startup_system(setup)
        .add_system(update_time_for_custom_material)
        // for the time to update in the shader,
        // this mod_scene must run before we try to update the time
        // TODO: figure out why in more detail.
        .add_system(mod_scene)
        .run();
}

#[derive(Component)]
struct GLTFScene;

#[derive(Component)]
struct Inserted;

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(AmbientLight {
        color: Color::ALICE_BLUE,
        brightness: 0.72,
    });

    const HALF_SIZE: f32 = 10.0;
    commands.spawn_bundle(DirectionalLightBundle {
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
        .spawn_bundle(SceneBundle {
            scene: asset_server.load(
                "models/tree_stylized.gltf#Scene0",
            ),
            ..default()
        })
        .insert(GLTFScene);
    // camera
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 15.5, 15.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn update_time_for_custom_material(
    mut materials: ResMut<Assets<CustomMaterial>>,
    time: Res<Time>,
) {
    for material in materials.iter_mut() {
        material.1.time =
            time.seconds_since_startup() as f32;
    }
}

/// The Material trait is very configurable, but comes with sensible defaults for all methods.
/// You only need to implement functions for features that need non-default behavior. See the Material api docs for details!
impl Material for CustomMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/custom_material.wgsl".into()
    }


    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }
    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        if let Some(label) = &mut descriptor.label {
            *label = format!("shield_{}", *label).into();
        }
        Ok(())
    }
}

// This is the struct that will be passed to your shader
#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "f690fdae-d598-45ab-8225-97e2a3f056e0"]
pub struct CustomMaterial {
    #[uniform(0)]
    time: f32,
    #[uniform(0)]
    color: Color,
    #[texture(1)]
    #[sampler(2)]
    color_texture: Option<Handle<Image>>,
    alpha_mode: AlphaMode,
}


fn mod_scene(
    mut commands: Commands,
    spheres: Query<
        (Entity, &Handle<Mesh>, &Name),
        Without<Inserted>,
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut custom_materials: ResMut<Assets<CustomMaterial>>,
    asset_server: Res<AssetServer>,
) {
    for (e, hand, name) in spheres.iter() {
        if name.as_str().contains("testplane"){
            let mesh = meshes.get_mut(hand).unwrap();
            if let Some(VertexAttributeValues::Float32x3(
                positions,
            )) = mesh.attribute(Mesh::ATTRIBUTE_POSITION)
            {
                let colors: Vec<[f32; 4]> = positions
                    .iter()
                    .map(|[r, g, b]| {
                        [
                            (1. - *r) / 2.,
                            (1. - *g) / 2.,
                            (1. - *b) / 2.,
                            1.,
                        ]
                    })
                    .collect();
                mesh.insert_attribute(
                    Mesh::ATTRIBUTE_COLOR,
                    colors,
                );
            }
            let custom_material =
                custom_materials.add(CustomMaterial {
                    color: Color::BLUE,
                    color_texture: Some(asset_server.load("shaders/leaf.png")),
                    alpha_mode: AlphaMode::Blend,
                    time: 0.5,
                });
            commands
                .entity(e)
                .remove::<Handle<StandardMaterial>>();
            commands.entity(e).insert(custom_material);
            commands.entity(e).insert(Inserted);
        }
        
    }
}