//! Strop — Razer BlackShark headset control.
//!
//! Usage:
//!   strop <subcommand> [options]
//!   strop tray    # System tray daemon (future)

use clap::{Parser, Subcommand};

mod cli;
mod ui;

fn main() {
    let args = Args::parse();

    let result = match args.command {
        Command::Probe(c) => c.run(),
        Command::Get(c) => c.run(),
        Command::Profile(c) => c.run(),
        Command::Eq(c) => c.run(),
        Command::Save(c) => c.run(),
        Command::Apply(c) => c.run(),
        Command::Presets(c) => c.run(),
        Command::Watch(c) => c.run(),
        Command::Listen(c) => c.run(),
        Command::Agent(c) => c.run(),
        Command::Console(c) => c.run(),
        Command::Tray(c) => c.run(),
    };

    if let Err(e) = result {
        eprintln!("strop: {e}");
        std::process::exit(1);
    }
}

#[derive(Parser)]
#[command(name = "strop", version, about = "Razer BlackShark headset control")]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Enumerate HID interfaces and report what the dongle exposes
    Probe(cli::probe::ProbeArgs),
    /// Read a value back from the headset
    Get(cli::get::GetArgs),
    /// Read every EQ slot back off the headset
    Profile(cli::profile::ProfileArgs),
    /// Write an EQ curve into one of the headset's slots
    Eq(cli::eq_cmd::EqArgs),
    /// Read the headset's current state into a profile file
    Save(cli::save::SaveArgs),
    /// Write the saved profile to the headset
    Apply(cli::apply::ApplyArgs),
    /// List Razer's built-in EQ library
    Presets(cli::presets::PresetsArgs),
    /// Re-apply the profile whenever the dongle reconnects
    Watch(cli::watch::WatchArgs),
    /// Print input reports the device pushes
    Listen(cli::listen::ListenArgs),
    /// Manage the autostart agent
    Agent(cli::agent::AgentArgs),
    /// Open a REPL on a Unix socket (experimental)
    Console(cli::console::ConsoleArgs),
    /// Run the system tray daemon
    Tray(cli::tray::TrayArgs),
}
