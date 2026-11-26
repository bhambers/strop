//! `strop watch` — re-apply the profile whenever the dongle reconnects.

use clap::Args;
use strop_hid;
use strop_profile;
use std::time::Duration;

use crate::cli::apply::apply_profile;

const POLL_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Args)]
pub struct WatchArgs {
    /// Profile path, defaults to ~/.config/strop/profile.toml
    #[arg(long)]
    pub profile: Option<String>,

    /// Apply immediately and exit
    #[arg(long)]
    pub once: bool,
}

impl WatchArgs {
    pub fn run(&self) -> Result<(), String> {
        let path = resolve_profile_path(self.profile.as_deref())?;
        let p = strop_profile::load(&path)
            .map_err(|e| format!("load profile: {e}"))?;

        if self.once {
            if !strop_hid::present() {
                return Err("no BlackShark V3 on the bus".into());
            }
            let device = strop_hid::open_razer(strop_hid::ALL_PIDS)
                .map_err(|e| format!("open: {e}"))?;
            let n = apply_profile(&device, &p)?;
            println!("applied  {n} change(s)");
            return Ok(());
        }

        println!("watching  (poll every {})", humantime(POLL_INTERVAL));
        println!("  press Ctrl+C to stop\n");

        let mut was_present = strop_hid::present();

        loop {
            std::thread::sleep(POLL_INTERVAL);
            let now_present = strop_hid::present();

            if now_present && !was_present {
                // Dongle just appeared — apply profile.
                match strop_hid::open_razer(strop_hid::ALL_PIDS) {
                    Ok(device) => {
                        let n = apply_profile(&device, &p).unwrap_or(0);
                        let stamp = now_stamp();
                        println!("  {stamp}  applied {n} change(s)");
                    }
                    Err(e) => {
                        let stamp = now_stamp();
                        println!("  {stamp}  open failed: {e}");
                    }
                }
            }

            was_present = now_present;
        }
    }
}

fn resolve_profile_path(override_path: Option<&str>) -> Result<std::path::PathBuf, String> {
    if let Some(p) = override_path {
        return Ok(std::path::PathBuf::from(p));
    }
    strop_profile::default_path()
}

fn now_stamp() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    let h = (secs / 3600) % 24;
    let m = (secs / 60) % 60;
    let s = secs % 60;
    format!("{h:02}:{m:02}:{s:02}")
}

fn humantime(d: Duration) -> String {
    if d.as_secs() >= 60 {
        format!("{}m", d.as_secs() / 60)
    } else {
        format!("{}s", d.as_secs())
    }
}
