use bevy_rapier3d::{
    rapier::dynamics::{JointAxis, JointAxesMask},
    prelude::*};
use potion::{
    physics::{ColliderBundle, RigidBodyBundle},
    player::prelude::{PlayerEvent, PlayerInputPlugin},
};

use bevy::prelude::*;

fn main() {
    let mut app = App::new();
    potion::setup_app(&mut app);
    app.add_plugin(PlayerInputPlugin);
    app.add_startup_system(spawn_local_player);
    //app.add_plugin(potion::maps::showcase::SetupPlugin);
    //app.add_plugin(potion::maps::puzzle::SetupPlugin);
    app.add_plugin(potion::maps::base_test::SetupPlugin);
    app.add_startup_system(spawn_multibody);

    app.run();
}

fn spawn_multibody(mut commands: Commands) {
    let mut joint = SphericalJointBuilder::new()
        .local_anchor1(Vec3::new(0.5, 0.0, 0.0))
        .local_anchor2(Vec3::new(-0.5, 0.0, 0.0))
    .build();
    joint.set_contacts_enabled(false);

    let r1 = commands
        .spawn(TransformBundle::default())
        .insert(RigidBodyBundle::default())
        .insert(ColliderBundle { ..default() })
        .insert(Name::new("r1"))
        .id();

    let r2 = commands
        .spawn(TransformBundle::default())
        .insert(RigidBodyBundle::default())
        .insert(ColliderBundle { ..default() })
        .insert(Name::new("r2"))
        .insert(MultibodyJoint::new(r1, joint))
        .id();
}

fn spawn_local_player(mut spawn_player: EventWriter<PlayerEvent>, _asset_server: Res<AssetServer>) {
    spawn_player.send(PlayerEvent::Spawn { id: 1 });
    spawn_player.send(PlayerEvent::SetupLocal { id: 1 });
    info!("spawning new player");
}
