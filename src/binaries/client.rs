use potion::prelude::*;

fn main() {
    let mut app = App::new();
    //app.insert_resource(sabi::Client);
    app.add_plugin(PotionCellarPlugin);
    app.add_plugin(PlayerInputPlugin);

    app.run();
}
