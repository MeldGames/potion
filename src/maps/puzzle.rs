use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub struct SetupPlugin;
impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(super::base_test::SetupPlugin);
        app.add_startup_system(setup);
    }
}

pub fn cube(commands: &mut Commands, transform: Transform) -> Entity {
    commands
        .spawn(TransformBundle::from_transform(transform))
        .insert((
            RigidBody::Fixed,
            Collider::cuboid(0.5, 0.5, 0.5),
            Name::new("Wall 1"),
            crate::physics::TERRAIN_GROUPING,
        ))
        .id()
}

pub fn setup(mut commands: Commands) {
    cube(
        &mut commands,
        Transform {
            translation: Vec3::new(-10., 0., 3.),
            scale: Vec3::new(15.0, 10.0, 0.5),
            ..default()
        },
    );

    cube(
        &mut commands,
        Transform {
            translation: Vec3::new(-10., 0., -3.),
            scale: Vec3::new(15.0, 10.0, 0.5),
            ..default()
        },
    );

    cube(
        &mut commands,
        Transform {
            translation: Vec3::new(-17.5, 0., 0.),
            scale: Vec3::new(0.5, 10.0, 6.0),
            ..default()
        },
    );
}
