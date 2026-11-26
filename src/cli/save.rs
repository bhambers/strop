//! `strop save` — read the headset's current state into a profile file.

use clap::Args;
use strop_hid::{self as hid, RAZER_VID, ALL_PIDS};
use strop_profile::{self as profile, Profile, SLOTS};

use crate::cli::device::read_slot;

#[derive(Args)]
pub struct SaveArgs {
    /// Profile path, defaults to ~/.config/strop/profile.toml
    #[arg(long)]
    pub profile: Option<String>,

    /// Save every slot, not just the active one
    #[arg(long)]
    pub all: bool,
}

impl SaveArgs {
    pub fn run(&self) -> Result<(), String> {
        let path = resolve_profile_path(self.profile.as_deref())?;
        let device = hid::open(RAZER_VID, ALL_PIDS)
            .map_err(|e| format!("open: {e}"))?;

        // Keep existing names from disk so save doesn't wipe them.
        let existing_names = load_existing_names(&path);

        let mut p = Profile::new();
        let mut first = true;

        for pos in 0..SLOTS as u8 {
            let (order, eq) = match read_slot(&device, pos) {
                Ok(v) => v,
                Err(_) => continue, // skip slots the headset can't read
            };

            if order.active {
                p.active = pos as i32;
            }

            if self.all || order.active || first {
                let name = existing_names.get(&(pos as i32)).cloned().flatten();
                let slot = profile::slot_from_eq(pos as i32, name, eq);
                p.slots.push(slot);
                first = false;
            }
        }

        profile::save(&path, &p)?;
        println!("saved  {}", path.display());
        print_plan(&p, &device);
        Ok(())
    }
}

fn load_existing_names(path: &std::path::Path) -> std::collections::HashMap<i32, Option<String>> {
    let mut map = std::collections::HashMap::new();
    if let Ok(p) = profile::load(path) {
        for s in p.slots {
            map.insert(s.index, s.name);
        }
    }
    map
}

fn resolve_profile_path(override_path: Option<&str>) -> Result<std::path::PathBuf, String> {
    if let Some(p) = override_path {
        return Ok(std::path::PathBuf::from(p));
    }
    profile::default_path()
}

fn print_plan(p: &Profile, _device: &strop_hid::Device) {
    for s in &p.slots {
        let bands: Vec<String> = s.bands.iter().map(|v| format!("{:+.0}", v)).collect();
        let bands_str = bands.join(",");
        if let Some(ref name) = s.name {
            println!("  slot {:<4} {}  {name}", s.index, bands_str);
        } else {
            println!("  slot {:<4} {}", s.index, bands_str);
        }
    }
    if p.active >= 0 {
        println!("  active    {}", p.active);
    }
    if p.sidetone >= 0 {
        println!("  sidetone  {}", p.sidetone);
    }
}
