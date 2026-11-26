//! Spatial audio tab — toggle binaural rendering and select layout.

use crate::ui::StropApp;

pub fn show(ui: &mut egui::Ui, app: &mut StropApp) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.heading("Spatial Audio");
        ui.add_space(8.0);
        ui.label("Realtime binaural rendering of system audio.");
        ui.add_space(8.0);

        // Toggle spatial audio on/off.
        let prev = app.spatial_enabled;
        ui.checkbox(&mut app.spatial_enabled, "Spatial audio enabled");
        if app.spatial_enabled != prev {
            app.toggle_spatial();
        }

        ui.add_space(8.0);

        // Layout selector.
        ui.label("Speaker layout:");
        let layouts = ["7.1", "7.1.4"];
        let prev_layout = app.spatial_layout;
        egui::ComboBox::from_id_salt("layout")
            .selected_text(layouts[app.spatial_layout])
            .show_ui(ui, |ui| {
                for (i, l) in layouts.iter().enumerate() {
                    ui.selectable_value(&mut app.spatial_layout, i, *l);
                }
            });
        if app.spatial_layout != prev_layout && app.spatial_enabled {
            app.toggle_spatial();
            app.toggle_spatial();
        }

        ui.add_space(16.0);
        ui.separator();
        ui.add_space(8.0);

        // Status.
        if app.spatial_enabled {
            ui.label("✅ Spatial audio is active");
            ui.label("Capturing system audio → binaural → headset");
        } else {
            ui.label("⏸ Spatial audio is off");
        }

        #[cfg(not(target_os = "macos"))]
        {
            ui.add_space(8.0);
            ui.label("ℹ️ Spatial audio requires macOS 14.2+");
            ui.label("(CoreAudio process tap API)");
        }
    });
}
