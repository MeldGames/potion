use bevy::{
    asset::HandleId,
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
};

#[derive(Resource)]
pub struct TestMaterial {
    pub base: Handle<StandardMaterial>,
    pub indexed: Handle<StandardMaterial>,
}

pub fn setup_test_texture(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let base_material = StandardMaterial {
        perceptual_roughness: 0.95,
        reflectance: 0.05,
        ..default()
    };

    let base = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load("placeholder.png")),
        ..base_material.clone()
    });

    let indexed = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load("placeholder-indexed.png")),
        ..base_material.clone()
    });

    commands.insert_resource(TestMaterial { base, indexed });
}

pub fn replace_blank_textures(
    mut materials: Query<&mut Handle<StandardMaterial>, Changed<Handle<StandardMaterial>>>,
    test_material: Res<TestMaterial>,
) {
    for mut handle in &mut materials {
        match handle.id() {
            HandleId::Id(_, id) if id == 0 => {
                *handle = test_material.base.clone();
            }
            _ => {}
        }
    }
}
