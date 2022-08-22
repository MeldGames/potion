use bevy_egui::EguiPlugin;
use bevy_inspector_egui_rapier::InspectableRapierPlugin;
use bevy_rapier3d::prelude::*;
use iyes_loopless::prelude::*;

use potion::diagnostics::DiagnosticsEguiPlugin;
use potion::network::NetworkPlugin;
use potion::physics::PhysicsPlugin;
use potion::player::{PlayerInputPlugin, PlayerPlugin};

use bevy::prelude::*;
use bevy_inspector_egui::InspectableRegistry;

fn main() {
    let mut app = App::new();
    app.insert_resource(sabi::Client);
    potion::setup_app(&mut app);
    app.add_plugin(PlayerInputPlugin);
    /*
    app.add_plugin(NetworkPlugin)
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.3)))
        .insert_resource(WindowDescriptor {
            title: "Brewification".to_string(),
            width: 800.,
            height: 600.,
            cursor_visible: false,
            cursor_locked: false,
            present_mode: bevy::window::PresentMode::Immediate,
            ..Default::default()
        })
        .add_plugin(PlayerPlugin)
        .add_plugin(potion::physics::PhysicsPlugin)
        .add_plugin(RapierDebugRenderPlugin {
            depth_test: true,
            style: Default::default(),
            mode: Default::default(),
        })
        .add_plugin(PlayerInputPlugin)
        .add_plugin(bevy::diagnostic::DiagnosticsPlugin)
        .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        .add_plugin(potion::diagnostics::DiagnosticsEguiPlugin);*/
    app.add_startup_system(setup);

    app.run();
}

fn setup(
    mut commands: Commands,
    _meshes: ResMut<Assets<Mesh>>,
    _materials: ResMut<Assets<StandardMaterial>>,
    _assets: Res<AssetServer>,
) {
    println!("yessir");
}
