use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::{MeshVertexBufferLayout, VertexAttributeValues},
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
        },
    },
};
use bevy_rapier3d::prelude::*;
use bevy_shader_utils::ShaderUtilsPlugin;

pub struct TreesPlugin;
impl Plugin for TreesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<CustomMaterial>::default())
            .add_plugin(ShaderUtilsPlugin)
            //.add_system(update_time_for_custom_material)
            .add_system(mod_scene);
    }
}

#[derive(Component)]
struct GLTFScene;

#[derive(Component)]
struct Inserted;

pub fn spawn_trees(
    commands: &mut Commands,
    asset_server: &AssetServer,
    _meshes: &mut Assets<Mesh>,
) {
    let tree_positions = vec![Vec3::new(12.5, 0., -0.075)];
    for i in tree_positions {
        let _tree = commands
            .spawn_bundle(SceneBundle {
                scene: asset_server.load("models/tree_stylized.gltf#Scene0"),
                transform: Transform {
                    translation: i.clone(),
                    scale: Vec3::splat(1.),
                    ..default()
                },
                ..default()
            })
            .insert_bundle((
                ColliderMassProperties::Density(5.0),
                RigidBody::Fixed,
                Collider::cylinder(3.4, 0.2),
                Name::new("Tree"),
                crate::physics::TERRAIN_GROUPING,
            ))
            .id();
    }
}

fn update_time_for_custom_material(mut materials: ResMut<Assets<CustomMaterial>>, time: Res<Time>) {
    for material in materials.iter_mut() {
        material.1.time = time.seconds_since_startup() as f32;
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
        _key: MaterialPipelineKey<Self>,
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
    spheres: Query<(Entity, &Handle<Mesh>, &Name), Without<Inserted>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut custom_materials: ResMut<Assets<CustomMaterial>>,
    asset_server: Res<AssetServer>,
) {
    for (e, hand, name) in spheres.iter() {
        if name.as_str().contains("Plane") {
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
            let custom_material = custom_materials.add(CustomMaterial {
                color: Color::BLUE,
                color_texture: Some(asset_server.load("shaders/leaf.png")),
                alpha_mode: AlphaMode::Blend,
                time: 0.5,
            });
            commands.entity(e).remove::<Handle<StandardMaterial>>();
            commands.entity(e).insert(custom_material);
            commands.entity(e).insert(Inserted);
        }
    }
}
