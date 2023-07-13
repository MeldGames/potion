use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef},
    window::CursorGrabMode,
};

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
    .add_plugins(MaterialPlugin::<GlowyMaterial>::default())
    .add_startup_system(setup);

    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut glowys: ResMut<Assets<GlowyMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    assets: Res<AssetServer>,
) {
    let env_texture = assets.load("stone_alley_02_1k.hdr");
    let material = glowys.add(GlowyMaterial {
        env_texture: Some(env_texture),
    });

    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(5.0, 2.0, 5.0).looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
        ..default()
    });

    // plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 100.0 })),
        material: materials.add(Color::rgb(0.1, 0.1, 0.1).into()),
        ..default()
    });

    // orb locations
    let locations = [
        Vec3::new(-0.15, 1.0, -2.0),
        Vec3::new(1.7, 1.07, -0.61),
        Vec3::new(0.21, 1.05, 1.99),
        Vec3::new(-2.16, 1.0, 0.01),
        Vec3::new(-2.2, 1.0, 2.13),
        Vec3::new(-1.06, 2.04, 1.02),
        Vec3::new(1.94, 1.02, 1.16),
        Vec3::new(0.91, 2.47, 0.83),
        Vec3::new(0.46, 2.48, -0.81),
        Vec3::new(-2.05, 0.93, -1.92),
        Vec3::new(-1.38, 2.46, -0.91),
        Vec3::new(-0.22, 3.48, 0.18),
    ];

    for location in locations {
        // spawn orbs
        commands
            .spawn(MaterialMeshBundle {
                mesh: meshes.add(Mesh::from(shape::UVSphere {
                    radius: 1.0,
                    ..default()
                })),
                transform: Transform::from_translation(location),
                material: material.clone(),
                ..default()
            })
            .add_children(|parent| {
                // child light
                parent.spawn(PointLightBundle {
                    point_light: PointLight {
                        intensity: 10000.0,
                        radius: 1.0,
                        color: Color::rgb(0.1, 0.1, 0.5),
                        ..default()
                    },
                    ..default()
                });
            });
    }
}

#[derive(AsBindGroup, Debug, Clone, TypeUuid)]
#[uuid = "717f64fe-6844-4822-8926-e0ed374294c8"]
pub struct GlowyMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub env_texture: Option<Handle<Image>>,
}

impl Material for GlowyMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/glowy.wgsl".into()
    }
}
