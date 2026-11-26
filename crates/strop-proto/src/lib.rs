//! HID frame protocol for the Razer BlackShark V3 Pro / V3.

mod commands;
pub use commands::*;

mod builders;
pub use builders::*;

mod library;
pub use library::*;

// ---- Constants ----

/// HID report ID carrying the protocol on usage page 0xFF14.
pub const REPORT_ID: u8 = 0x02;

/// Total payload length in bytes.
pub const PAYLOAD_LEN: usize = 63;

// Payload offsets (frame offsets are these + 0; HID report ID is separate).
const OFF_STATUS: usize = 0;
const OFF_TRANS_ID: usize = 1;
const OFF_REMAINING: usize = 2;
const OFF_PROTOCOL: usize = 4;
const OFF_DATA_SIZE: usize = 5;
const OFF_CLASS: usize = 6;
const OFF_COMMAND_ID: usize = 7;
const OFF_ARGS: usize = 8;
const OFF_CRC: usize = 61;
#[allow(dead_code)]
const OFF_RESERVED: usize = 62;

// Audio message offsets (relative to OFF_ARGS).
const OFF_FLAGS: usize = 0;
const OFF_COMMAND: usize = 1;
const OFF_SUB: usize = 2;
const OFF_ARG_LEN: usize = 3;
const OFF_AUDIO_ARG: usize = 4;
const AUDIO_HEADER: usize = 4; // flags, command, sub, length

// Status values.
pub const STATUS_NEW: u8 = 0x00;
pub const STATUS_BUSY: u8 = 0x01;
pub const STATUS_SUCCESS: u8 = 0x02;
pub const STATUS_FAILURE: u8 = 0x03;
pub const STATUS_NO_RESPONSE: u8 = 0x04;
pub const STATUS_UNSUPPORTED: u8 = 0x05;

/// Transaction ID carried by every captured frame.
pub const TRANSACTION_ID: u8 = 0x60;

// Flag values.
pub const FLAG_SET: u8 = 0x80;
pub const FLAG_CLEAR: u8 = 0x00;

pub const CLASS_AUDIO: u8 = 0x00;
pub const COMMAND_AUDIO: u8 = 0x00;

/// Maximum argument bytes that fit before the CRC.
pub const MAX_ARGS: usize = OFF_CRC - OFF_ARGS - AUDIO_HEADER; // 49

// ---- Types ----

/// One audio command message.
#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    pub status: u8,
    pub trans_id: u8,
    pub class: u8,
    pub command_id: u8,
    pub flags: u8,
    pub command: u8,
    pub sub: u8,
    pub args: Vec<u8>,
}

impl Message {
    /// Build a host-to-device audio command.
    pub fn new(command: u8, sub: u8, args: &[u8]) -> Self {
        Self {
            status: STATUS_NEW,
            trans_id: TRANSACTION_ID,
            class: CLASS_AUDIO,
            command_id: COMMAND_AUDIO,
            flags: FLAG_SET,
            command,
            sub,
            args: args.to_vec(),
        }
    }

    /// Encode the message into a 63-byte payload (without HID report ID).
    pub fn encode(&self) -> Result<[u8; PAYLOAD_LEN], &'static str> {
        if self.args.len() > MAX_ARGS {
            return Err("too many argument bytes");
        }
        let mut p = [0u8; PAYLOAD_LEN];
        p[OFF_STATUS] = self.status;
        p[OFF_TRANS_ID] = self.trans_id;
        p[OFF_REMAINING] = 0x00;
        p[OFF_REMAINING + 1] = 0x00;
        p[OFF_PROTOCOL] = 0x00;
        p[OFF_DATA_SIZE] = (AUDIO_HEADER + self.args.len()) as u8;
        p[OFF_CLASS] = self.class;
        p[OFF_COMMAND_ID] = self.command_id;
        p[OFF_ARGS + OFF_FLAGS] = self.flags;
        p[OFF_ARGS + OFF_COMMAND] = self.command;
        p[OFF_ARGS + OFF_SUB] = self.sub;
        p[OFF_ARGS + OFF_ARG_LEN] = self.args.len() as u8;
        p[OFF_ARGS + OFF_AUDIO_ARG..OFF_ARGS + OFF_AUDIO_ARG + self.args.len()]
            .copy_from_slice(&self.args);
        p[OFF_CRC] = crc(&p);
        // OFF_RESERVED stays 0x00 (already zeroed)
        Ok(p)
    }
}

// ---- CRC ----

/// XOR fold over bytes 1 through 60 (transaction ID through the byte before CRC).
pub fn crc(payload: &[u8]) -> u8 {
    let mut x: u8 = 0;
    for &b in &payload[OFF_TRANS_ID..OFF_CRC] {
        x ^= b;
    }
    x
}

// ---- Decode ----

/// Errors that can occur when decoding a frame.
#[derive(Debug)]
pub enum DecodeError {
    /// Payload is not 63 bytes (or 64 with report ID prefix).
    BadLength(usize),
    /// Payload has a non-matching report ID prefix.
    WrongReportId(u8),
    /// CRC in the frame does not match the computed CRC.
    BadCrc { got: u8, want: u8 },
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadLength(n) => write!(f, "payload is {n} bytes, need {PAYLOAD_LEN}"),
            Self::WrongReportId(id) => write!(f, "frame is on report 0x{id:02X}, want 0x{REPORT_ID:02X}"),
            Self::BadCrc { got, want } => {
                write!(f, "checksum mismatch: frame says 0x{got:02X}, contents give 0x{want:02X}")
            }
        }
    }
}

impl std::error::Error for DecodeError {}

/// Decode a reply payload back into a [`Message`].
///
/// Accepts either the 63-byte payload or the full 64-byte frame (with HID
/// report ID prefix).
pub fn decode(p: &[u8]) -> Result<Message, DecodeError> {
    let p = if p.len() == PAYLOAD_LEN + 1 {
        if p[0] != REPORT_ID {
            return Err(DecodeError::WrongReportId(p[0]));
        }
        &p[1..]
    } else if p.len() == PAYLOAD_LEN {
        p
    } else {
        return Err(DecodeError::BadLength(p.len()));
    };

    let computed = crc(p);
    if p[OFF_CRC] != computed {
        return Err(DecodeError::BadCrc {
            got: p[OFF_CRC],
            want: computed,
        });
    }

    let n = p[OFF_ARGS + OFF_ARG_LEN] as usize;
    let args_end = OFF_ARGS + OFF_AUDIO_ARG + n;
    let args = if n > 0 && args_end <= OFF_CRC {
        p[OFF_ARGS + OFF_AUDIO_ARG..args_end].to_vec()
    } else {
        Vec::new()
    };

    Ok(Message {
        status: p[OFF_STATUS],
        trans_id: p[OFF_TRANS_ID],
        class: p[OFF_CLASS],
        command_id: p[OFF_COMMAND_ID],
        flags: p[OFF_ARGS + OFF_FLAGS],
        command: p[OFF_ARGS + OFF_COMMAND],
        sub: p[OFF_ARGS + OFF_SUB],
        args,
    })
}

/// Reports whether a reply is the device's "nothing to tell you" answer.
pub fn unavailable(args: &[u8]) -> bool {
    args.len() == 1 && args[0] == 0xFF
}

/// Name a status byte.
pub fn status_text(s: u8) -> &'static str {
    match s {
        STATUS_NEW => "new",
        STATUS_BUSY => "busy",
        STATUS_SUCCESS => "success",
        STATUS_FAILURE => "failure",
        STATUS_NO_RESPONSE => "no response",
        STATUS_UNSUPPORTED => "not supported",
        _ => "unknown",
    }
}

// ---- EQ Types ----

/// Number of EQ bands (10).
pub const BANDS: usize = 10;

/// Number of EQ slots (9).
pub const SLOTS: usize = 9;

/// Sign bit for sign-magnitude encoding.
const SIGN_BIT: u8 = 0x80;

/// Labels for the ten EQ bands (centre frequencies).
pub const BAND_LABELS: [&str; BANDS] = [
    "31Hz", "63Hz", "125Hz", "250Hz", "500Hz",
    "1kHz", "2kHz", "4kHz", "8kHz", "16kHz",
];

/// One EQ curve, in dB per band.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Eq(pub [i8; BANDS]);

impl Eq {
    /// Render the curve as ten sign-magnitude bytes.
    pub fn bytes(&self) -> [u8; BANDS] {
        let mut out = [0u8; BANDS];
        for (i, v) in self.0.iter().enumerate() {
            out[i] = if *v < 0 {
                SIGN_BIT | (-*v as u8)
            } else {
                *v as u8
            };
        }
        out
    }

    /// Parse ten sign-magnitude bytes into an EQ curve.
    pub fn from_bytes(b: &[u8]) -> Result<Self, &'static str> {
        if b.len() != BANDS {
            return Err("expected 10 band bytes");
        }
        let mut eq = [0i8; BANDS];
        for (i, &v) in b.iter().enumerate() {
            eq[i] = if v & SIGN_BIT != 0 {
                -((v & 0x7F) as i8)
            } else {
                v as i8
            };
        }
        Ok(Self(eq))
    }
}

impl std::fmt::Display for Eq {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, v) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, ",")?;
            }
            write!(f, "{:+}", v)?;
        }
        Ok(())
    }
}

/// Built-in presets (V2 Pro curves from Ashesh3/razer-device-control).
pub fn presets() -> Vec<(&'static str, Eq)> {
    vec![
        ("flat", Eq([0, 0, 0, 0, 0, 0, 0, 0, 0, 0])),
        ("game", Eq([-3, -3, -4, 0, 5, 5, 4, 1, 0, -1])),
        ("music", Eq([2, 2, 1, 1, 2, 3, 3, 3, 1, 0])),
        ("movie", Eq([4, 4, 3, 0, -3, -1, 3, 5, 2, 1])),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    // capturedBattery is a real reply from a Synapse 4 log: a Get Battery
    // Status response reporting level 15. 63 payload bytes, no report ID.
    const CAPTURED_BATTERY_HEX: &str = concat!(
        "026000000005000080210101",
        "0f",
        "000000000000000000000000000000000000000000000000",
        "000000000000000000000000000000000000000000000000",
        "cb00"
    );

    fn captured_battery() -> Vec<u8> {
        hex::decode(CAPTURED_BATTERY_HEX).expect("valid hex")
    }

    // ---- Frame decode tests ----

    #[test]
    fn decode_captured_frame() {
        let p = captured_battery();
        assert_eq!(p.len(), PAYLOAD_LEN, "fixture is not {PAYLOAD_LEN} bytes");

        let m = decode(&p).expect("captured frame did not decode");
        assert_eq!(m.status, STATUS_SUCCESS, "status mismatch");
        assert_eq!(m.trans_id, TRANSACTION_ID, "transaction id mismatch");
        assert_eq!(m.command, CMD_GET_BATTERY_STATUS, "command mismatch");
        assert_eq!(m.flags, FLAG_SET, "flags mismatch");
        assert_eq!(m.args, vec![15], "args mismatch");
    }

    #[test]
    fn captured_crc_is_reproducible() {
        let p = captured_battery();
        let computed = crc(&p);
        assert_eq!(computed, p[OFF_CRC], "computed CRC does not match frame");
    }

    #[test]
    fn decode_rejects_bad_crc() {
        let mut p = captured_battery();
        p[OFF_CRC] ^= 0xFF;
        let result = decode(&p);
        assert!(result.is_err(), "corrupted checksum decoded without error");
    }

    #[test]
    fn decode_accepts_64_byte_frame_with_report_id() {
        let p63 = captured_battery();
        let mut p64 = vec![REPORT_ID];
        p64.extend_from_slice(&p63);
        let m = decode(&p64).expect("64-byte frame should decode");
        assert_eq!(m.command, CMD_GET_BATTERY_STATUS);
        assert_eq!(m.args, vec![15]);
    }

    #[test]
    fn decode_rejects_wrong_report_id() {
        let p63 = captured_battery();
        let mut p64 = vec![0x03]; // wrong report ID
        p64.extend_from_slice(&p63);
        let result = decode(&p64);
        assert!(matches!(result, Err(DecodeError::WrongReportId(0x03))));
    }

    #[test]
    fn decode_rejects_wrong_length() {
        assert!(matches!(decode(&[0; 10]), Err(DecodeError::BadLength(10))));
        assert!(matches!(decode(&[0; 128]), Err(DecodeError::BadLength(128))));
    }

    // ---- Encode tests ----

    #[test]
    fn encode_frame_layout() {
        let bands = Eq([0, 0, 0, 0, 0, 0, 0, 0, 0, 0]); // flat
        let band_bytes = bands.bytes();
        let mut args = vec![4u8]; // slot index
        args.extend_from_slice(&band_bytes);

        let m = Message::new(CMD_SET_CUSTOMER_EQ_BAND, 0, &args);
        let p = m.encode().expect("encode");

        assert_eq!(p.len(), PAYLOAD_LEN);
        assert_eq!(p[OFF_STATUS], STATUS_NEW);
        assert_eq!(p[OFF_TRANS_ID], TRANSACTION_ID);
        assert_eq!(p[OFF_DATA_SIZE], AUDIO_HEADER as u8 + 1 + BANDS as u8);
        assert_eq!(p[OFF_CLASS], CLASS_AUDIO);
        assert_eq!(p[OFF_COMMAND_ID], COMMAND_AUDIO);
        assert_eq!(p[OFF_ARGS + OFF_FLAGS], FLAG_SET);
        assert_eq!(p[OFF_ARGS + OFF_COMMAND], CMD_SET_CUSTOMER_EQ_BAND);
        assert_eq!(p[OFF_ARGS + OFF_ARG_LEN], 1 + BANDS as u8);
        assert_eq!(p[OFF_ARGS + OFF_AUDIO_ARG], 4); // slot index
        assert_eq!(p[OFF_RESERVED], 0);
        assert_eq!(p[OFF_CRC], crc(&p), "encoder wrote wrong CRC");
    }

    #[test]
    fn encode_rejects_oversized_args() {
        let oversized = vec![0u8; MAX_ARGS + 1];
        let m = Message::new(CMD_SET_CUSTOMER_EQ_BAND, 0, &oversized);
        assert!(m.encode().is_err(), "oversized args encoded without error");
    }

    // ---- Round-trip tests ----

    #[test]
    fn round_trip_flat() {
        let eq = Eq([0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let mut args = vec![2u8]; // slot
        args.extend_from_slice(&eq.bytes());
        let in_msg = Message::new(CMD_SET_CUSTOMER_EQ_BAND, 0, &args);
        let p = in_msg.encode().expect("encode");
        let out_msg = decode(&p).expect("decode");

        assert_eq!(out_msg.command, in_msg.command);
        assert_eq!(out_msg.sub, in_msg.sub);
        assert_eq!(out_msg.args.len(), 1 + BANDS);
        let decoded_eq = Eq::from_bytes(&out_msg.args[1..]).expect("parse bands");
        assert_eq!(decoded_eq, eq);
    }

    #[test]
    fn round_trip_movie() {
        let eq = Eq([4, 4, 3, 0, -3, -1, 3, 5, 2, 1]);
        let mut args = vec![2u8];
        args.extend_from_slice(&eq.bytes());
        let in_msg = Message::new(CMD_SET_CUSTOMER_EQ_BAND, 0, &args);
        let p = in_msg.encode().expect("encode");
        let out_msg = decode(&p).expect("decode");
        let decoded_eq = Eq::from_bytes(&out_msg.args[1..]).expect("parse bands");
        assert_eq!(decoded_eq, eq);
    }

    // ---- EQ sign-magnitude tests ----

    #[test]
    fn sign_magnitude_matches_capture() {
        let raw = [1u8, 1, 129, 0, 2, 0, 4, 4, 4, 131];
        let want = Eq([1, 1, -1, 0, 2, 0, 4, 4, 4, -3]);

        let got = Eq::from_bytes(&raw).expect("parse");
        assert_eq!(got, want, "decoded {:?}, want {:?}", got, want);

        let encoded = want.bytes();
        assert_eq!(encoded, raw, "encoded {:?}, want {:?}", encoded, raw);
    }

    #[test]
    fn encode_band_zero() {
        assert_eq!(Eq([0; 10]).bytes(), [0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn encode_band_positive() {
        let eq = Eq([5, 5, 0, 0, 0, 0, 0, 0, 0, 0]);
        let b = eq.bytes();
        assert_eq!(b[0], 5);
    }

    #[test]
    fn encode_band_negative() {
        let eq = Eq([-3, -4, 0, 0, 0, 0, 0, 0, 0, 0]);
        let b = eq.bytes();
        assert_eq!(b[0], 0x83); // sign bit | 3
        assert_eq!(b[1], 0x84); // sign bit | 4
    }

    #[test]
    fn decode_negative_zero_is_zero() {
        let eq = Eq::from_bytes(&[0x80, 0, 0, 0, 0, 0, 0, 0, 0, 0]).unwrap();
        assert_eq!(eq.0[0], 0);
    }

    #[test]
    fn from_bytes_rejects_wrong_length() {
        assert!(Eq::from_bytes(&[0; 9]).is_err());
        assert!(Eq::from_bytes(&[0; 11]).is_err());
    }

    // ---- Preset tests ----

    #[test]
    fn every_preset_is_ten_bands() {
        for (_name, eq) in presets() {
            assert_eq!(eq.bytes().len(), BANDS, "preset has wrong band count");
        }
    }

    #[test]
    fn eq_display_formats_correctly() {
        let eq = Eq([1, -2, 3, -4, 5, 0, -1, 2, -3, 4]);
        let s = eq.to_string();
        assert_eq!(s, "+1,-2,+3,-4,+5,+0,-1,+2,-3,+4");
    }

    #[test]
    fn eq_display_flat() {
        let eq = Eq([0; 10]);
        assert_eq!(eq.to_string(), "+0,+0,+0,+0,+0,+0,+0,+0,+0,+0");
    }

    #[test]
    fn predefined_presets_exist() {
        let names: Vec<&str> = presets().into_iter().map(|(n, _)| n).collect();
        assert!(names.contains(&"flat"));
        assert!(names.contains(&"game"));
        assert!(names.contains(&"music"));
        assert!(names.contains(&"movie"));
    }

    // ---- Utility tests ----

    #[test]
    fn unavailable_detects_ff() {
        assert!(unavailable(&[0xFF]));
        assert!(!unavailable(&[0x0F]));
        assert!(!unavailable(&[]));
        assert!(!unavailable(&[0xFF, 0x00]));
    }

    #[test]
    fn status_text_names_known_values() {
        assert_eq!(status_text(STATUS_NEW), "new");
        assert_eq!(status_text(STATUS_SUCCESS), "success");
        assert_eq!(status_text(STATUS_FAILURE), "failure");
        assert_eq!(status_text(STATUS_BUSY), "busy");
        assert_eq!(status_text(STATUS_NO_RESPONSE), "no response");
        assert_eq!(status_text(STATUS_UNSUPPORTED), "not supported");
        assert_eq!(status_text(0xFF), "unknown");
    }

    #[test]
    fn max_args_bounds() {
        // 49 bytes should fit
        let ok = Message::new(0, 0, &[0u8; MAX_ARGS]);
        assert!(ok.encode().is_ok());

        // 50 bytes should not
        let too_many = Message::new(0, 0, &[0u8; MAX_ARGS + 1]);
        assert!(too_many.encode().is_err());
    }

    // ================================================================
    // Task 2 tests: command names, builders, library
    // ================================================================

    // ---- Command name tests ----

    #[test]
    fn command_name_is_known_for_get_battery() {
        let name = command_name(CMD_GET_BATTERY_STATUS);
        assert!(name.contains("Battery"), "unexpected name: {name}");
    }

    #[test]
    fn command_name_is_known_for_set_eq() {
        let name = command_name(CMD_SET_CUSTOMER_EQ_BAND);
        assert!(name.contains("EQ"), "unexpected name: {name}");
    }

    #[test]
    fn command_name_unknown_fallback() {
        let name = command_name(0xFE);
        assert!(name.contains("FE"), "unexpected name: {name}");
    }

    #[test]
    fn command_name_for_outer_device_mode() {
        let name = command_name(CMD_OUTER_GET_DEVICE_MODE);
        assert_eq!(name, "getDeviceMode");
    }

    // ---- Builder tests ----
    // Test that each builder function produces the expected Message shape.

    #[test]
    fn builder_battery() {
        let m = battery();
        assert_eq!(m.command, CMD_GET_BATTERY_STATUS);
        assert_eq!(m.sub, 0);
        assert!(m.args.is_empty());
    }

    #[test]
    fn builder_serial() {
        let m = serial_number();
        assert_eq!(m.command, CMD_GET_SERIAL_NUMBER);
        assert!(m.args.is_empty());
    }

    #[test]
    fn builder_firmware() {
        let m = firmware_version();
        assert_eq!(m.command, CMD_GET_FIRMWARE_VERSION);
        assert!(m.args.is_empty());
    }

    #[test]
    fn builder_charging() {
        let m = charging();
        assert_eq!(m.command, CMD_GET_CHARGING_STATUS);
        assert!(m.args.is_empty());
    }

    #[test]
    fn builder_set_bands_has_slot_and_ten_bytes() {
        let eq = Eq([1, 2, 3, -1, -2, 0, 0, 0, 0, 0]);
        let m = set_bands(4, eq);
        assert_eq!(m.command, CMD_SET_CUSTOMER_EQ_BAND);
        assert_eq!(m.args.len(), 1 + BANDS);
        assert_eq!(m.args[0], 4); // slot index
    }

    #[test]
    fn builder_get_bands() {
        let m = get_bands(3);
        assert_eq!(m.command, CMD_GET_CUSTOMER_EQ_BAND);
        assert_eq!(m.args, vec![3]);
    }

    #[test]
    fn builder_order_info() {
        let m = order_info(2);
        assert_eq!(m.command, CMD_GET_EQ_ORDER_INFO);
        assert_eq!(m.args, vec![2]);
    }

    #[test]
    fn builder_activate_slot() {
        let m = activate_slot(5);
        assert_eq!(m.command, CMD_SET_EQ_ORDER_UPDATE);
        assert_eq!(m.args, vec![5]);
    }

    #[test]
    fn builder_select_preset() {
        let m = select_preset(1);
        assert_eq!(m.command, CMD_SET_SPEAKER_PRESET_EQ);
        assert_eq!(m.args, vec![1]);
    }

    #[test]
    fn builder_commit_bands() {
        let eq = Eq([1, 1, -1, 0, 2, 0, 4, 4, 4, -3]);
        let m = commit_bands(3, eq);
        assert_eq!(m.command, CMD_SET_EQ_FOOTSTEP_SCALING);
        assert_eq!(m.args[0], 3);
        assert_eq!(&m.args[1..], &eq.bytes());
    }

    // ---- ANC tests ----

    #[test]
    fn anc_builder() {
        let m = anc();
        assert_eq!(m.command, CMD_GET_ANC_STATUS_AND_LEVEL);
    }

    #[test]
    fn set_anc_builder() {
        let m = set_anc(0x01, 3);
        assert_eq!(m.command, CMD_SET_ANC_STATUS_AND_LEVEL);
        assert_eq!(m.args, vec![0x01, 3]);
    }

    #[test]
    fn anc_modes_cycle_correctly() {
        let modes = anc_modes();
        assert_eq!(modes.len(), 3);
        assert_eq!(modes[0].value, 0x00);
        assert_eq!(modes[0].name, "Off");
        assert_eq!(modes[1].name, "Noise cancelling");
        assert_eq!(modes[2].name, "Ambient");
    }

    #[test]
    fn anc_level_applies_only_to_noise_cancelling() {
        assert!(!anc_level_applies(0)); // Off
        assert!(anc_level_applies(1));  // Noise cancelling
        assert!(!anc_level_applies(2)); // Ambient
        assert!(!anc_level_applies(3)); // Out of range
    }

    #[test]
    fn anc_mode_row_mapping() {
        assert_eq!(anc_mode_row(0x00), Some(0)); // Off
        assert_eq!(anc_mode_row(0x01), Some(1)); // Noise cancelling
        assert_eq!(anc_mode_row(0x50), Some(2)); // Ambient
        assert_eq!(anc_mode_row(0xFF), None);    // Unknown
    }

    #[test]
    fn anc_mode_value_from_row() {
        assert_eq!(anc_mode_value(0), Some(0x00));
        assert_eq!(anc_mode_value(1), Some(0x01));
        assert_eq!(anc_mode_value(2), Some(0x50));
        assert_eq!(anc_mode_value(5), None);
    }

    // ---- Sidetone tests ----

    #[test]
    fn builder_set_sidetone() {
        let m = set_sidetone(10);
        assert_eq!(m.command, CMD_SET_SIDETONE_VOLUME);
        assert_eq!(m.args, vec![10]);
    }

    // ---- Mic tests ----

    #[test]
    fn builder_mic_status() {
        let m = mic_status();
        assert_eq!(m.command, CMD_GET_MIC_STATUS);
    }

    #[test]
    fn builder_set_mic_muted() {
        let m = set_mic_muted(true);
        assert_eq!(m.command, CMD_SET_MIC_STATUS);
        assert_eq!(m.args, vec![1]);

        let m = set_mic_muted(false);
        assert_eq!(m.args, vec![0]);
    }

    #[test]
    fn mic_presets_have_four_entries() {
        let presets = mic_presets();
        assert_eq!(presets.len(), 4);
        assert_eq!(presets[0].name, "Default");
        assert_eq!(presets[1].name, "Flat");
        assert_eq!(presets[2].name, "Broadcast");
        assert_eq!(presets[3].name, "Esports");
    }

    #[test]
    fn mic_preset_indices_start_at_base() {
        let presets = mic_presets();
        for p in &presets {
            assert!(p.index >= MIC_PRESET_BASE, "preset {} index too low", p.name);
        }
    }

    #[test]
    fn builder_mic_bands() {
        let m = mic_bands();
        assert_eq!(m.command, CMD_GET_MIC_CUSTOMER_EQ_BAND);
    }

    #[test]
    fn builder_set_mic_bands() {
        let eq = Eq([0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let m = set_mic_bands(eq);
        assert_eq!(m.command, CMD_SET_MIC_CUSTOMER_EQ_BAND);
        assert_eq!(m.args.len(), BANDS);
    }

    #[test]
    fn builder_mic_preset_index() {
        let m = mic_preset_index();
        assert_eq!(m.command, CMD_GET_MIC_PRESET_EQ_INDEX);
    }

    #[test]
    fn builder_set_mic_preset_index() {
        let m = set_mic_preset_index(0x21);
        assert_eq!(m.command, CMD_SET_MIC_PRESET_EQ_INDEX);
        assert_eq!(m.args, vec![0x21]);
    }

    // ---- Dongle LED tests ----

    #[test]
    fn led_modes_have_three_entries() {
        let modes = led_modes();
        assert_eq!(modes.len(), 3);
        assert_eq!(modes[0], "Connection status");
        assert_eq!(modes[1], "Battery status");
    }

    #[test]
    fn builder_dongle_led() {
        let m = dongle_led();
        assert_eq!(m.command, CMD_GET_DONGLE_LED_STATUS);
    }

    #[test]
    fn builder_set_dongle_led() {
        let m = set_dongle_led(1);
        assert_eq!(m.command, CMD_SET_DONGLE_LED_STATUS);
        assert_eq!(m.args, vec![1]);
    }

    // ---- Sleep / auto-power-off tests ----

    #[test]
    fn sleep_minutes_are_fixed() {
        assert_eq!(sleep_minutes(), &[15, 30, 45, 60]);
    }

    #[test]
    fn builder_auto_power_off() {
        let m = auto_power_off();
        assert_eq!(m.command, CMD_GET_AUTO_POWER_OFF_STATUS);
    }

    #[test]
    fn builder_set_auto_power_off() {
        let m = set_auto_power_off(30);
        assert_eq!(m.command, CMD_SET_AUTO_POWER_OFF_STATUS);
        assert_eq!(m.args, vec![30]);
    }

    // ---- HyperSpeed tests ----

    #[test]
    fn builder_hyperspeed() {
        let m = hyperspeed();
        assert_eq!(m.command, CMD_GET_HYPERSPEED_MODE);
    }

    #[test]
    fn builder_set_hyperspeed_on() {
        let m = set_hyperspeed(true);
        assert_eq!(m.args, vec![1]);
    }

    #[test]
    fn builder_set_hyperspeed_off() {
        let m = set_hyperspeed(false);
        assert_eq!(m.args, vec![0]);
    }

    // ---- Game/chat balance tests ----

    #[test]
    fn builder_game_chat_balance() {
        let m = game_chat_balance();
        assert_eq!(m.command, CMD_GET_GAME_CHAT_BALANCE);
    }

    #[test]
    fn builder_set_game_chat() {
        let m = set_game_chat(128);
        assert_eq!(m.command, CMD_SET_GAME_CHAT_BALANCE);
        assert_eq!(m.args, vec![128]);
    }

    // ---- Order parse tests ----

    #[test]
    fn parse_order_valid() {
        let order = parse_order(&[3, 0, 1, 1, 10, 0]).expect("parse");
        assert_eq!(order.index, 3);
        assert_eq!(order.voice_prompt, 0);
        assert!(order.enabled);
        assert!(order.active);
        assert_eq!(order.cloud_id, 10);
        assert!(!order.custom);
    }

    #[test]
    fn parse_order_inactive() {
        let order = parse_order(&[0, 0, 1, 0, 0, 0]).expect("parse");
        assert_eq!(order.index, 0);
        assert!(!order.active);
    }

    #[test]
    fn parse_order_custom() {
        let order = parse_order(&[0, 0, 1, 1, 0, 1]).expect("parse");
        assert!(order.custom);
    }

    #[test]
    fn parse_order_too_short() {
        assert!(parse_order(&[1, 2, 3]).is_err());
    }

    // ---- Library tests ----

    #[test]
    fn library_has_41_entries() {
        let entries = library();
        assert_eq!(entries.len(), 41, "library should have 41 presets");
    }

    #[test]
    fn library_entry_by_id_found() {
        let e = library_entry_by_id(10).expect("Valorant preset");
        assert_eq!(e.name, "Valorant");
    }

    #[test]
    fn library_entry_by_id_not_found() {
        assert!(library_entry_by_id(255).is_none());
    }

    #[test]
    fn library_name_found() {
        let name = library_name(11).expect("CS2");
        assert_eq!(name, "Counter-Strike 2");
    }

    #[test]
    fn library_name_not_found() {
        assert!(library_name(255).is_none());
    }

    #[test]
    fn library_by_name_found() {
        let e = library_by_name("Valorant").expect("Valorant");
        assert_eq!(e.id, 10);
    }

    #[test]
    fn library_by_name_not_found() {
        assert!(library_by_name("NonExistent").is_none());
    }

    #[test]
    fn library_every_entry_has_ten_bands() {
        for e in library() {
            assert_eq!(
                e.bands.bytes().len(),
                BANDS,
                "library entry {} has wrong band count",
                e.id
            );
        }
    }

    #[test]
    fn library_every_entry_has_footstep_ten_bands() {
        for e in library() {
            assert_eq!(
                e.footstep.bytes().len(),
                BANDS,
                "library entry {} has wrong footstep band count",
                e.id
            );
        }
    }

    // ---- Preset names test ----

    #[test]
    fn preset_names_has_six_entries() {
        let names = preset_names();
        assert_eq!(names.len(), 6);
        assert_eq!(names[0], "Default");
        assert_eq!(names[3], "Esports 1");
    }

    // ---- Profile command flow tests ----
    // These verify the get_bands reply format: args are [slot_idx, b0..b9].
    // The CLI's profile command must skip the slot byte before parsing bands.

    #[test]
    fn get_bands_reply_has_slot_prefix() {
        // Simulate a get_bands(slot=3) -> reply with slot prefix + 10 bands.
        let bands = Eq([1, 1, -1, 0, 2, 0, 4, 4, 4, -3]);
        let mut reply_args = vec![3u8]; // slot index
        reply_args.extend_from_slice(&bands.bytes());

        // The reply args should be 11 bytes: 1 slot + 10 bands.
        assert_eq!(reply_args.len(), 1 + BANDS);
        assert_eq!(reply_args[0], 3);

        // Skip the slot prefix — this is what profile.rs does.
        let parsed = Eq::from_bytes(&reply_args[1..=BANDS])
            .expect("should parse 10 band bytes from args[1..=10]");
        assert_eq!(parsed, bands);
    }

    #[test]
    fn get_bands_encode_decode_round_trip() {
        // Build a get_bands message.
        let msg = get_bands(5);
        assert_eq!(msg.args, vec![5]);

        // Encode it.
        let _encoded = msg.encode().expect("encode");

        // Simulate a device reply: same command, status SUCCESS, 11 args.
        // Build a reply payload that decode() will accept.
        let bands = Eq([-3, -2, -1, 0, 1, 2, 3, 2, 1, 0]);
        let mut reply_args = vec![5u8]; // slot index
        reply_args.extend_from_slice(&bands.bytes());

        let mut reply_msg = Message::new(CMD_GET_CUSTOMER_EQ_BAND, 0, &reply_args);
        reply_msg.status = STATUS_SUCCESS;
        let reply_encoded = reply_msg.encode().expect("encode reply");

        // Decode the reply.
        let decoded = decode(&reply_encoded).expect("decode");
        assert_eq!(decoded.status, STATUS_SUCCESS);
        assert_eq!(decoded.command, CMD_GET_CUSTOMER_EQ_BAND);

        // The args should be [slot, b0..b9] — 11 bytes.
        assert_eq!(decoded.args.len(), 1 + BANDS);
        assert_eq!(decoded.args[0], 5, "first byte should be slot index");

        // Parse bands from args[1..=10] (skip the slot prefix).
        let parsed = Eq::from_bytes(&decoded.args[1..=BANDS])
            .expect("parse bands from args[1..=10]");
        assert_eq!(parsed, bands);
    }
}
