//! `strop apply` — write the saved profile to the headset.

use clap::Args;
use strop_hid::{self as hid, RAZER_VID, ALL_PIDS, V3_PIDS};
use strop_profile::{self as profile};

use crate::cli::device::{send, write_slot, select_slot};

#[derive(Args)]
pub struct ApplyArgs {
    /// Profile path, defaults to ~/.config/strop/profile.toml
    #[arg(long)]
    pub profile: Option<String>,

    /// Print what would be written without opening the device
    #[arg(long)]
    pub dry_run: bool,
}

impl ApplyArgs {
    pub fn run(&self) -> Result<(), String> {
        let path = resolve_profile_path(self.profile.as_deref())?;
        let p = profile::load(&path)
            .map_err(|e| format!("load profile: {e}"))?;

        if self.dry_run {
            return dry_run(&p);
        }

        let device = hid::open(RAZER_VID, ALL_PIDS)
            .map_err(|e| format!("open: {e}"))?;

        if V3_PIDS.contains(&device.info.product_id) {
            return Err("profile apply is not supported on BlackShark V3 (non-Pro)".into());
        }

        let n = apply_profile(&device, &p)?;
        println!("applied  {n} change(s)");
        Ok(())
    }
}

/// Apply a profile to the device. Returns the number of changes made.
pub fn apply_profile(device: &hid::Device, p: &profile::Profile) -> Result<usize, String> {
    let mut changes = 0;

    for slot in &p.slots {
        let eq = profile::slot_eq(slot)
            .map_err(|e| format!("slot {}: {e}", slot.index))?;
        write_slot(device, slot.index as u8, eq)?;
        changes += 1;
    }

    if p.sidetone >= 0 {
        send(device, &strop_proto::set_sidetone(p.sidetone as u8))?;
        changes += 1;
    }

    if p.active >= 0 {
        select_slot(device, p.active as u8)?;
        changes += 1;
    }

    Ok(changes)
}

fn dry_run(p: &profile::Profile) -> Result<(), String> {
    for slot in &p.slots {
        let bands: Vec<String> = slot.bands.iter().map(|v| format!("{:+.0}", v)).collect();
        let bands_str = bands.join(",");
        if let Some(ref name) = slot.name {
            println!("  slot {:<4} {}  {name}", slot.index, bands_str);
        } else {
            println!("  slot {:<4} {}", slot.index, bands_str);
        }
    }
    if p.active >= 0 {
        println!("  active    {}", p.active);
    }
    if p.sidetone >= 0 {
        println!("  sidetone  {}", p.sidetone);
    }
    println!("  dry run, nothing is sent");
    Ok(())
}

fn resolve_profile_path(override_path: Option<&str>) -> Result<std::path::PathBuf, String> {
    if let Some(p) = override_path {
        return Ok(std::path::PathBuf::from(p));
    }
    profile::default_path()
}
