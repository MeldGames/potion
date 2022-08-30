use potion::player::PlayerInputPlugin;

use bevy::prelude::*;

fn main() {
    let mut app = App::new();
    app.insert_resource(sabi::Client);
    potion::setup_app(&mut app);
    app.add_plugin(PlayerInputPlugin);

    app.run();
}
