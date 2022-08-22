//use bevy_editor_pls::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_fly_camera::FlyCamera;
use bevy_inspector_egui::InspectableRegistry;
use bevy_mod_outline::{Outline, OutlinePlugin};
use bevy_rapier3d::prelude::*;
use iyes_loopless::prelude::*;
use potion::network::NetworkPlugin;
use potion::player::{LockToggle, MouseState, PlayerPlugin};

use bevy::prelude::*;
//use bevy_fly_camera::{camera_movement_system, mouse_motion_system, FlyCamera};

use potion::player::{mouse_lock, toggle_mouse_lock, window_focused};
use wgpu_types::PrimitiveTopology;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new();
    app.insert_resource(sabi::Server);
    potion::setup_app(&mut app);
    app.add_plugin(OutlinePlugin);
    //app.insert_resource(bevy::ecs::schedule::ReportExecutionOrderAmbiguities);

    app.add_loopless_state(MouseState::Locked);
    app.insert_resource(LockToggle::default());

    app.add_system(
        bevy_fly_camera::camera_movement_system
            .run_if(window_focused)
            .label("player_fly_movement"),
    )
    .add_system(
        bevy_fly_camera::mouse_motion_system
            .run_in_state(MouseState::Locked)
            .run_if(window_focused)
            .label("player_mouse_input"),
    );
    app.add_system(outline_meshes);
    app.add_system(
        toggle_mouse_lock
            .run_if(window_focused)
            .label("toggle_mouse_lock"),
    )
    .add_system(mouse_lock.run_if(window_focused).label("toggle_mouse_lock"));

    app.add_startup_system(setup_camera);

    #[cfg(feature = "public")]
    let ip = sabi::protocol::public_ip()?;
    #[cfg(not(feature = "public"))]
    let ip = sabi::protocol::localhost_ip();

    let new_server: bevy_renet::renet::RenetServer =
        sabi::protocol::new_renet_server(ip, None, sabi::protocol::PORT)
            .expect("could not make new server");
    app.insert_resource(new_server);

    app.run();

    Ok(())
}

fn setup_camera(mut commands: Commands, _asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(0., 12., 10.))
                .looking_at(Vec3::new(0.0, 0.3, 0.0), Vec3::Y),
            ..Default::default()
        })
        .insert(FlyCamera::default());

    commands.spawn_bundle(SceneBundle {
        scene: _asset_server.load("models/cauldron.glb#Scene0"),
        ..default()
    });
}

fn outline_meshes(
    mut commands: Commands,
    mut outlines: ResMut<Assets<Outline>>,
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<(Entity, &Handle<Mesh>), (With<Handle<Mesh>>, Without<Handle<Outline>>)>,
) {
    for (entity, mesh) in &query {
        if let Some(mesh) = meshes.get(mesh) {
            if mesh.contains_attribute(Mesh::ATTRIBUTE_NORMAL) {
                commands.entity(entity).insert(outlines.add(Outline {
                    colour: Color::rgba(0.0, 0.0, 0.0, 0.8),
                    width: 5.0,
                }));
            }
        }
    }
    /*
       commands
           .spawn_bundle(PbrBundle {
               mesh: meshes.add(cube_mesh),
               material: materials.add(Color::rgb(0.1, 0.1, 0.9).into()),
               transform: Transform::from_xyz(0.0, 1.0, 0.0),
               ..default()
           })
           .insert_bundle(OutlineBundle {
               outline: Outline {
                   visible: true,
                   colour: Color::rgba(0.0, 1.0, 0.0, 1.0),
                   width: 25.0,
               },
               ..default()
           })
           .insert(Wobbles);

       // Add torus using the regular surface normals for outlining
       commands
           .spawn_bundle(PbrBundle {
               mesh: meshes.add(Mesh::from(Torus {
                   radius: 0.3,
                   ring_radius: 0.1,
                   subdivisions_segments: 20,
                   subdivisions_sides: 10,
               })),
               material: materials.add(Color::rgb(0.9, 0.1, 0.1).into()),
               transform: Transform::from_xyz(0.0, 1.2, 2.0)
                   .with_rotation(Quat::from_rotation_x(0.5 * PI)),
               ..default()
           })
           .insert_bundle(OutlineBundle {
               outline: Outline {
                   visible: true,
                   colour: Color::rgba(1.0, 0.0, 1.0, 0.3),
                   width: 15.0,
               },
               ..default()
           })
           .insert(Orbits);

       // Add plane, light source, and camera
       commands.spawn_bundle(PbrBundle {
           mesh: meshes.add(Mesh::from(bevy::prelude::shape::Plane { size: 5.0 })),
           material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
           ..default()
       });
       commands.spawn_bundle(PointLightBundle {
           point_light: PointLight {
               intensity: 1500.0,
               shadows_enabled: true,
               ..default()
           },
           transform: Transform::from_xyz(4.0, 8.0, 4.0),
           ..default()
       });
       commands.spawn_bundle(Camera3dBundle {
           transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
           ..default()
       });
    */
}
