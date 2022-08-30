use bevy::diagnostic::Diagnostics;
use bevy::prelude::*;
use bevy_egui::EguiContext;

pub fn diagnostic_ui(ui: &mut egui::Ui, diagnostics: &Diagnostics) {
    egui::Grid::new("frame time diagnostics").show(ui, |ui| {
        let mut has_diagnostics = false;
        for diagnostic in diagnostics.iter() {
            has_diagnostics = true;
            ui.label(diagnostic.name.as_ref());
            if let Some(average) = diagnostic.average() {
                ui.label(format!("{:.2}", average));
            }
            ui.end_row();
        }

        if !has_diagnostics {
            ui.label(
                r#"No diagnostics found. Possible plugins to add:
            - `FrameTimeDiagnosticsPlugin`
            - `EntityCountDiagnisticsPlugin`
            - `AssetCountDiagnosticsPlugin`
            "#,
            );
        }
    });
}

pub fn display_diagnostics(mut egui_context: ResMut<EguiContext>, diagnostics: Res<Diagnostics>) {
    egui::Window::new("Diagnostics")
        .min_width(0.0)
        .default_width(1.0)
        .show(egui_context.ctx_mut(), |ui| {
            diagnostic_ui(ui, &*diagnostics);
        });
}

pub struct DiagnosticsEguiPlugin;

impl Plugin for DiagnosticsEguiPlugin {
    fn build(&self, _app: &mut App) {
        //app.add_system(display_diagnostics);
    }
}
