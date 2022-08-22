use std::time::Duration;

use bevy::prelude::*;
use bevy_renet::renet::{RenetClient, RenetServer, ServerEvent};
use iyes_loopless::prelude::*;

use renet_visualizer::RenetServerVisualizer;
use sabi::{
    prelude::*,
    protocol::{client_connected, input::QueuedInputs, ServerChannel},
};

use bevy_rapier3d::prelude::*;

use crate::player::{FromCamera, Neck, PlayerBundle, PlayerCam, Reticle, Speed};

use crate::player::{Player, PlayerInput};

pub mod ui;

pub const PORT: u16 = 42069;

pub const TICK_RATE: Duration = tick_hz(100);

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        use sabi::stage::NetworkSimulationAppExt;

        app.add_system(
            server_update_system
                .run_if_resource_exists::<RenetServer>()
                .label("server_update_system"),
        );

        app.add_plugin(SabiPlugin::<PlayerInput> {
            tick_rate: TICK_RATE,
            ..Default::default()
        });

        app.insert_resource(QueuedInputs::<PlayerInput>::new());
        app.insert_resource(ui::NetworkUiState::default());
        app.add_meta_network_system(ui::update_network_stats);
        app.add_system(ui::display_network_stats);
        app.add_system(
            ui::update_connected_clients
                .run_if_resource_exists::<RenetServer>()
                .run_if_resource_exists::<RenetServerVisualizer<{ ui::DATA_POINTS }>>(),
        );
        app.add_meta_network_system(
            client_sync_players
                .run_if_resource_exists::<RenetClient>()
                .run_if(client_connected),
        );
    }
}

fn server_update_system(
    mut server_events: EventReader<ServerEvent>,
    mut commands: Commands,
    mut lobby: ResMut<Lobby>,
    mut server: ResMut<RenetServer>,
) {
    for event in server_events.iter() {
        match event {
            ServerEvent::ClientConnected(id, _) => {
                info!("player {} connected.", id);
                // Spawn player cube
                let player_entity = commands
                    .spawn()
                    .insert(Collider::capsule(Vec3::ZERO, Vec3::Y, 0.5))
                    .insert(RigidBody::Dynamic)
                    .insert(LockedAxes::ROTATION_LOCKED)
                    .insert(crate::physics::PLAYER_GROUPING)
                    .insert(Velocity::default())
                    //.insert(Ccd::enabled())
                    .insert(Speed::default())
                    .insert(Transform::default())
                    .insert(GlobalTransform::default())
                    .insert(PlayerInput::default())
                    .insert(Player { id: *id })
                    .insert(Name::new(format!("Player {}", id.to_string())))
                    .insert(Owned)
                    //.insert(Loader::<Mesh>::new("scenes/gltfs/boi.glb#Mesh0/Primitive0"))
                    .insert(Friction {
                        coefficient: 0.0,
                        ..Default::default()
                    })
                    .id();

                // We could send an InitState with all the players id and positions for the client
                // but this is easier to do.
                for (existing_id, existing_entity) in lobby.players.iter() {
                    let message = bincode::serialize(&ServerMessage::PlayerConnected {
                        id: *existing_id,
                        entity: (*existing_entity).into(),
                    })
                    .unwrap();
                    server.send_message(*id, ServerChannel::Message.id(), message);
                }

                lobby.players.insert(*id, player_entity);

                let message = bincode::serialize(&ServerMessage::PlayerConnected {
                    id: *id,
                    entity: player_entity.into(),
                })
                .unwrap();
                server.broadcast_message(ServerChannel::Message.id(), message);

                let message = bincode::serialize(&ServerMessage::AssignOwnership {
                    entity: player_entity.into(),
                })
                .unwrap();
                server.send_message(*id, ServerChannel::Message.id(), message);

                let message = bincode::serialize(&ServerMessage::SetPlayer {
                    id: *id,
                    entity: player_entity.into(),
                })
                .unwrap();
                server.send_message(*id, ServerChannel::Message.id(), message);
            }
            ServerEvent::ClientDisconnected(id) => {
                info!("player {} disconnected.", id);
                if let Some(player_entity) = lobby.players.remove(id) {
                    commands.entity(player_entity).despawn();
                }

                let message =
                    bincode::serialize(&ServerMessage::PlayerDisconnected { id: *id }).unwrap();
                server.broadcast_message(ServerChannel::Message.id(), message);
            }
        }
    }
}

pub fn client_sync_players(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    mut server_entities: ResMut<ServerEntities>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut client: ResMut<RenetClient>,
    mut lobby: ResMut<Lobby>,
    _player_transforms: Query<(&mut Transform, &Children, Option<&Owned>), With<Player>>,
    _neck_transforms: Query<&mut Transform, (With<Neck>, Without<Player>)>,
) {
    while let Some(message) = client.receive_message(ServerChannel::Message.id()) {
        let server_message = bincode::deserialize(&message).unwrap();
        match server_message {
            ServerMessage::PlayerConnected {
                id,
                entity: server_entity,
            } => {
                info!("player {} connected.", id);
                let entity = server_entities.spawn_or_get(&mut commands, server_entity);
                lobby.players.insert(id, entity);
            }
            ServerMessage::PlayerDisconnected { id } => {
                info!("player {} disconnected.", id);
            }
            ServerMessage::SetPlayer {
                id,
                entity: server_entity,
            } => {
                let player_entity = server_entities.spawn_or_get(&mut commands, server_entity);

                // TODO: Remove all this aside from local inputs and have it all sync from the server.
                //
                // We will need to make all of these components implement `Replicate` first.
                commands.entity(player_entity).insert_bundle(PlayerBundle {
                    transform: Transform {
                        translation: Vec3::new(0.0, 0.5, 0.0),
                        rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
                        ..Default::default()
                    },
                    global_transform: GlobalTransform::identity(),
                    player_component: Player { id: id },
                    speed: Speed::default(),
                    name: Name::new("Player"),
                });

                let reticle_cube =
                    meshes.add(Mesh::from(bevy::render::mesh::shape::Cube { size: 0.2 }));

                let camera = commands
                    .spawn_bundle(Camera3dBundle {
                        transform: Transform::from_translation(Vec3::new(0., 0., 4.))
                            .looking_at(Vec3::ZERO, Vec3::Y),
                        ..Default::default()
                    })
                    .insert(PlayerCam)
                    .insert(Name::new("Player Camera"))
                    .id();

                let reticle = commands
                    .spawn_bundle((
                        Transform {
                            translation: Vec3::new(0., 0., 0.),
                            ..Default::default()
                        },
                        GlobalTransform::identity(),
                        Reticle {
                            max_distance: 6.0,
                            from_height: 4.0,
                        },
                        Name::new("Reticle"),
                        FromCamera(camera),
                    ))
                    .id();

                let neck = commands
                    .spawn_bundle((
                        Transform {
                            translation: Vec3::new(0., 1., 0.),
                            ..Default::default()
                        },
                        GlobalTransform::identity(),
                        Neck,
                        Name::new("Neck"),
                    ))
                    .id();

                commands.entity(neck).push_children(&[camera]);

                let mut material = StandardMaterial::default();
                material.base_color = Color::hex("800000").unwrap().into();
                material.perceptual_roughness = 0.97;
                material.reflectance = 0.0;
                let red = materials.add(material);

                let ret_mesh = commands
                    .spawn_bundle(PbrBundle {
                        material: red.clone(),
                        mesh: reticle_cube.clone(),
                        ..Default::default()
                    })
                    .id();

                commands.entity(reticle).push_children(&[ret_mesh]);

                commands
                    .entity(player_entity)
                    .insert(PlayerInput::default())
                    .push_children(&[neck, reticle]);
                commands.spawn_bundle(SceneBundle {
                    scene: _asset_server.load("models/cauldron.glb#Scene0"),
                    ..default()
                });

            }
            ServerMessage::AssignOwnership {
                entity: server_entity,
            } => {
                let entity = server_entities.spawn_or_get(&mut commands, server_entity);
                commands.entity(entity).insert(Owned);
                println!(
                    "Ownership assigned for entity {:?} (server id {:?})",
                    entity, server_entity
                );
            }
        }
    }
}
