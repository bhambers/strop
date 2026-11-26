//! Message builders for audio commands.

use crate::{commands::*, Eq, Message};

/// Build a battery status read command.
pub fn battery() -> Message {
    Message::new(CMD_GET_BATTERY_STATUS, 0x00, &[])
}

/// Build a serial number read command.
pub fn serial_number() -> Message {
    Message::new(CMD_GET_SERIAL_NUMBER, 0x00, &[])
}

/// Build a firmware version read command.
pub fn firmware_version() -> Message {
    Message::new(CMD_GET_FIRMWARE_VERSION, 0x00, &[])
}

/// Build a charging status read command.
pub fn charging() -> Message {
    Message::new(CMD_GET_CHARGING_STATUS, 0x00, &[])
}

/// Build a set EQ bands command for a specific slot.
///
/// The argument field is the slot index followed by the ten band bytes.
pub fn set_bands(slot: u8, eq: Eq) -> Message {
    let mut args = vec![slot];
    args.extend_from_slice(&eq.bytes());
    Message::new(CMD_SET_CUSTOMER_EQ_BAND, 0x00, &args)
}

/// Build a get EQ bands command (reads the currently-cursored slot).
pub fn get_bands(slot: u8) -> Message {
    Message::new(CMD_GET_CUSTOMER_EQ_BAND, 0x00, &[slot])
}

/// Build an order info command to select a slot and read its metadata.
pub fn order_info(position: u8) -> Message {
    Message::new(CMD_GET_EQ_ORDER_INFO, 0x00, &[position])
}

/// Metadata for one EQ slot, decoded from an OrderInfo reply.
#[derive(Debug, Clone, PartialEq)]
pub struct Order {
    pub index: u8,
    pub voice_prompt: u8,
    pub enabled: bool,
    pub active: bool,
    pub cloud_id: u8,
    pub custom: bool,
}

/// Parse the six argument bytes of an OrderInfo reply.
pub fn parse_order(args: &[u8]) -> Result<Order, &'static str> {
    if args.len() < 6 {
        return Err("order info is too short, need 6 bytes");
    }
    Ok(Order {
        index: args[0],
        voice_prompt: args[1],
        enabled: args[2] != 0,
        active: args[3] != 0,
        cloud_id: args[4],
        custom: args[5] != 0,
    })
}

/// Select the active onboard preset (by slot index).
pub fn select_preset(slot: u8) -> Message {
    Message::new(CMD_SET_SPEAKER_PRESET_EQ, 0x00, &[slot])
}

/// Point the EQ cursor at a slot before writing bands.
pub fn activate_slot(slot: u8) -> Message {
    Message::new(CMD_SET_EQ_ORDER_UPDATE, 0x00, &[slot])
}

/// Commit a band write — makes it stick on the device.
pub fn commit_bands(slot: u8, eq: Eq) -> Message {
    let mut args = vec![slot];
    args.extend_from_slice(&eq.bytes());
    Message::new(CMD_SET_EQ_FOOTSTEP_SCALING, 0x00, &args)
}

// ---- ANC ----

/// An ANC mode with its byte value, display name, and whether it supports level.
#[derive(Debug, Clone)]
pub struct AncMode {
    pub value: u8,
    pub name: &'static str,
    pub level: bool,
}

const ANC_MODES: [AncMode; 3] = [
    AncMode { value: 0x00, name: "Off", level: false },
    AncMode { value: 0x01, name: "Noise cancelling", level: true },
    AncMode { value: 0x50, name: "Ambient", level: false },
];

/// All three ANC modes: Off, Noise cancelling, Ambient.
pub fn anc_modes() -> Vec<AncMode> {
    ANC_MODES.to_vec()
}

/// Whether the ANC level slider applies for a given row index.
pub fn anc_level_applies(row: usize) -> bool {
    ANC_MODES.get(row).map_or(false, |m| m.level)
}

/// Map a device ANC value byte to a row index in [`anc_modes()`].
pub fn anc_mode_row(value: u8) -> Option<usize> {
    ANC_MODES.iter().position(|m| m.value == value)
}

/// Map a row index back to the device ANC value byte.
pub fn anc_mode_value(row: usize) -> Option<u8> {
    ANC_MODES.get(row).map(|m| m.value)
}

/// Build a read ANC status and level command.
pub fn anc() -> Message {
    Message::new(CMD_GET_ANC_STATUS_AND_LEVEL, 0x00, &[])
}

/// Build a set ANC mode and level command.
pub fn set_anc(mode: u8, level: u8) -> Message {
    Message::new(CMD_SET_ANC_STATUS_AND_LEVEL, 0x00, &[mode, level])
}

// ---- Sidetone ----

/// Build a set sidetone volume command (0–255).
pub fn set_sidetone(level: u8) -> Message {
    Message::new(CMD_SET_SIDETONE_VOLUME, 0x00, &[level])
}

/// Build a read sidetone volume command.
pub fn sidetone_volume() -> Message {
    Message::new(CMD_GET_SIDETONE_VOLUME, 0x00, &[])
}

// ---- Mic ----

/// Mic preset base index (0x20).
pub const MIC_PRESET_BASE: u8 = 0x20;

/// One mic EQ preset.
#[derive(Debug, Clone)]
pub struct MicPreset {
    pub index: u8,
    pub name: &'static str,
    pub bands: Eq,
}

const MIC_PRESETS: [MicPreset; 4] = [
    MicPreset { index: MIC_PRESET_BASE + 0, name: "Default", bands: Eq([-5, -4, -4, -3, -2, 1, 2, 3, 3, 3]) },
    MicPreset { index: MIC_PRESET_BASE + 1, name: "Flat", bands: Eq([0, 0, 0, 0, 0, 0, 0, 0, 0, 0]) },
    MicPreset { index: MIC_PRESET_BASE + 2, name: "Broadcast", bands: Eq([5, 4, 3, 1, -1, 0, 2, 3, 4, 4]) },
    MicPreset { index: MIC_PRESET_BASE + 3, name: "Esports", bands: Eq([-6, -5, -5, -4, 0, 1, 1, 1, 1, 1]) },
];

/// The four mic EQ presets.
pub fn mic_presets() -> Vec<MicPreset> {
    MIC_PRESETS.to_vec()
}

/// Build a mic status read command.
pub fn mic_status() -> Message {
    Message::new(CMD_GET_MIC_STATUS, 0x00, &[])
}

/// Build a set mic muted/unmuted command.
pub fn set_mic_muted(muted: bool) -> Message {
    let v = if muted { 1u8 } else { 0 };
    Message::new(CMD_SET_MIC_STATUS, 0x00, &[v])
}

/// Build a read mic EQ bands command.
pub fn mic_bands() -> Message {
    Message::new(CMD_GET_MIC_CUSTOMER_EQ_BAND, 0x00, &[])
}

/// Build a set mic EQ bands command.
pub fn set_mic_bands(eq: Eq) -> Message {
    Message::new(CMD_SET_MIC_CUSTOMER_EQ_BAND, 0x00, &eq.bytes())
}

/// Build a read mic preset index command.
pub fn mic_preset_index() -> Message {
    Message::new(CMD_GET_MIC_PRESET_EQ_INDEX, 0x00, &[])
}

/// Build a set mic preset index command.
pub fn set_mic_preset_index(index: u8) -> Message {
    Message::new(CMD_SET_MIC_PRESET_EQ_INDEX, 0x00, &[index])
}

// ---- Dongle LED ----

const LED_MODE_NAMES: [&str; 3] = ["Connection status", "Battery status", "Battery warning only"];

/// The three dongle LED modes.
pub fn led_modes() -> Vec<&'static str> {
    LED_MODE_NAMES.to_vec()
}

/// Build a read dongle LED mode command.
pub fn dongle_led() -> Message {
    Message::new(CMD_GET_DONGLE_LED_STATUS, 0x00, &[])
}

/// Build a set dongle LED mode command (0–2).
pub fn set_dongle_led(mode: u8) -> Message {
    Message::new(CMD_SET_DONGLE_LED_STATUS, 0x00, &[mode])
}

// ---- Sleep / Auto power-off ----

const SLEEP_TIMES: [u8; 4] = [15, 30, 45, 60];

/// The four idle timeout values Synapse offers.
pub fn sleep_minutes() -> &'static [u8] {
    &SLEEP_TIMES
}

/// Build a read auto power-off command.
pub fn auto_power_off() -> Message {
    Message::new(CMD_GET_AUTO_POWER_OFF_STATUS, 0x00, &[])
}

/// Build a set auto power-off command.
pub fn set_auto_power_off(minutes: u8) -> Message {
    Message::new(CMD_SET_AUTO_POWER_OFF_STATUS, 0x00, &[minutes])
}

// ---- HyperSpeed ----

/// Build a read HyperSpeed (ultra-low latency) mode command.
pub fn hyperspeed() -> Message {
    Message::new(CMD_GET_HYPERSPEED_MODE, 0x00, &[])
}

/// Build a set HyperSpeed mode command.
pub fn set_hyperspeed(on: bool) -> Message {
    let v = if on { 1u8 } else { 0 };
    Message::new(CMD_SET_HYPERSPEED_MODE, 0x00, &[v])
}

// ---- Game/Chat balance ----

/// Build a read game/chat balance command.
pub fn game_chat_balance() -> Message {
    Message::new(CMD_GET_GAME_CHAT_BALANCE, 0x00, &[])
}

/// Build a set game/chat balance command.
pub fn set_game_chat(balance: u8) -> Message {
    Message::new(CMD_SET_GAME_CHAT_BALANCE, 0x00, &[balance])
}

// ---- Preset names ----

const PRESET_NAMES: [&str; 6] = ["Default", "Game", "Music", "Esports 1", "Esports 2", "Esports 3"];

/// The six visible slot labels from Synapse's EQ tab.
pub fn preset_names() -> Vec<&'static str> {
    PRESET_NAMES.to_vec()
}
