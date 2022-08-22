use potion::player::{PlayerEvent, PlayerInputPlugin};

use bevy::prelude::*;

fn main() {
    let mut app = App::new();
    app.insert_resource(sabi::Local);
    potion::setup_app(&mut app);
    app.add_plugin(PlayerInputPlugin);
    app.add_startup_system(spawn_local_player);

    app.run();
}

fn spawn_local_player(mut spawn_player: EventWriter<PlayerEvent>) {
    println!("yessir");
    info!("spawning new player");
    spawn_player.send(PlayerEvent::Spawn { id: 1 });
    spawn_player.send(PlayerEvent::SetupLocal { id: 1 });
}
