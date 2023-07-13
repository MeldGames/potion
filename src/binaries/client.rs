use potion::prelude::*;

fn main() {
    let mut app = App::new();
    //app.insert_resource(sabi::Client);
    app.add_plugins(PotionCellarPlugin);
    app.add_plugins(PlayerInputPlugin);

    app.run();
}
