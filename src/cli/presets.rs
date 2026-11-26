//! `strop presets` — list Razer's built-in EQ library.

use clap::Args;
use strop_proto::{self as proto, Eq};

#[derive(Args)]
pub struct PresetsArgs {
    /// Optional name filter (case-insensitive substring match)
    pub filter: Option<String>,

    /// Show footstep scaling curve instead of the main band curve
    #[arg(long)]
    pub footstep: bool,
}

impl PresetsArgs {
    pub fn run(&self) -> Result<(), String> {
        let entries = proto::library();

        for e in entries {
            // Apply name filter if given.
            if let Some(ref f) = self.filter {
                if !e.name.to_lowercase().contains(&f.to_lowercase()) {
                    continue;
                }
            }

            let curve = if self.footstep { e.footstep } else { e.bands };
            let bands = format_bands(&curve);
            println!("  {:>4}  {}   {}", e.id, bands, e.name);
        }

        Ok(())
    }
}

fn format_bands(eq: &Eq) -> String {
    eq.0.iter()
        .map(|v| format!("{:+.0}", v))
        .collect::<Vec<_>>()
        .join(",")
}
