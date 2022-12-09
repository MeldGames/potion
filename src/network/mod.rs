use bevy::{ecs::entity::Entities, prelude::*};
use bevy_renet::renet::{RenetClient, RenetServer, ServerEvent};
use iyes_loopless::prelude::*;

use renet_visualizer::RenetServerVisualizer;
use sabi::{
    prelude::*,
    protocol::{client_connected, input::QueuedInputs, ServerChannel},
    stage::NetworkSimulationInfo,
};

use crate::player::prelude::{PlayerEvent, PlayerInput};

pub mod ui;

pub const PORT: u16 = 42069;

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
            tick_rate: crate::TICK_RATE,
            ..Default::default()
        });

        app.insert_resource(QueuedInputs::<PlayerInput>::new());
        let info = NetworkSimulationInfo::new(crate::TICK_RATE);
        //info.slowdown = 3.0;
        app.insert_resource(info);
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
    mut player_events: EventWriter<PlayerEvent>,
) {
    for event in server_events.iter() {
        match event {
            &ServerEvent::ClientConnected(id, _) => {
                info!("player {} connected.", id);
                player_events.send(PlayerEvent::Spawn { id });
            }
            &ServerEvent::ClientDisconnected(id) => {
                info!("player {} disconnected.", id);
                if let Some(player_entity) = lobby.players.remove(&id) {
                    commands.entity(player_entity).despawn();
                }

                let message =
                    bincode::serialize(&ServerMessage::PlayerDisconnected { id: id }).unwrap();
                server.broadcast_message(ServerChannel::Message.id(), message);
            }
        }
    }
}

pub fn client_sync_players(
    mut commands: Commands,
    entities: &Entities,
    mut server_entities: ResMut<ServerEntities>,
    mut client: ResMut<RenetClient>,
    mut lobby: ResMut<Lobby>,
    mut player_events: EventWriter<PlayerEvent>,
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
                if let Some(player) = lobby.players.get(&id) {
                    if entities.contains(*player) {
                        commands.entity(*player).despawn_recursive();
                    }
                }
            }
            ServerMessage::SetPlayer { id } => {
                info!("set up local player: {:?}.", id);
                player_events.send(PlayerEvent::SetupLocal { id })
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
