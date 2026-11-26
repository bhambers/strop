//! `strop probe` — enumerate HID interfaces and report what the dongle exposes.

use clap::Args;
use strop_hid::{self as hid, DeviceInfo, RAZER_VID, ALL_PIDS};

#[derive(Args)]
pub struct ProbeArgs {
    /// Also try opening the BlackShark V3 Pro/V3
    #[arg(long)]
    pub open: bool,

    /// List every HID interface (may need Input Monitoring permission)
    #[arg(long)]
    pub all: bool,
}

impl ProbeArgs {
    pub fn run(&self) -> Result<(), String> {
        if self.all {
            return list_all();
        }

        let devices = hid::list().map_err(|e| format!("enumerate: {e}"))?;

        if devices.is_empty() {
            eprintln!("no Razer devices found");
            return Ok(());
        }

        for d in &devices {
            print_device(d);
        }

        if self.open {
            try_open(&devices);
        }

        Ok(())
    }
}

fn print_device(d: &DeviceInfo) {
    let pid_str = format!("{:04x}", d.product_id);
    let name = if !d.product.is_empty() {
        &d.product
    } else {
        "Unknown"
    };

    println!("{name}  {RAZER_VID:04x}:{pid_str}");
    if !d.manufacturer.is_empty() {
        println!("  manufacturer  {}", d.manufacturer);
    }
    let transport = hid::transport(d.product_id);
    println!("  transport     {transport}");
    if let Some(page) = d.usage_page {
        if let Some(usage) = d.usage {
            println!("  usage         {page:#06X}/{usage:#04X}");
        }
    }
    if !d.serial.is_empty() {
        println!("  serial        {}", d.serial);
    }
    println!();
}

fn try_open(devices: &[DeviceInfo]) {
    let target = devices.iter().find(|d| ALL_PIDS.contains(&d.product_id));
    match target {
        Some(d) => {
            println!("open ok");
            let pid = d.product_id;
            println!("  interface     {RAZER_VID:04x}:{pid:04x}");
            println!("  transport     {}", hid::transport(pid));
            let has_vendor = d.usage_page.map_or(false, |p| p >= 0xFF00);
            println!("  0xFF14/0x01   {has_vendor}");
        }
        None => {
            println!("open: no BlackShark V3 dongle found");
        }
    }
}

fn list_all() -> Result<(), String> {
    Err("--all requires raw HID enumeration which is not yet implemented via hidapi".into())
}
