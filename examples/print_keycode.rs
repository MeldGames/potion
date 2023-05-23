use bevy::{prelude::*, window::WindowPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: (100.0, 100.0).into(),
                ..default()
            }),
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
