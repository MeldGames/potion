
use crate::prelude::*;
use bevy::{
    ecs::{
        entity::Entities,
        system::{SystemParam, SystemState},
    },
};
use bevy_editor_pls::editor_window::{EditorWindow, EditorWindowContext};
use bevy_renet::renet::{RenetClient, RenetServer};
//use renet_visualizer::{RenetClientVisualizer, RenetServerVisualizer, RenetVisualizerStyle};
use sabi::{prelude::ServerEntities, protocol::update::UpdateMessages, tick::NetworkTick};

pub const DATA_POINTS: usize = 100;

#[derive(Debug, Clone)]
pub struct NetworkWindow {
    pub client_ip: String,
    pub client_port: u16,
    pub client_error: Option<String>,

    pub server_ip: String,
    pub server_port: u16,
    pub server_error: Option<String>,
}

impl Default for NetworkWindow {
    fn default() -> Self {
        Self {
            #[cfg(not(feature = "public"))]
            client_ip: "127.0.0.1".to_owned(),
            #[cfg(feature = "public")]
            client_ip: "spite.aceeri.com".to_owned(),

            client_port: sabi::protocol::PORT,
            client_error: None,

            #[cfg(not(feature = "public"))]
            server_ip: "127.0.0.1".to_owned(),
            #[cfg(feature = "public")]
            server_ip: sabi::protocol::public_ip().expect("public ip to bind to"),
            server_port: sabi::protocol::PORT,
            server_error: None,
        }
    }
}

impl EditorWindow for NetworkWindow {
    type State = (
        Self,
        Option<SystemState<NetworkInfoQuery<'static, 'static>>>,
    );
    const NAME: &'static str = "Network Info";

    fn ui(world: &mut World, mut cx: EditorWindowContext, ui: &mut egui::Ui) {
        let (ref mut state, ref mut system_state) = cx.state_mut::<Self>().unwrap();
        if let None = system_state {
            *system_state = Some(SystemState::from_world(world));
        }

        let system_state = system_state.as_mut().unwrap();
        {
            let query = system_state.get_mut(world);
            network_info(ui, state, query);
        }
        system_state.apply(world);
    }
}

/*
pub fn update_network_stats(
    mut commands: Commands,
    client: Option<Res<RenetClient>>,
    client_visualizer: Option<ResMut<RenetClientVisualizer<DATA_POINTS>>>,

    server: Option<Res<RenetServer>>,
    server_visualizer: Option<ResMut<RenetServerVisualizer<DATA_POINTS>>>,
) {
    if let Some(ref client) = client {
        match client_visualizer {
            Some(mut visualizer) => {
                visualizer.add_network_info(client.network_info());
            }
            None => {
                commands.insert_resource(RenetClientVisualizer::<DATA_POINTS>::new(
                    RenetVisualizerStyle::default(),
                ));
            }
        };
    }

    if let Some(ref server) = server {
        match server_visualizer {
            Some(mut visualizer) => {
                visualizer.update(&**server);
            }
            None => {
                commands.insert_resource(RenetServerVisualizer::<DATA_POINTS>::new(
                    RenetVisualizerStyle::default(),
                ));
            }
        };
    }
}

pub fn update_connected_clients(
    mut events: EventReader<ServerEvent>,
    mut visualizer: ResMut<RenetServerVisualizer<DATA_POINTS>>,
) {
    for event in events.iter() {
        match event {
            ServerEvent::ClientConnected(client_id, _user_data) => {
                visualizer.add_client(*client_id);
            }
            ServerEvent::ClientDisconnected(client_id) => {
                visualizer.remove_client(*client_id);
            }
            _ => {}
        }
    }
}
 */

#[derive(SystemParam)]
pub struct NetworkInfoQuery<'w, 's> {
    commands: Commands<'w, 's>,
    entities: &'w Entities,

    server_entities: ResMut<'w, ServerEntities>,

    tick: Option<Res<'w, NetworkTick>>,
    updates: Option<Res<'w, UpdateMessages>>,

    is_client: Option<Res<'w, sabi::Client>>,
    client: Option<ResMut<'w, RenetClient>>,
    //client_visualizer: Option<ResMut<'w, RenetClientVisualizer<DATA_POINTS>>>,
    is_server: Option<Res<'w, sabi::Server>>,
    server: Option<ResMut<'w, RenetServer>>,
    //server_visualizer: Option<ResMut<'w, RenetServerVisualizer<DATA_POINTS>>>,
}

pub fn network_info(ui: &mut egui::Ui, ui_state: &mut NetworkWindow, query: NetworkInfoQuery) {
    let NetworkInfoQuery {
        mut commands,
        entities,

        mut server_entities,

        tick,
        updates,

        is_client,
        mut client,
        //client_visualizer,
        is_server,
        mut server,
        //server_visualizer,
    } = query;

    egui::ScrollArea::vertical().show(ui, |ui| {
        if let Some(tick) = tick {
            ui.label(format!("Client tick: {:?}", tick));
            if let Some(updates) = updates {
                ui.label(format!("Server tick: {:?}", updates.latest()));
                ui.label(format!(
                    "Diff: {:?}",
                    updates
                        .latest()
                        .and_then(|latest| Some(tick.tick() as i64 - latest.tick() as i64))
                ));
            }
        }

        if let Some(_) = is_client {
            ui.heading("Client");
            ui.horizontal(|ui| {
                ui.label("IP");
                ui.add_sized(
                    [125.0, 16.0],
                    egui::TextEdit::singleline(&mut ui_state.client_ip)
                        .hint_text("ip address or domain name (e.g. 127.0.0.1)"),
                );
                ui.add(egui::DragValue::new(&mut ui_state.client_port));

                if let Some(ref mut client) = client {
                    if ui.button("disconnect").clicked() {
                        client.disconnect();
                        server_entities.disconnect(entities, &mut commands);
                        commands.remove_resource::<RenetClient>();
                    }
                } else {
                    if ui.button("connect").clicked() {
                        match sabi::protocol::new_renet_client(
                            &ui_state.client_ip,
                            ui_state.client_port,
                        ) {
                            Ok(new_client) => {
                                let new_client: RenetClient = new_client;
                                ui_state.client_error = None;
                                commands.insert_resource(new_client);
                            }
                            Err(err) => ui_state.client_error = Some(err.to_string()),
                        }
                    }
                }
            });

            if let Some(error) = &ui_state.client_error {
                ui.colored_label(egui::Color32::RED, error);
            }

            /*
            if let Some(visualizer) = client_visualizer {
                ui.collapsing("Client Stats", |ui| visualizer.draw_all(ui));
            }
            */
        }

        if let Some(_) = is_server {
            ui.heading("Server");
            ui.horizontal(|ui| {
                ui.label("IP");
                ui.add_sized(
                    [125.0, 16.0],
                    egui::TextEdit::singleline(&mut ui_state.server_ip)
                        .hint_text("ip address or domain name (e.g. 127.0.0.1)"),
                );
                ui.add(egui::DragValue::new(&mut ui_state.server_port));

                if let Some(ref mut _server) = server {
                    if ui.button("stop").clicked() {
                        commands.remove_resource::<RenetServer>();
                    }
                } else {
                    if ui.button("host").clicked() {
                        match sabi::protocol::new_renet_server(
                            &ui_state.server_ip,
                            None,
                            ui_state.server_port,
                        ) {
                            Ok(new_server) => {
                                let new_server: RenetServer = new_server;
                                ui_state.server_error = None;
                                commands.insert_resource(new_server);
                            }
                            Err(err) => {
                                ui_state.server_error = Some(err.to_string());
                            }
                        }
                    }
                }
            });

            if let Some(error) = &ui_state.server_error {
                ui.colored_label(egui::Color32::RED, error);
            }

            /*
            if let Some(visualizer) = server_visualizer {
                egui::CollapsingHeader::new("Server Stats")
                    .default_open(true)
                    .show(ui, |ui| {
                        if let Some(server) = server {
                            for client_id in server.clients_id() {
                                ui.collapsing(format!("Client {}", client_id), |ui| {
                                    visualizer.draw_client_metrics(client_id, ui);
                                });
                            }
                        }
                    });
            }
            */
        }
    });
}
