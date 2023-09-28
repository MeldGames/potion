use crate::physics::{ColliderBundle, RigidBodyBundle};

use crate::prelude::*;

pub struct SetupPlugin;
impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

pub fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube::new(1.0))),
            transform: Transform {
                translation: Vec3::new(0.0, -0.5, 0.0),
                scale: Vec3::new(1000.0, 1.0, 100.0),
                ..default()
            },
            ..default()
        })
        .insert(Name::new("Ground"))
        .insert(RigidBodyBundle {
            rigid_body: RigidBody::Fixed,
            ..default()
        })
        .insert(ColliderBundle {
            collider: Collider::cuboid(0.5, 0.5, 0.5),
            collision_groups: crate::physics::TERRAIN_GROUPING,
            ..default()
        });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            illuminance: 100_000.0,
            ..default()
        },
        transform: Transform {
            rotation: Quat::from_rotation_x(-0.2),
            ..default()
        },
        ..default()
    });

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 5000.0,
            range: 50.0,
            ..default()
        },
        ..default()
    });
}
