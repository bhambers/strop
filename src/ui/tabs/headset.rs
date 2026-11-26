//! Headset tab — EQ sliders, ANC, balance, LED, sleep, low-latency.

use crate::ui::{HidCommand, StropApp};
use egui::ComboBox;

pub fn show(ui: &mut egui::Ui, app: &mut StropApp) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.heading("Equalizer");
        ui.label("Drag a slider to adjust that frequency band (-6 dB to +6 dB).");
        ui.add_space(8.0);

        eq_sliders(ui, app);

        ui.add_space(16.0);
        ui.separator();
        ui.add_space(8.0);

        // --- ANC ---
        ui.heading("Noise Cancelling");
        ui.horizontal(|ui| {
            ui.label("Mode:");
            let anc_modes = ["Off", "Noise cancelling", "Ambient"];
            let prev = app.anc_idx;
            ComboBox::from_id_salt("anc")
                .selected_text(anc_modes[app.anc_idx])
                .show_ui(ui, |ui| {
                    for (i, mode) in anc_modes.iter().enumerate() {
                        ui.selectable_value(&mut app.anc_idx, i, *mode);
                    }
                });
            if app.anc_idx != prev {
                let mode_byte = match app.anc_idx { 0 => 0x00, 1 => 0x01, _ => 0x50 };
                app.send_cmd(HidCommand::SetAnc(mode_byte, app.anc_level as u8));
            }
            if app.anc_idx == 1 {
                let prev_lv = app.anc_level as u8;
                ui.add(egui::Slider::new(&mut app.anc_level, 1.0..=4.0)
                    .text("Level").clamping(egui::SliderClamping::Always));
                if (app.anc_level as u8) != prev_lv {
                    app.send_cmd(HidCommand::SetAnc(0x01, app.anc_level as u8));
                }
            }
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        // --- Dongle LED ---
        ui.heading("Dongle Indicator");
        let led_modes = ["Connection status", "Battery status", "Battery warning only"];
        let prev = app.led_idx;
        ui.horizontal(|ui| {
            ui.label("Mode:");
            ComboBox::from_id_salt("led")
                .selected_text(led_modes[app.led_idx])
                .show_ui(ui, |ui| {
                    for (i, mode) in led_modes.iter().enumerate() {
                        ui.selectable_value(&mut app.led_idx, i, *mode);
                    }
                });
        });
        if app.led_idx != prev {
            app.send_cmd(HidCommand::SetLed(app.led_idx as u8));
        }

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        // --- Sleep timer ---
        ui.heading("Auto Power-Off");
        let sleep_opts = ["15 min", "30 min", "45 min", "60 min"];
        let sleep_vals = [15u8, 30, 45, 60];
        let prev = app.sleep_idx;
        ui.horizontal(|ui| {
            ui.label("Timeout:");
            ComboBox::from_id_salt("sleep")
                .selected_text(sleep_opts[app.sleep_idx])
                .show_ui(ui, |ui| {
                    for (i, opt) in sleep_opts.iter().enumerate() {
                        ui.selectable_value(&mut app.sleep_idx, i, *opt);
                    }
                });
        });
        if app.sleep_idx != prev {
            app.send_cmd(HidCommand::SetAutoPowerOff(sleep_vals[app.sleep_idx]));
        }

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        // --- Low-latency ---
        ui.heading("Ultra-Low Latency");
        let prev = app.ll_enabled;
        ui.checkbox(&mut app.ll_enabled, "Enabled (HyperSpeed)");
        if app.ll_enabled != prev {
            app.send_cmd(HidCommand::SetHyperspeed(app.ll_enabled));
        }
    });
}

fn eq_sliders(ui: &mut egui::Ui, app: &mut StropApp) {
    let freq_labels = [
        "31Hz", "63Hz", "125Hz", "250Hz", "500Hz",
        "1kHz", "2kHz", "4kHz", "8kHz", "16kHz",
    ];

    ui.horizontal(|ui| {
        for (i, &label) in freq_labels.iter().enumerate() {
            ui.vertical(|ui| {
                ui.add_space(4.0);
                let resp = ui.add(
                    egui::Slider::new(&mut app.state.bands[i], -6.0..=6.0)
                        .vertical()
                        .step_by(1.0)
                        .show_value(false)
                        .clamping(egui::SliderClamping::Always),
                );
                ui.label(label);
                ui.label(format!("{:+.0} dB", app.state.bands[i]));

                if resp.drag_stopped() || (resp.changed() && !resp.has_focus()) {
                    app.schedule_band_write(i, app.state.bands[i]);
                }
            });
        }
    });
}
