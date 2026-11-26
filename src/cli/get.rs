//! `strop get` — read device properties (battery, serial, firmware, etc.).

use clap::{Args, Subcommand};
use strop_hid::{self as hid, RAZER_VID, ALL_PIDS};
use strop_proto;
use std::time::Duration;

const TIMEOUT: Duration = Duration::from_secs(3);

#[derive(Args)]
pub struct GetArgs {
    #[command(subcommand)]
    pub property: GetProperty,
}

#[derive(Subcommand, Clone)]
pub enum GetProperty {
    /// Battery percentage
    Battery,
    /// Charging status
    Charging,
    /// Serial number
    Serial,
    /// Firmware version
    Firmware,
    /// ANC mode and level (raw bytes)
    Anc,
    /// Mic status (raw bytes)
    Mic,
    /// Game/chat balance (raw bytes)
    Balance,
    /// Dongle LED mode (raw bytes)
    Led,
    /// Auto power-off timeout (raw bytes)
    Poweroff,
    /// Sidetone volume (raw bytes)
    Sidetone,
    /// Ultra-low latency mode (raw bytes)
    Hyperspeed,
    /// Mic EQ preset index (raw bytes)
    Micpreset,
}

impl GetArgs {
    pub fn run(&self) -> Result<(), String> {
        let device = hid::open(RAZER_VID, ALL_PIDS)
            .map_err(|e| format!("open: {e}"))?;

        match &self.property {
            GetProperty::Battery => {
                let reply = device.request(&strop_proto::battery(), TIMEOUT)
                    .map_err(|e| format!("battery: {e}"))?;
                if strop_proto::unavailable(&reply.args) {
                    println!("battery   headset is off or out of range");
                } else {
                    let pct = reply.args.first().copied().unwrap_or(0);
                    println!("battery   {pct}%");
                }
                Ok(())
            }
            GetProperty::Charging => read_raw("charging", strop_proto::charging, &device),
            GetProperty::Serial => read_raw("serial", strop_proto::serial_number, &device),
            GetProperty::Firmware => read_raw("firmware", strop_proto::firmware_version, &device),
            GetProperty::Anc => read_raw("anc", strop_proto::anc, &device),
            GetProperty::Mic => read_raw("mic", strop_proto::mic_status, &device),
            GetProperty::Balance => read_raw("balance", strop_proto::game_chat_balance, &device),
            GetProperty::Led => read_raw("led", strop_proto::dongle_led, &device),
            GetProperty::Poweroff => read_raw("poweroff", strop_proto::auto_power_off, &device),
            GetProperty::Sidetone => read_raw("sidetone", strop_proto::sidetone_volume, &device),
            GetProperty::Hyperspeed => read_raw("hyperspeed", strop_proto::hyperspeed, &device),
            GetProperty::Micpreset => read_raw("micpreset", strop_proto::mic_preset_index, &device),
        }
    }
}

fn read_raw(
    name: &str,
    build: fn() -> strop_proto::Message,
    device: &hid::Device,
) -> Result<(), String> {
    let reply = device.request(&build(), TIMEOUT)
        .map_err(|e| format!("{name}: {e}"))?;
    if strop_proto::unavailable(&reply.args) {
        println!("{name}    headset is off or out of range");
    } else {
        println!("{name}    {:02X?}", reply.args);
    }
    Ok(())
}
