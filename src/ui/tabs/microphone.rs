//! Microphone tab — mic mute, sidetone, mic EQ presets.

use crate::ui::{HidCommand, StropApp};
use egui::ComboBox;

pub fn show(ui: &mut egui::Ui, app: &mut StropApp) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.heading("Microphone");
        ui.add_space(8.0);

        // --- Mic mute ---
        let prev = app.mic_muted;
        ui.checkbox(&mut app.mic_muted, "Muted");
        if app.mic_muted != prev {
            app.send_cmd(HidCommand::SetMicMuted(app.mic_muted));
        }

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        // --- Sidetone ---
        ui.label("Sidetone (mic monitoring)");
        let prev = app.sidetone as u8;
        ui.add(egui::Slider::new(&mut app.sidetone, 0.0..=15.0)
            .text("Level")
            .clamping(egui::SliderClamping::Always));
        if (app.sidetone as u8) != prev {
            app.send_cmd(HidCommand::SetSidetone(app.sidetone as u8));
        }

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        // --- Mic EQ preset ---
        ui.label("Mic EQ Preset");
        let mic_presets = ["Default", "Flat", "Broadcast", "Esports"];
        let prev = app.mic_preset_idx;
        ComboBox::from_id_salt("mic_preset")
            .selected_text(mic_presets[app.mic_preset_idx])
            .show_ui(ui, |ui| {
                for (i, p) in mic_presets.iter().enumerate() {
                    ui.selectable_value(&mut app.mic_preset_idx, i, *p);
                }
            });
        if app.mic_preset_idx != prev {
            let index = 0x20 + app.mic_preset_idx as u8;
            app.send_cmd(HidCommand::SetMicPreset(index));
        }

        ui.add_space(8.0);

        ui.label("Mic Equalizer");
        let freq_labels = [
            "31Hz", "63Hz", "125Hz", "250Hz", "500Hz",
            "1kHz", "2kHz", "4kHz", "8kHz", "16kHz",
        ];
        ui.horizontal(|ui| {
            for (i, &label) in freq_labels.iter().enumerate() {
                ui.vertical(|ui| {
                    ui.add(
                        egui::Slider::new(&mut app.state.mic_bands[i], -6.0..=6.0)
                            .vertical()
                            .step_by(1.0)
                            .show_value(false)
                            .clamping(egui::SliderClamping::Always),
                    );
                    ui.label(label);
                });
            }
        });
    });
}
