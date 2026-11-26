//! System tray + egui settings window for strop.

mod tabs;

use strop_hid::{self as hid, RAZER_VID, ALL_PIDS};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tray_icon::menu::{Menu, MenuItem, PredefinedMenuItem};
use tray_icon::TrayIconBuilder;

use spatial_impl::SpatialController;

const REFRESH_SECS: u64 = 30;

// ---- Background HID thread protocol ----

pub(crate) enum HidCommand {
    ReadAll,
    SetAnc(u8, u8),
    SetSidetone(u8),
    SetLed(u8),
    SetAutoPowerOff(u8),
    SetHyperspeed(bool),
    SetMicMuted(bool),
    SetMicPreset(u8),
    Quit,
}

/// All device state read in one batch.
#[derive(Default, Clone, Debug)]
pub(crate) struct DeviceState {
    pub battery: Option<u8>,
    pub anc_mode: Option<u8>,
    pub anc_level: Option<u8>,
    pub sidetone: Option<u8>,
    pub led_mode: Option<u8>,
    pub auto_power_off: Option<u8>,
    pub hyperspeed: Option<bool>,
    pub mic_muted: Option<bool>,
    pub mic_preset_idx: Option<u8>,
    pub bands: [f32; 10],
    pub mic_bands: [f32; 10],
}

enum HidResult {
    State(DeviceState),
    WriteAck,
}

// ---- Public API ----

pub fn run_tray() -> Result<(), String> {
    let (cmd_tx, cmd_rx) = mpsc::channel::<HidCommand>();
    let (res_tx, res_rx) = mpsc::channel::<HidResult>();
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    thread::spawn(move || {
        // Cache the last known state so a read failure doesn't clear everything.
        let mut cached = DeviceState::default();

        while r.load(Ordering::SeqCst) {
            match cmd_rx.recv_timeout(Duration::from_millis(500)) {
                Ok(HidCommand::ReadAll) => {
                    if let Ok(state) = read_all_sync() {
                        cached = state;
                    }
                    let _ = res_tx.send(HidResult::State(cached.clone()));
                }
                Ok(HidCommand::SetAnc(mode, level)) => {
                    let _ = write_anc_sync(mode, level);
                    let _ = res_tx.send(HidResult::WriteAck);
                }
                Ok(HidCommand::SetSidetone(level)) => {
                    let _ = write_sidetone_sync(level);
                    let _ = res_tx.send(HidResult::WriteAck);
                }
                Ok(HidCommand::SetLed(mode)) => {
                    let _ = write_led_sync(mode);
                    let _ = res_tx.send(HidResult::WriteAck);
                }
                Ok(HidCommand::SetAutoPowerOff(mins)) => {
                    let _ = write_poweroff_sync(mins);
                    let _ = res_tx.send(HidResult::WriteAck);
                }
                Ok(HidCommand::SetHyperspeed(on)) => {
                    let _ = write_hyperspeed_sync(on);
                    let _ = res_tx.send(HidResult::WriteAck);
                }
                Ok(HidCommand::SetMicMuted(muted)) => {
                    let device = hid::open(RAZER_VID, ALL_PIDS).ok();
                    if let Some(ref d) = device {
                        let _ = d.request(&strop_proto::set_mic_muted(muted), Duration::from_secs(3));
                    }
                    let _ = res_tx.send(HidResult::WriteAck);
                }
                Ok(HidCommand::SetMicPreset(index)) => {
                    let device = hid::open(RAZER_VID, ALL_PIDS).ok();
                    if let Some(ref d) = device {
                        let _ = d.request(&strop_proto::set_mic_preset_index(index), Duration::from_secs(3));
                    }
                    let _ = res_tx.send(HidResult::WriteAck);
                }
                Ok(HidCommand::Quit) | Err(RecvTimeoutError::Disconnected) => break,
                Err(RecvTimeoutError::Timeout) => {}
            }
        }
    });

    let viewport = egui::ViewportBuilder::default()
        .with_inner_size([660.0, 520.0])
        .with_min_inner_size([600.0, 400.0]);

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    let cmd_tx_ui = cmd_tx.clone();

    eframe::run_native(
        "strop",
        options,
        Box::new(move |cc| {
            let app = StropApp::new(cc, cmd_tx_ui, res_rx);
            Ok(Box::new(app))
        }),
    )
    .map_err(|e| format!("eframe: {e}"))?;

    running.store(false, Ordering::SeqCst);
    let _ = cmd_tx.send(HidCommand::Quit);
    Ok(())
}

// ---- Application state ----

pub(crate) struct StropApp {
    pub(crate) current_tab: Tab,
    pub(crate) state: DeviceState,
    pub(crate) anc_idx: usize,
    pub(crate) anc_level: f32,
    pub(crate) sidetone: f32,
    pub(crate) led_idx: usize,
    pub(crate) sleep_idx: usize,
    pub(crate) ll_enabled: bool,
    pub(crate) mic_muted: bool,
    pub(crate) mic_preset_idx: usize,
    pub(crate) spatial_enabled: bool,
    pub(crate) spatial_layout: usize,
    spatial: Option<Box<SpatialController>>,
    last_device_poll: Instant,
    pending_write: Option<(Instant, DeviceState)>,
    cmd_tx: mpsc::Sender<HidCommand>,
    res_rx: mpsc::Receiver<HidResult>,
}

#[cfg(target_os = "macos")]
mod spatial_impl {
    pub type SpatialController = strop_coreaudio::SpatialAudio;
    pub fn start_spatial(layout: usize) -> Result<SpatialController, String> {
        let mut s = strop_coreaudio::SpatialAudio::new("BlackShark");
        s.set_layout(match layout {
            1 => strop_coreaudio::AudioLayout::Surround714,
            _ => strop_coreaudio::AudioLayout::Surround71,
        });
        s.start().map_err(|e| format!("{e}"))?;
        Ok(s)
    }
}

#[cfg(not(target_os = "macos"))]
mod spatial_impl {
    pub struct SpatialController;
    pub fn start_spatial(_layout: usize) -> Result<SpatialController, String> {
        Err("spatial audio requires macOS 14.2+".into())
    }
}#[derive(PartialEq)]
pub(crate) enum Tab {
    Headset,
    Microphone,
    Spatial,
}

impl StropApp {
    fn new(
        _cc: &eframe::CreationContext<'_>,
        cmd_tx: mpsc::Sender<HidCommand>,
        res_rx: mpsc::Receiver<HidResult>,
    ) -> Self {
        let menu = Menu::new();
        let _battery_item = MenuItem::new("Battery: —", true, None);
        menu.append(&_battery_item).ok();
        _battery_item.set_enabled(false);
        menu.append(&PredefinedMenuItem::separator()).ok();
        menu.append(&PredefinedMenuItem::quit(None)).ok();
        let _tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("strop")
            .build()
            .expect("tray icon");

        Self {
            current_tab: Tab::Headset,
            state: DeviceState::default(),
            anc_idx: 0,
            anc_level: 1.0,
            sidetone: 0.0,
            led_idx: 0,
            sleep_idx: 0,
            ll_enabled: false,
            mic_muted: false,
            mic_preset_idx: 0,
            spatial_enabled: false,
            spatial_layout: 0,
            spatial: None,
            last_device_poll: Instant::now() - Duration::from_secs(REFRESH_SECS + 1),
            pending_write: None,
            cmd_tx,
            res_rx,
        }
    }

    pub(crate) fn send_cmd(&self, cmd: HidCommand) {
        let _ = self.cmd_tx.send(cmd);
    }

    /// Schedule an EQ band write with 250ms debounce.
    pub(crate) fn schedule_band_write(&mut self, index: usize, value: f32) {
        self.state.bands[index] = value;
        let deadline = Instant::now() + Duration::from_millis(250);
        let state = self.state.clone();
        self.pending_write = Some((deadline, state));
    }

    pub(crate) fn toggle_spatial(&mut self) {
        if self.spatial_enabled {
            // Start spatial audio.
            match spatial_impl::start_spatial(self.spatial_layout) {
                Ok(controller) => {
                    self.spatial = Some(Box::new(controller));
                }
                Err(e) => {
                    self.spatial_enabled = false;
                    eprintln!("spatial audio: {e}");
                }
            }
        } else {
            // Stop spatial audio by dropping the controller.
            self.spatial = None;
        }
    }

    fn flush_pending_write(&mut self) {
        if let Some((deadline, ref _state)) = self.pending_write {
            if Instant::now() >= deadline {
                self.pending_write = None;
                // Full EQ write needs all 10 bands together.
                // TODO: build Eq from self.state.bands and send write command
            }
        }
    }
}

impl eframe::App for StropApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Drain results from the HID thread.
        loop {
            match self.res_rx.try_recv() {
                Ok(HidResult::State(state)) => {
                    self.state = state;
                    // Sync UI knobs to device state.
                    self.anc_idx = self.state.anc_mode.map(|v| {
                        match v { 0x00 => 0, 0x01 => 1, 0x50 => 2, _ => 0 }
                    }).unwrap_or(0);
                    self.anc_level = self.state.anc_level.unwrap_or(1) as f32;
                    self.sidetone = self.state.sidetone.unwrap_or(0) as f32;
                    self.led_idx = self.state.led_mode.unwrap_or(0).min(2) as usize;
                    self.sleep_idx = match self.state.auto_power_off {
                        Some(15) => 0, Some(30) => 1, Some(45) => 2, Some(60) => 3,
                        _ => 0,
                    };
                    self.ll_enabled = self.state.hyperspeed.unwrap_or(false);
                    self.mic_muted = self.state.mic_muted.unwrap_or(false);
                    self.mic_preset_idx = self.state.mic_preset_idx
                        .map(|v| v.saturating_sub(0x20) as usize)
                        .unwrap_or(0).min(3);
                }
                Ok(HidResult::WriteAck) => {}
                Err(_) => break,
            }
        }

        // Periodic full state read.
        let now = Instant::now();
        if now - self.last_device_poll >= Duration::from_secs(REFRESH_SECS) {
            self.last_device_poll = now;
            self.send_cmd(HidCommand::ReadAll);
        }

        self.flush_pending_write();

        // Top bar.
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(match self.state.battery {
                    Some(pct) => format!("🔋 {pct}%"),
                    None => "🔋 —".into(),
                });
                ui.separator();
                ui.selectable_value(&mut self.current_tab, Tab::Headset, "Headset");
                ui.selectable_value(&mut self.current_tab, Tab::Microphone, "Microphone");
                ui.selectable_value(&mut self.current_tab, Tab::Spatial, "Spatial");
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| match self.current_tab {
            Tab::Headset => tabs::headset::show(ui, self),
            Tab::Microphone => tabs::microphone::show(ui, self),
            Tab::Spatial => tabs::spatial::show(ui, self),
        });

        if self.pending_write.is_some() {
            ctx.request_repaint();
        }
    }
}

// ---- Synchronous HID operations (run on background thread) ----

fn read_all_sync() -> Result<DeviceState, String> {
    let device = hid::open(RAZER_VID, ALL_PIDS).map_err(|e| format!("open: {e}"))?;
    let mut s = DeviceState::default();

    let timeout = Duration::from_secs(3);

    // Battery
    if let Ok(reply) = device.request(&strop_proto::battery(), timeout) {
        s.battery = reply.args.first().copied().filter(|&v| v != 0xFF);
    }

    // ANC
    if let Ok(reply) = device.request(&strop_proto::anc(), timeout) {
        if reply.args.len() >= 2 {
            s.anc_mode = Some(reply.args[0]);
            s.anc_level = Some(reply.args[1]);
        }
    }

    // Sidetone
    if let Ok(reply) = device.request(&strop_proto::sidetone_volume(), timeout) {
        if let Some(&v) = reply.args.first() {
            s.sidetone = (v != 0xFF).then_some(v);
        }
    }

    // LED mode
    if let Ok(reply) = device.request(&strop_proto::dongle_led(), timeout) {
        s.led_mode = reply.args.first().copied().filter(|&v| v != 0xFF);
    }

    // Auto power-off
    if let Ok(reply) = device.request(&strop_proto::auto_power_off(), timeout) {
        s.auto_power_off = reply.args.first().copied().filter(|&v| v != 0xFF);
    }

    // HyperSpeed
    if let Ok(reply) = device.request(&strop_proto::hyperspeed(), timeout) {
        s.hyperspeed = reply.args.first().map(|&v| v == 1);
    }

    // Mic status
    if let Ok(reply) = device.request(&strop_proto::mic_status(), timeout) {
        if let Some(&v) = reply.args.first() {
            s.mic_muted = (v != 0xFF).then(|| v == 1);
        }
    }

    // Mic preset
    if let Ok(reply) = device.request(&strop_proto::mic_preset_index(), timeout) {
        if let Some(&v) = reply.args.first() {
            s.mic_preset_idx = (v != 0xFF).then_some(v);
        }
    }

    // Read active slot bands
    for pos in 0..9 {
        if let Ok(reply) = device.request(&strop_proto::order_info(pos), timeout) {
            if let Ok(order) = strop_proto::parse_order(&reply.args) {
                if order.active {
                    if let Ok(reply) = device.request(&strop_proto::get_bands(pos), timeout) {
                        if reply.args.len() >= 11 {
                            if let Ok(eq) = strop_proto::Eq::from_bytes(&reply.args[1..11]) {
                                for (i, v) in eq.0.iter().enumerate() {
                                    s.bands[i] = *v as f32;
                                }
                            }
                        }
                    }
                    break;
                }
            }
        }
    }

    Ok(s)
}

fn write_anc_sync(mode: u8, level: u8) -> Result<(), String> {
    let device = hid::open(RAZER_VID, ALL_PIDS).map_err(|e| format!("open: {e}"))?;
    device.request(&strop_proto::set_anc(mode, level), Duration::from_secs(3))
        .map_err(|e| format!("set_anc: {e}"))?;
    Ok(())
}

fn write_sidetone_sync(level: u8) -> Result<(), String> {
    let device = hid::open(RAZER_VID, ALL_PIDS).map_err(|e| format!("open: {e}"))?;
    device.request(&strop_proto::set_sidetone(level), Duration::from_secs(3))
        .map_err(|e| format!("set_sidetone: {e}"))?;
    Ok(())
}

fn write_led_sync(mode: u8) -> Result<(), String> {
    let device = hid::open(RAZER_VID, ALL_PIDS).map_err(|e| format!("open: {e}"))?;
    device.request(&strop_proto::set_dongle_led(mode), Duration::from_secs(3))
        .map_err(|e| format!("set_led: {e}"))?;
    Ok(())
}

fn write_poweroff_sync(mins: u8) -> Result<(), String> {
    let device = hid::open(RAZER_VID, ALL_PIDS).map_err(|e| format!("open: {e}"))?;
    device.request(&strop_proto::set_auto_power_off(mins), Duration::from_secs(3))
        .map_err(|e| format!("set_poweroff: {e}"))?;
    Ok(())
}

fn write_hyperspeed_sync(on: bool) -> Result<(), String> {
    let device = hid::open(RAZER_VID, ALL_PIDS).map_err(|e| format!("open: {e}"))?;
    device.request(&strop_proto::set_hyperspeed(on), Duration::from_secs(3))
        .map_err(|e| format!("set_hyperspeed: {e}"))?;
    Ok(())
}
