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
};
use bevy_mod_outline::{OutlineStencil, OutlineVolume};
use bevy_rapier3d::prelude::*;
use bevy_shader_utils::ShaderUtilsPlugin;

pub struct TreesPlugin;
impl Plugin for TreesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<LeafMaterial>::default())
            .add_plugin(ShaderUtilsPlugin)
            .add_system_to_stage(CoreStage::PostUpdate, mod_scene);
    }
}

#[derive(Component)]
struct Inserted;

pub fn spawn_trees(
    commands: &mut Commands,
    asset_server: &AssetServer,
    _meshes: &mut Assets<Mesh>,
) {
    let tree_positions = vec![
        Vec3::new(15., 0., -2.),
        Vec3::new(-15., -2., -30.),
        Vec3::new(-6., -2.5, 100.),
        Vec3::new(43., 0., 20.),
    ];
    for i in tree_positions {
        let _tree = commands
            .spawn(SceneBundle {
                scene: asset_server.load("models/tree.gltf#Scene0"),
                transform: Transform {
                    translation: i.clone(),
                    scale: Vec3::splat(1.),
                    ..default()
                },
                ..default()
            })
            .insert((
                ColliderMassProperties::Density(5.0),
                RigidBody::Fixed,
                Name::new("Tree"),
                crate::physics::TERRAIN_GROUPING,
            ))
            .id();
    }
}

/// The Material trait is very configurable, but comes with sensible defaults for all methods.
/// You only need to implement functions for features that need non-default behavior. See the Material api docs for details!
impl Material for LeafMaterial {
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
                color: Color::GREEN,
                color_texture: Some(asset_server.load("shaders/leaves.png")),
                alpha_mode: AlphaMode::Blend,
            });
            commands.entity(e).remove::<Handle<StandardMaterial>>();
            commands.entity(e).remove::<OutlineStencil>();
            commands.entity(e).remove::<OutlineVolume>();
            commands
                .entity(e)
                .insert((custom_material, NotShadowReceiver, Inserted));
        }
    }
}
