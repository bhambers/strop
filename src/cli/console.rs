//! `strop console` — hold the device open on a Unix socket REPL.

use clap::Args;

#[derive(Args)]
pub struct ConsoleArgs {
    /// Socket path
    #[arg(long, default_value = "/tmp/strop.sock")]
    pub socket: String,
}

impl ConsoleArgs {
    pub fn run(&self) -> Result<(), String> {
        eprintln!("strop console: not yet implemented");
        eprintln!("  (see the legacy occam console for reference: uses glass REPL)");
        eprintln!("  planned socket path: {}", self.socket);
        Ok(())
    }
}
