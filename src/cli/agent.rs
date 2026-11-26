//! `strop agent` — manage the autostart service (launchd on macOS, systemd on Linux).

use clap::{Args, Subcommand};

#[derive(Args)]
pub struct AgentArgs {
    #[command(subcommand)]
    pub action: AgentAction,
}

#[derive(Subcommand, Clone)]
pub enum AgentAction {
    /// Install the autostart agent
    Install,
    /// Uninstall the autostart agent
    Uninstall,
    /// Show installed/loaded state
    Status,
}

impl AgentArgs {
    pub fn run(&self) -> Result<(), String> {
        match &self.action {
            AgentAction::Install => install_agent(),
            AgentAction::Uninstall => uninstall_agent(),
            AgentAction::Status => agent_status(),
        }
    }
}

#[cfg(target_os = "macos")]
fn agent_label() -> &'static str {
    "io.strop.agent"
}

#[cfg(target_os = "macos")]
fn agent_path() -> Result<std::path::PathBuf, String> {
    let home = std::env::var("HOME").map_err(|_| "cannot determine home directory".to_string())?;
    Ok(std::path::PathBuf::from(home).join("Library/LaunchAgents/io.strop.agent.plist"))
}

#[cfg(target_os = "macos")]
fn install_agent() -> Result<(), String> {
    let plist_path = agent_path()?;
    let binary = std::env::current_exe()
        .map_err(|e| format!("cannot determine binary path: {e}"))?;

    let plist = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
        <string>watch</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>ProcessType</key>
    <string>Background</string>
</dict>
</plist>
"#,
        agent_label(),
        binary.display(),
    );

    // Ensure parent directory exists.
    if let Some(parent) = plist_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("create directory: {e}"))?;
    }

    std::fs::write(&plist_path, &plist)
        .map_err(|e| format!("write plist: {e}"))?;

    // Load with launchctl.
    let status = std::process::Command::new("launchctl")
        .args(["load", &plist_path.to_string_lossy()])
        .status()
        .map_err(|e| format!("launchctl load: {e}"))?;

    if status.success() {
        println!("installed  {}", plist_path.display());
    } else {
        return Err(format!("launchctl load exited with {status}"));
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn uninstall_agent() -> Result<(), String> {
    let plist_path = agent_path()?;

    // Unload with launchctl.
    let _ = std::process::Command::new("launchctl")
        .args(["unload", &plist_path.to_string_lossy()])
        .status();

    if plist_path.exists() {
        std::fs::remove_file(&plist_path)
            .map_err(|e| format!("remove plist: {e}"))?;
    }

    println!("uninstalled");
    Ok(())
}

#[cfg(target_os = "macos")]
fn agent_status() -> Result<(), String> {
    let plist_path = agent_path()?;

    if !plist_path.exists() {
        println!("status  not installed");
        return Ok(());
    }

    let output = std::process::Command::new("launchctl")
        .args(["list", agent_label()])
        .output()
        .map_err(|e| format!("launchctl list: {e}"))?;

    if output.status.success() {
        println!("status  loaded");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("Could not find") {
            println!("status  installed but not loaded");
        } else {
            println!("status  error: {}", stderr.trim());
        }
    }

    Ok(())
}

// Non-macOS fallbacks.
#[cfg(target_os = "linux")]
fn install_agent() -> Result<(), String> {
    let home = std::env::var("HOME").map_err(|_| "cannot determine home directory".to_string())?;
    let service_path = std::path::PathBuf::from(&home)
        .join(".config/systemd/user/io.strop.service");
    let binary = std::env::current_exe()
        .map_err(|e| format!("cannot determine binary path: {e}"))?;

    let unit = format!(
        "[Unit]\n\
         Description=strop — Razer headset control\n\
         After=sound.target\n\n\
         [Service]\n\
         ExecStart={} watch\n\
         Restart=always\n\
         RestartSec=2\n\n\
         [Install]\n\
         WantedBy=default.target\n",
        binary.display()
    );

    if let Some(parent) = service_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("create directory: {e}"))?;
    }
    std::fs::write(&service_path, &unit)
        .map_err(|e| format!("write unit: {e}"))?;

    let _ = std::process::Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .status();
    let status = std::process::Command::new("systemctl")
        .args(["--user", "enable", "--now", &service_path.to_string_lossy()])
        .status()
        .map_err(|e| format!("systemctl: {e}"))?;

    if status.success() {
        println!("installed  {}", service_path.display());
    } else {
        eprintln!("warning: systemctl exited with {status}");
        println!("unit written to {}", service_path.display());
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn uninstall_agent() -> Result<(), String> {
    let home = std::env::var("HOME").map_err(|_| "cannot determine home directory".to_string())?;
    let service_path = std::path::PathBuf::from(&home)
        .join(".config/systemd/user/io.strop.service");

    let _ = std::process::Command::new("systemctl")
        .args(["--user", "disable", "--now", "io.strop.service"])
        .status();

    if service_path.exists() {
        std::fs::remove_file(&service_path)
            .map_err(|e| format!("remove unit: {e}"))?;
    }
    let _ = std::process::Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .status();

    println!("uninstalled");
    Ok(())
}

#[cfg(target_os = "linux")]
fn agent_status() -> Result<(), String> {
    let output = std::process::Command::new("systemctl")
        .args(["--user", "is-enabled", "io.strop.service"])
        .output()
        .map_err(|e| format!("systemctl: {e}"))?;

    let status = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if status == "enabled" {
        println!("status  enabled");
    } else if status == "disabled" {
        println!("status  disabled");
    } else {
        println!("status  not installed");
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn install_agent() -> Result<(), String> {
    let binary = std::env::current_exe()
        .map_err(|e| format!("cannot determine binary path: {e}"))?;
    let key = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER)
        .open_subkey_with_flags(
            "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
            winreg::enums::KEY_SET_VALUE,
        )
        .map_err(|e| format!("open registry: {e}"))?;
    key.set_value("strop", &binary.to_string_lossy().to_string())
        .map_err(|e| format!("set registry: {e}"))?;
    println!("installed  HKCU Run key");
    Ok(())
}

#[cfg(target_os = "windows")]
fn uninstall_agent() -> Result<(), String> {
    let key = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER)
        .open_subkey_with_flags(
            "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
            winreg::enums::KEY_SET_VALUE,
        )
        .map_err(|e| format!("open registry: {e}"))?;
    let _ = key.delete_value("strop");
    println!("uninstalled");
    Ok(())
}

#[cfg(target_os = "windows")]
fn agent_status() -> Result<(), String> {
    let key = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER)
        .open_subkey_with_flags(
            "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
            winreg::enums::KEY_QUERY_VALUE,
        )
        .map_err(|e| format!("open registry: {e}"))?;
    match key.get_value::<String, _>("strop") {
        Ok(_) => println!("status  installed"),
        Err(_) => println!("status  not installed"),
    }
    Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn install_agent() -> Result<(), String> {
    Err("autostart agent is not yet supported on this platform".into())
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn uninstall_agent() -> Result<(), String> {
    Err("autostart agent is not yet supported on this platform".into())
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn agent_status() -> Result<(), String> {
    Err("autostart agent is not yet supported on this platform".into())
}
