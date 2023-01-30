use bevy::{prelude::*, window::WindowPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                width: 100.0,
                height: 100.0,
                ..default()
            },
            ..default()
        }))
        .add_system(print_keycode)
        .run();
}

fn print_keycode(mut previous: Local<Vec<KeyCode>>, input: Res<Input<KeyCode>>) {
    let pressed = input.get_pressed().cloned().collect::<Vec<_>>();
    if *previous != pressed {
        info!("pressed: {:?}", pressed);
        *previous = pressed;
    }
}
