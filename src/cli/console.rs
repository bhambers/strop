//! `strop console` — REPL over Unix socket or stdin.
//!
//! Opens the HID device once, then reads commands and issues them.
//! Two modes:
//!   - **Socket** (default, Unix only): listens on `/tmp/strop.sock`, accepts one
//!     connection, runs commands line‑by‑line, sends back results.
//!   - **stdin** (`--stdin`): reads commands from stdin, prints to stdout.
//!
//! Socket mode needs `std::os::unix::net`, which does not exist on Windows, so it is
//! compiled in on Unix only; `--stdin` works on every platform.

use clap::Args;
use strop_hid::{self as hid, Device, RAZER_VID, ALL_PIDS};
use strop_proto::{self as proto, Eq};
use std::io::BufRead;
#[cfg(unix)]
use std::io::Write;
use std::time::Duration;

const TIMEOUT: Duration = Duration::from_secs(3);

#[derive(Args)]
pub struct ConsoleArgs {
    /// Unix socket path
    #[arg(long, default_value = "/tmp/strop.sock")]
    pub socket: String,

    /// Read commands from stdin instead of a socket
    #[arg(long)]
    pub stdin: bool,
}

impl ConsoleArgs {
    pub fn run(&self) -> Result<(), String> {
        let device = hid::open(RAZER_VID, ALL_PIDS)
            .map_err(|e| format!("open: {e}"))?;

        if self.stdin {
            run_stdin(&device)
        } else {
            run_socket(&device, &self.socket)
        }
    }
}

// ── Socket mode ──────────────────────────────────────────────────

#[cfg(unix)]
fn run_socket(device: &Device, path: &str) -> Result<(), String> {
    // Remove stale socket file
    let _ = std::fs::remove_file(path);

    let listener = std::os::unix::net::UnixListener::bind(path)
        .map_err(|e| format!("bind socket at {path}: {e}"))?;

    eprintln!("strop console listening on {path}");
    eprintln!("Connect with: nc -U {path}");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                eprintln!("connection accepted");
                let mut reader = std::io::BufReader::new(stream.try_clone()
                    .map_err(|e| format!("clone: {e}"))?);
                let mut line = String::new();
                loop {
                    line.clear();
                    let n = reader.read_line(&mut line)
                        .map_err(|e| format!("read: {e}"))?;
                    if n == 0 {
                        break; // EOF
                    }
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    let response = dispatch(device, trimmed);
                    writeln!(stream, "{response}").map_err(|e| format!("write: {e}"))?;
                    stream.flush().map_err(|e| format!("flush: {e}"))?;
                }
                eprintln!("connection closed");
            }
            Err(e) => {
                eprintln!("accept error: {e}");
                break;
            }
        }
    }
    Ok(())
}

#[cfg(not(unix))]
fn run_socket(_device: &Device, _path: &str) -> Result<(), String> {
    Err("socket mode is not available on this platform; use --stdin".into())
}

// ── Stdin mode ───────────────────────────────────────────────────

fn run_stdin(device: &Device) -> Result<(), String> {
    let stdin = std::io::stdin();
    let reader = stdin.lock();
    println!("strop console (stdin mode). Type 'help' for commands, 'quit' to exit.");

    for line in reader.lines() {
        let line = line.map_err(|e| format!("stdin: {e}"))?;
        let trimmed = line.trim().to_string();
        if trimmed.is_empty() {
            continue;
        }
        match trimmed.as_str() {
            "quit" | "exit" => break,
            _ => {
                let response = dispatch(device, &trimmed);
                println!("{response}");
            }
        }
    }
    Ok(())
}

// ── Command dispatcher ───────────────────────────────────────────

fn dispatch(device: &Device, cmd: &str) -> String {
    let mut parts = cmd.split_whitespace();
    let command = match parts.next() {
        Some(c) => c,
        None => return String::new(),
    };

    match command {
        "help" => cmd_help(),
        "battery" => read_cmd(device, "battery", proto::battery, format_battery),
        "charging" => read_cmd(device, "charging", proto::charging, format_hex),
        "serial" => read_cmd(device, "serial", proto::serial_number, format_hex),
        "firmware" => read_cmd(device, "firmware", proto::firmware_version, format_hex),
        "anc" => read_cmd(device, "anc", proto::anc, format_hex),
        "mic" => read_cmd(device, "mic", proto::mic_status, format_hex),
        "balance" => read_cmd(device, "balance", proto::game_chat_balance, format_hex),
        "led" => read_cmd(device, "led", proto::dongle_led, format_hex),
        "poweroff" => read_cmd(device, "poweroff", proto::auto_power_off, format_hex),
        "sidetone" => read_cmd(device, "sidetone", proto::sidetone_volume, format_hex),
        "hyperspeed" => read_cmd(device, "hyperspeed", proto::hyperspeed, format_hex),
        "micpreset" => read_cmd(device, "micpreset", proto::mic_preset_index, format_hex),

        // Write commands
        "set-anc" => set_anc(device, parts.collect::<Vec<&str>>()),
        "set-mic-mute" => set_mic_mute(device, parts.next()),
        "set-sidetone" => set_sidetone(device, parts.next()),
        "set-led" => set_led(device, parts.next()),
        "set-poweroff" => set_poweroff(device, parts.next()),
        "set-hyperspeed" => set_hyperspeed(device, parts.next()),
        "set-balance" => set_balance(device, parts.next()),
        "set-mic-preset" => set_mic_preset(device, parts.next()),

        // EQ
        "eq" => cmd_eq(device, parts.collect::<Vec<&str>>()),
        "apply" => cmd_apply(device, parts.next()),

        _ => format!("unknown command: {command}"),
    }
}

// ── Command implementations ──────────────────────────────────────

fn cmd_help() -> String {
    [
        "Commands:",
        "  help                          Show this help",
        "  battery                       Read battery percentage",
        "  charging                      Read charging status",
        "  serial                        Read serial number",
        "  firmware                      Read firmware version",
        "  anc                           Read ANC mode and level",
        "  mic                           Read mic status",
        "  balance                       Read game/chat balance",
        "  led                           Read dongle LED mode",
        "  poweroff                      Read auto power-off timeout",
        "  sidetone                      Read sidetone volume",
        "  hyperspeed                    Read hyperspeed mode",
        "  micpreset                     Read mic EQ preset index",
        "",
        "  set-anc <mode> <level>        Set ANC (mode: 0=off,1=nc,2=ambient; level: 1-4)",
        "  set-mic-mute <0|1>            Mute/unmute microphone",
        "  set-sidetone <0-15>           Set sidetone level",
        "  set-led <0-2>                 Set dongle LED mode",
        "  set-poweroff <minutes>        Set auto power-off (15, 30, 45, 60)",
        "  set-hyperspeed <0|1>          Toggle hyperspeed mode",
        "  set-balance <0-255>           Set game/chat balance",
        "  set-mic-preset <index>        Set mic EQ preset",
        "",
        "  eq <slot> <b0>..<b9>          Write 10-band EQ to slot (1-9), bands -6..+6",
        "  apply <slot>                  Activate a slot (1-9)",
        "",
        "  quit / exit                   Exit the REPL",
    ]
    .join("\n")
}

type ReadCmd = fn() -> proto::Message;
type Formatter = fn(&[u8]) -> String;

fn read_cmd(device: &Device, name: &str, build: ReadCmd, fmt: Formatter) -> String {
    let reply = match device.request(&build(), TIMEOUT) {
        Ok(r) => r,
        Err(e) => return format!("{name}: error: {e}"),
    };
    if proto::unavailable(&reply.args) {
        format!("{name}: headset is off or out of range")
    } else {
        format!("{name}: {}", fmt(&reply.args))
    }
}

fn format_battery(args: &[u8]) -> String {
    let pct = args.first().copied().unwrap_or(0);
    format!("{pct}%")
}

fn format_hex(args: &[u8]) -> String {
    let hex: Vec<String> = args.iter().map(|b| format!("{b:02X}")).collect();
    format!("{:02X?} {:?}", args, hex.join(" "))
}

// ── Write command helpers ────────────────────────────────────────

fn set_anc(_device: &Device, parts: Vec<&str>) -> String {
    if parts.len() < 2 {
        return "usage: set-anc <mode> <level>  (mode: 0=off,1=nc,2=ambient; level 1-4)".into();
    }
    let mode: u8 = match parts[0].parse() {
        Ok(v) if v <= 2 => v,
        _ => return "invalid mode: use 0 (off), 1 (nc), or 2 (ambient)".into(),
    };
    let level: u8 = match parts[1].parse() {
        Ok(v) if (1..=4).contains(&v) => v,
        _ => return "invalid level: use 1-4".into(),
    };
    let mode_val = proto::anc_mode_value(mode as usize)
        .unwrap_or(0);
    send_cmd(_device, "set-anc", || proto::set_anc(mode_val, level))
}

fn set_mic_mute(device: &Device, arg: Option<&str>) -> String {
    let val = match arg.and_then(|s| s.parse::<u8>().ok()) {
        Some(0) => false,
        Some(1) => true,
        _ => return "usage: set-mic-mute <0|1>".into(),
    };
    send_cmd(device, "set-mic-mute", || proto::set_mic_muted(val))
}

fn set_sidetone(device: &Device, arg: Option<&str>) -> String {
    let level = match arg.and_then(|s| s.parse::<u8>().ok()) {
        Some(v) if v <= 15 => v,
        _ => return "usage: set-sidetone <0-15>".into(),
    };
    send_cmd(device, "set-sidetone", || proto::set_sidetone(level))
}

fn set_led(device: &Device, arg: Option<&str>) -> String {
    let mode = match arg.and_then(|s| s.parse::<u8>().ok()) {
        Some(v) if v <= 2 => v,
        _ => return "usage: set-led <0-2>  (0=connection,1=battery,2=spectrum)".into(),
    };
    send_cmd(device, "set-led", || proto::set_dongle_led(mode))
}

fn set_poweroff(device: &Device, arg: Option<&str>) -> String {
    let minutes = match arg.and_then(|s| s.parse::<u8>().ok()) {
        Some(v) if [15, 30, 45, 60].contains(&v) => v,
        _ => return "usage: set-poweroff <15|30|45|60>".into(),
    };
    send_cmd(device, "set-poweroff", || proto::set_auto_power_off(minutes))
}

fn set_hyperspeed(device: &Device, arg: Option<&str>) -> String {
    let val = match arg.and_then(|s| s.parse::<u8>().ok()) {
        Some(0) => false,
        Some(1) => true,
        _ => return "usage: set-hyperspeed <0|1>".into(),
    };
    send_cmd(device, "set-hyperspeed", || proto::set_hyperspeed(val))
}

fn set_balance(device: &Device, arg: Option<&str>) -> String {
    let val = match arg.and_then(|s| s.parse::<u8>().ok()) {
        Some(v) => v,
        _ => return "usage: set-balance <0-255>".into(),
    };
    send_cmd(device, "set-balance", || proto::set_game_chat(val))
}

fn set_mic_preset(device: &Device, arg: Option<&str>) -> String {
    let index = match arg.and_then(|s| s.parse::<u8>().ok()) {
        Some(v) => v,
        _ => return "usage: set-mic-preset <index> (try 0x21-0x24)".into(),
    };
    send_cmd(device, "set-mic-preset", || proto::set_mic_preset_index(index))
}

fn send_cmd(device: &Device, name: &str, build: impl FnOnce() -> proto::Message) -> String {
    let reply = match device.request(&build(), TIMEOUT) {
        Ok(r) => r,
        Err(e) => return format!("{name}: error: {e}"),
    };
    if reply.status == proto::STATUS_SUCCESS {
        format!("{name}: ok")
    } else {
        format!("{name}: device replied status {:#04X}", reply.status)
    }
}

// ── EQ / apply ───────────────────────────────────────────────────

fn cmd_eq(device: &Device, parts: Vec<&str>) -> String {
    if parts.len() < 11 {
        return "usage: eq <slot> <b0> <b1> ... <b9>  (slot 1-9, bands -6..+6)".into();
    }
    let slot: u8 = match parts[0].parse() {
        Ok(v) if (1..=9).contains(&v) => v,
        _ => return "invalid slot: use 1-9".into(),
    };
    let mut bands = [0i8; 10];
    for (i, p) in parts[1..=10].iter().enumerate() {
        match p.parse::<i8>() {
            Ok(v) if (-6..=6).contains(&v) => bands[i] = v,
            _ => return format!("invalid band {i} value: use -6..+6"),
        }
    }
    let eq = Eq(bands);
    send_cmd(device, "eq", || proto::set_bands(slot, eq))
}

fn cmd_apply(device: &Device, arg: Option<&str>) -> String {
    let slot = match arg.and_then(|s| s.parse::<u8>().ok()) {
        Some(v) if (1..=9).contains(&v) => v,
        _ => return "usage: apply <slot>  (1-9)".into(),
    };
    send_cmd(device, "apply", || proto::select_preset(slot))
}

// ── Tests ────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_contains_key_commands() {
        let help = cmd_help();
        assert!(help.contains("battery"));
        assert!(help.contains("set-anc"));
        assert!(help.contains("eq"));
        assert!(help.contains("apply"));
        assert!(help.contains("quit"));
    }

    #[test]
    fn format_battery_works() {
        assert_eq!(format_battery(&[50]), "50%");
        assert_eq!(format_battery(&[100]), "100%");
        assert_eq!(format_battery(&[]), "0%");
    }

    #[test]
    fn format_hex_works() {
        let s = format_hex(&[0x12, 0x34]);
        assert!(s.contains("12"));
        assert!(s.contains("34"));
    }
}
