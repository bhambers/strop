//! `strop eq` — write an EQ curve into one of the headset's slots.

use clap::Args;
use strop_hid::{self as hid, RAZER_VID, ALL_PIDS, V3_PIDS};
use strop_proto::{self as proto, Eq, BANDS};

use crate::cli::device::{send, write_slot};

#[derive(Args)]
pub struct EqArgs {
    /// Ten comma-separated dB values (-6 to +6), overrides --preset
    #[arg(long)]
    pub bands: Option<String>,

    /// One of flat, game, music, movie, or any library preset name
    #[arg(long, default_value = "flat")]
    pub preset: String,

    /// EQ slot to write (0–8)
    #[arg(long, default_value_t = 0)]
    pub slot: u8,

    /// Also select the slot as active
    #[arg(long, default_value_t = true)]
    pub activate: bool,

    /// Print frames without opening or writing to the device
    #[arg(long)]
    pub dry_run: bool,
}

impl EqArgs {
    pub fn run(&self) -> Result<(), String> {
        if self.slot > 8 {
            return Err("slot must be 0–8".into());
        }

        let eq = resolve_eq(self.bands.as_deref(), &self.preset)?;

        if self.dry_run {
            return dry_run(self.slot, eq, self.activate);
        }

        let device = hid::open(RAZER_VID, ALL_PIDS)
            .map_err(|e| format!("open: {e}"))?;

        if V3_PIDS.contains(&device.info.product_id) {
            return Err("EQ write is not supported on BlackShark V3 (non-Pro)".into());
        }

        write_slot(&device, self.slot, eq)?;

        if self.activate {
            send(&device, &proto::select_preset(self.slot))?;
        }

        println!("sent");
        Ok(())
    }
}

fn dry_run(slot: u8, eq: Eq, activate: bool) -> Result<(), String> {
    let msgs: Vec<(&str, proto::Message)> = vec![
        ("activateSlot", proto::activate_slot(slot)),
        ("setCustomerEQBand", proto::set_bands(slot, eq)),
        ("commitFootstepScaling", proto::commit_bands(slot, eq)),
    ];

    println!("slot {slot}");
    for (label, msg) in &msgs {
        let payload = msg.encode().map_err(|e| format!("encode {label}: {e}"))?;
        print_frame(label, &payload);
    }

    if activate {
        let sel = proto::select_preset(slot);
        let payload = sel.encode().map_err(|e| format!("encode selectPreset: {e}"))?;
        print_frame("selectPreset", &payload);
    }

    Ok(())
}

fn print_frame(label: &str, payload: &[u8; 63]) {
    println!("  {label}");
    for (i, chunk) in payload.chunks(16).enumerate() {
        let hex: Vec<String> = chunk.iter().map(|b| format!("{b:02X}")).collect();
        println!("    {:3}  {}", i * 16, hex.join(" "));
    }
}

/// Resolve a band string or preset name to an EQ curve.
pub fn resolve_eq(bands: Option<&str>, preset: &str) -> Result<Eq, String> {
    if let Some(b) = bands {
        parse_bands(b)
    } else {
        resolve_preset(preset)
    }
}

fn parse_bands(s: &str) -> Result<Eq, String> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != BANDS {
        return Err(format!("expected {BANDS} band values, got {}", parts.len()));
    }
    let mut arr = [0i8; BANDS];
    for (i, p) in parts.iter().enumerate() {
        let v: i32 = p.trim().parse().map_err(|_| format!("invalid band value: {p}"))?;
        if v < -6 || v > 6 {
            return Err(format!("band {i} value {v} is outside -6 to +6"));
        }
        arr[i] = v as i8;
    }
    Ok(Eq(arr))
}

fn resolve_preset(name: &str) -> Result<Eq, String> {
    for (n, eq) in strop_proto::presets() {
        if n == name {
            return Ok(eq);
        }
    }
    if let Some(entry) = strop_proto::library_by_name(name) {
        return Ok(entry.bands);
    }
    Err(format!("unknown preset: {name}"))
}
