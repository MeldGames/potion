use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey, NotShadowReceiver},
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::MeshVertexBufferLayout,
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
        },
    },
};
use bevy_rapier3d::prelude::*;

pub struct TreesPlugin;
impl Plugin for TreesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<LeafMaterial>::default())
            .add_plugin(MaterialPlugin::<BarkMaterial>::default())
            .add_system(mod_scene);
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
                scene: asset_server.load("models/tree3.gltf#Scene0"),
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
        "shaders/leaf_material2.wgsl".into()
    }
    fn vertex_shader() -> ShaderRef {
        "shaders/leaf_material2.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Opaque
    }
    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        //descriptor.primitive.cull_mode = None;
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
    alpha_mode: AlphaMode,
}

impl Material for BarkMaterial {
    // fn fragment_shader() -> ShaderRef {
    //     "shaders/bark_material.wgsl".into()
    // }
    // fn vertex_shader() -> ShaderRef {
    //     "shaders/bark_material.wgsl".into()
    // }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        //descriptor.primitive.cull_mode = None;
        Ok(())
    }
}

// This is the struct that will be passed to your shader
#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "dac0f52c-a570-11ed-afa1-0242ac120002"]
pub struct BarkMaterial {
    #[uniform(0)]
    color: Color,
}

fn mod_scene(
    mut commands: Commands,
    spheres: Query<(Entity, &Handle<Mesh>, &Name), Without<Inserted>>,
    meshes: ResMut<Assets<Mesh>>,
    mut custom_materials: ResMut<Assets<LeafMaterial>>,
    mut bark_materials: ResMut<Assets<BarkMaterial>>,
    asset_server: Res<AssetServer>,
) {
    for (e, _hand, name) in spheres.iter() {
        if name.as_str().contains("leaves") {
            let custom_material = custom_materials.add(LeafMaterial {
                color: Color::YELLOW_GREEN,
                color_texture: Some(asset_server.load("shaders/leaves.png")),
                alpha_mode: AlphaMode::Mask(1.0),
            });
            commands.entity(e).remove::<Handle<StandardMaterial>>();
            commands
                .entity(e)
                .insert((custom_material, NotShadowReceiver, Inserted));
        }
        if name.as_str().contains("bark") {
            let custom_material = bark_materials.add(BarkMaterial {
                color: Color::rgb(0.3, 0.2, 0.18),
            });
            commands.entity(e).remove::<Handle<StandardMaterial>>();
            commands
                .entity(e)
                .insert((custom_material, NotShadowReceiver, Inserted));
        }
    }
}
