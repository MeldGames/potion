use potion::player::prelude::{PlayerEvent, PlayerInputPlugin};

use bevy::prelude::*;

fn main() {
    let mut app = App::new();
    potion::setup_app(&mut app);
    app.add_plugin(PlayerInputPlugin);
    app.add_startup_system(spawn_local_player);
    app.add_startup_system(potion::maps::showcase::setup);
    //app.add_startup_system(potion::maps::base_test::setup);

    app.run();
}

fn spawn_local_player(mut spawn_player: EventWriter<PlayerEvent>, _asset_server: Res<AssetServer>) {
    spawn_player.send(PlayerEvent::Spawn { id: 1 });
    spawn_player.send(PlayerEvent::SetupLocal { id: 1 });
    info!("spawning new player");
}
