//! Command ID constants for the HID audio protocol.

// Read commands
pub const CMD_GET_SERIAL_NUMBER: u8 = 0x00;
pub const CMD_GET_EDITION_ID: u8 = 0x01;
pub const CMD_GET_FIRMWARE_VERSION: u8 = 0x02;
pub const CMD_GET_ANC_STATUS_AND_LEVEL: u8 = 0x12;
pub const CMD_GET_CUSTOMER_EQ_BAND: u8 = 0x15;
pub const CMD_GET_MIC_PRESET_EQ_INDEX: u8 = 0x16;
pub const CMD_GET_MIC_CUSTOMER_EQ_BAND: u8 = 0x17;
pub const CMD_GET_SIDETONE_VOLUME: u8 = 0x19;
pub const CMD_GET_WIRELESS_CONNECTION: u8 = 0x20;
pub const CMD_GET_BATTERY_STATUS: u8 = 0x21;
pub const CMD_GET_CHARGING_STATUS: u8 = 0x2A;
pub const CMD_GET_AUTO_POWER_OFF_STATUS: u8 = 0x2C;
pub const CMD_GET_MIC_STATUS: u8 = 0x55;
pub const CMD_GET_GAME_CHAT_BALANCE: u8 = 0x5C;
pub const CMD_GET_BT_INCOMING_CALL: u8 = 0x5D;
pub const CMD_GET_HYPERSPEED_MODE: u8 = 0x5F;
pub const CMD_GET_EQ_ORDER_INFO: u8 = 0x60;
pub const CMD_GET_VOICE_PROMPT_STATUS: u8 = 0x65;
pub const CMD_GET_DONGLE_LED_STATUS: u8 = 0x66;
pub const CMD_GET_ROLLER_FUNCTION: u8 = 0x6A;

// Write commands (derived: set = get | 0x80)
pub const CMD_SET_ANC_STATUS_AND_LEVEL: u8 = 0x92;
pub const CMD_SET_AUTO_POWER_OFF_STATUS: u8 = 0xAC;
pub const CMD_SET_MIC_STATUS: u8 = 0xD5;
pub const CMD_SET_HYPERSPEED_MODE: u8 = 0xDF;
pub const CMD_SET_GAME_CHAT_BALANCE: u8 = 0xDC;

// Write commands (captured directly)
pub const CMD_SET_CUSTOMER_EQ_BAND: u8 = 0x95;
pub const CMD_SET_MIC_PRESET_EQ_INDEX: u8 = 0x96;
pub const CMD_SET_MIC_CUSTOMER_EQ_BAND: u8 = 0x97;
pub const CMD_SET_SIDETONE_STATUS: u8 = 0x98;
pub const CMD_SET_SIDETONE_VOLUME: u8 = 0x99;
pub const CMD_SET_SPEAKER_PRESET_EQ: u8 = 0x9E;
pub const CMD_SET_EQ_ORDER_INFO: u8 = 0xE0;
pub const CMD_SET_EQ_ORDER_UPDATE: u8 = 0xE1;
pub const CMD_SET_DONGLE_LED_STATUS: u8 = 0xE6;
pub const CMD_SET_EQ_FOOTSTEP_SCALING: u8 = 0xEB;

// Outer commands (device mode — rides the outer command_id, not the audio command)
pub const CMD_OUTER_GET_DEVICE_MODE: u8 = 0x84;
pub const CMD_OUTER_SET_DEVICE_MODE: u8 = 0x04;

/// Return a human-readable name for a command ID.
pub fn command_name(c: u8) -> String {
    match c {
        CMD_GET_SERIAL_NUMBER => "getSerialNumber".into(),
        CMD_GET_EDITION_ID => "getEditionID".into(),
        CMD_GET_FIRMWARE_VERSION => "getFirmwareVersion".into(),
        CMD_GET_ANC_STATUS_AND_LEVEL => "getANCStatusAndLevel".into(),
        CMD_GET_CUSTOMER_EQ_BAND => "getCustomerEQBand".into(),
        CMD_GET_MIC_PRESET_EQ_INDEX => "getMicPresetEQIndex".into(),
        CMD_GET_MIC_CUSTOMER_EQ_BAND => "getMicCustomerEQBand".into(),
        CMD_GET_SIDETONE_VOLUME => "getSidetoneVolume".into(),
        CMD_GET_WIRELESS_CONNECTION => "getWirelessConnectionStatus".into(),
        CMD_GET_BATTERY_STATUS => "getBatteryStatus".into(),
        CMD_GET_CHARGING_STATUS => "getChargingStatus".into(),
        CMD_GET_AUTO_POWER_OFF_STATUS => "getAutoPowerOffStatus".into(),
        CMD_GET_MIC_STATUS => "getMicStatus".into(),
        CMD_GET_GAME_CHAT_BALANCE => "getGameChatBalance".into(),
        CMD_GET_BT_INCOMING_CALL => "getBTIncomingCallAction".into(),
        CMD_GET_HYPERSPEED_MODE => "getHyperSpeedMode".into(),
        CMD_GET_EQ_ORDER_INFO => "getEQOrderInfo".into(),
        CMD_GET_VOICE_PROMPT_STATUS => "getVoicePromptStatus".into(),
        CMD_GET_DONGLE_LED_STATUS => "getDongleLEDStatus".into(),
        CMD_GET_ROLLER_FUNCTION => "getRollerFunction".into(),

        CMD_SET_ANC_STATUS_AND_LEVEL => "setANCStatusAndLevel".into(),
        CMD_SET_AUTO_POWER_OFF_STATUS => "setAutoPowerOffStatus".into(),
        CMD_SET_MIC_STATUS => "setMicStatus".into(),
        CMD_SET_HYPERSPEED_MODE => "setHyperSpeedMode".into(),
        CMD_SET_GAME_CHAT_BALANCE => "setGameChatBalance".into(),

        CMD_SET_CUSTOMER_EQ_BAND => "setCustomerEQBand".into(),
        CMD_SET_MIC_PRESET_EQ_INDEX => "setMicPresetEQIndex".into(),
        CMD_SET_MIC_CUSTOMER_EQ_BAND => "setMicCustomerEQBand".into(),
        CMD_SET_SIDETONE_STATUS => "setSidetoneStatus".into(),
        CMD_SET_SIDETONE_VOLUME => "setSidetoneVolume".into(),
        CMD_SET_SPEAKER_PRESET_EQ => "setSpeakerPresetEQStatus".into(),
        CMD_SET_EQ_ORDER_INFO => "setEQOrderInfo".into(),
        CMD_SET_EQ_ORDER_UPDATE => "setEQOrderUpdateStartStop".into(),
        CMD_SET_DONGLE_LED_STATUS => "setDongleLEDStatus".into(),
        CMD_SET_EQ_FOOTSTEP_SCALING => "setEQFootstepScalingBands".into(),

        CMD_OUTER_GET_DEVICE_MODE => "getDeviceMode".into(),
        CMD_OUTER_SET_DEVICE_MODE => "setDeviceMode".into(),
        _ => format!("unknown(0x{c:02X})"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_read_commands_have_names() {
        let cmds = [
            CMD_GET_SERIAL_NUMBER,
            CMD_GET_EDITION_ID,
            CMD_GET_FIRMWARE_VERSION,
            CMD_GET_ANC_STATUS_AND_LEVEL,
            CMD_GET_CUSTOMER_EQ_BAND,
            CMD_GET_MIC_PRESET_EQ_INDEX,
            CMD_GET_MIC_CUSTOMER_EQ_BAND,
            CMD_GET_SIDETONE_VOLUME,
            CMD_GET_WIRELESS_CONNECTION,
            CMD_GET_BATTERY_STATUS,
            CMD_GET_CHARGING_STATUS,
            CMD_GET_AUTO_POWER_OFF_STATUS,
            CMD_GET_MIC_STATUS,
            CMD_GET_GAME_CHAT_BALANCE,
            CMD_GET_BT_INCOMING_CALL,
            CMD_GET_HYPERSPEED_MODE,
            CMD_GET_EQ_ORDER_INFO,
            CMD_GET_VOICE_PROMPT_STATUS,
            CMD_GET_DONGLE_LED_STATUS,
            CMD_GET_ROLLER_FUNCTION,
        ];
        for &c in &cmds {
            let name = command_name(c);
            assert!(!name.is_empty(), "empty name for command {c:#04X}");
            assert!(!name.contains("unknown"), "unknown name for command {c:#04X}: {name}");
        }
    }

    #[test]
    fn all_set_commands_have_names() {
        let cmds = [
            CMD_SET_ANC_STATUS_AND_LEVEL,
            CMD_SET_AUTO_POWER_OFF_STATUS,
            CMD_SET_MIC_STATUS,
            CMD_SET_HYPERSPEED_MODE,
            CMD_SET_GAME_CHAT_BALANCE,
            CMD_SET_CUSTOMER_EQ_BAND,
            CMD_SET_MIC_PRESET_EQ_INDEX,
            CMD_SET_MIC_CUSTOMER_EQ_BAND,
            CMD_SET_SIDETONE_STATUS,
            CMD_SET_SIDETONE_VOLUME,
            CMD_SET_SPEAKER_PRESET_EQ,
            CMD_SET_EQ_ORDER_INFO,
            CMD_SET_EQ_ORDER_UPDATE,
            CMD_SET_DONGLE_LED_STATUS,
            CMD_SET_EQ_FOOTSTEP_SCALING,
        ];
        for &c in &cmds {
            let name = command_name(c);
            assert!(!name.is_empty(), "empty name for command {c:#04X}");
            assert!(!name.contains("unknown"), "unknown name for command {c:#04X}: {name}");
        }
    }

    #[test]
    fn outer_commands_have_names() {
        assert!(command_name(CMD_OUTER_GET_DEVICE_MODE).contains("getDeviceMode"));
        assert!(command_name(CMD_OUTER_SET_DEVICE_MODE).contains("setDeviceMode"));
    }

    #[test]
    fn constants_have_correct_values() {
        // Spot-check a few against the reverse-engineered values.
        assert_eq!(CMD_GET_SERIAL_NUMBER, 0x00);
        assert_eq!(CMD_GET_BATTERY_STATUS, 0x21);
        assert_eq!(CMD_SET_CUSTOMER_EQ_BAND, 0x95);
        assert_eq!(CMD_GET_EQ_ORDER_INFO, 0x60);
        assert_eq!(CMD_SET_EQ_ORDER_UPDATE, 0xE1);
        assert_eq!(CMD_SET_EQ_FOOTSTEP_SCALING, 0xEB);
        // set = get | 0x80 rule
        assert_eq!(CMD_SET_ANC_STATUS_AND_LEVEL, CMD_GET_ANC_STATUS_AND_LEVEL | 0x80);
        assert_eq!(CMD_SET_MIC_STATUS, CMD_GET_MIC_STATUS | 0x80);
    }
}
