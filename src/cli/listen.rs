//! `strop listen` — print input reports the device pushes unprompted.

use clap::Args;
use strop_hid::{self as hid, RAZER_VID, ALL_PIDS};
use std::time::Duration;

#[derive(Args)]
pub struct ListenArgs {
    /// How long to listen, in seconds
    #[arg(long, default_value_t = 8)]
    pub seconds: u64,
}

impl ListenArgs {
    pub fn run(&self) -> Result<(), String> {
        let device = hid::open(RAZER_VID, ALL_PIDS)
            .map_err(|e| format!("open: {e}"))?;

        let timeout = Duration::from_secs(self.seconds);
        println!("listening  {timeout:?}, press buttons on the headset\n");

        let reports = device.listen(timeout)
            .map_err(|e| format!("listen: {e}"))?;

        for (i, report) in reports.iter().enumerate() {
            let hex: Vec<String> = report.iter().map(|b| format!("{b:02X}")).collect();
            println!("  report {} ({} bytes): {}", i, report.len(), hex.join(" "));
        }

        println!("\nreports  {}", reports.len());
        Ok(())
    }
}
