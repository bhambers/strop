//! `strop tray` — run the system tray daemon.

use clap::Args;

#[derive(Args)]
pub struct TrayArgs;

impl TrayArgs {
    pub fn run(&self) -> Result<(), String> {
        crate::ui::run_tray()
    }
}
