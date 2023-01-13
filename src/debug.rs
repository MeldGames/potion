use bevy::prelude::*;

#[derive(Debug, Clone, Resource)]
pub struct Debug(bool);

impl Debug {
    pub fn toggle(&mut self) {
        self.0 = !self.0;
    }

    pub fn visible(&self) -> bool {
        self.0
    }
}

#[derive(Debug, Clone, Component)]
pub struct DebugVisible;

pub fn toggle_debug(kb: Res<Input<KeyCode>>, mut debug: ResMut<Debug>) {
    if kb.just_pressed(KeyCode::P) {
        debug.toggle();
    }
}

pub fn debug_visible(debug: Res<Debug>, mut visibility: Query<(&mut Visibility, &DebugVisible)>) {
    if debug.is_changed() {
        for (mut visibility, debug_visible) in &mut visibility {
            visibility.is_visible = debug.visible();
        }
    }
}

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Debug(true));
        app.add_system(toggle_debug);
        app.add_system(debug_visible);
    }
}