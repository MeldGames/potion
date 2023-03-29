use bevy::prelude::*;
use bevy_egui::EguiContexts;
use egui::{FontData, FontDefinitions, FontFamily};

pub struct SetupEguiPlugin;

impl Plugin for SetupEguiPlugin {
    fn build(&self, app: &mut App) {
        //app.add_system(setup_fonts)
        //.add_system(setup_style);
    }
}

pub fn setup_fonts(mut egui_context: EguiContexts) {
    let mut fonts = FontDefinitions::default();

    // Install my own font (maybe supporting non-latin characters):
    fonts.font_data.insert(
        "dense".to_owned(),
        FontData::from_static(include_bytes!("../assets/fonts/Exo/Exo2-Light.otf")),
    ); // .ttf and .otf supported

    // Put my font first (highest priority):
    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, "dense".to_owned());

    // Put my font as last fallback for monospace:
    fonts
        .families
        .get_mut(&FontFamily::Monospace)
        .unwrap()
        .push("dense".to_owned());

    egui_context.ctx_mut().set_fonts(fonts);
}

pub fn setup_style(mut egui_context: EguiContexts) {
    let mut visuals = egui::Visuals::dark();
    visuals.popup_shadow.extrusion = 1.0;
    visuals.window_shadow.extrusion = 1.0;
    egui_context.ctx_mut().set_visuals(visuals);
}
