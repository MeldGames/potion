
use bevy::prelude::*;

use potion::player::prelude::{mouse_lock, toggle_mouse_lock, LockToggle, MouseState};
use potion::player::window_focused;
use potion::setup_map;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new();
    potion::setup_app(&mut app);

    app.insert_resource(LockToggle::default());

    /*
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
    */
    /*
    app.add_system(
        toggle_mouse_lock
            .run_if(window_focused)
    )
    .add_system(mouse_lock.run_if(window_focused).label("toggle_mouse_lock"));
 */

    app.add_startup_system(setup_map);

/*
    #[cfg(feature = "public")]
    let ip = sabi::protocol::public_ip()?;
    #[cfg(not(feature = "public"))]
    let ip = sabi::protocol::localhost_ip();

    let new_server: bevy_renet::renet::RenetServer =
        sabi::protocol::new_renet_server(ip, None, sabi::protocol::PORT)
            .expect("could not make new server");
    app.insert_resource(new_server);
 */
    app.run();
    Ok(())
}
