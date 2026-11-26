# strop

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="assets/western-strop-small.jpeg">
  <img alt="strop — sharpen your sound" src="assets/western-strop-small.jpeg" width="480">
</picture>

> Razer BlackShark V3 Pro / V3 headset control — native, cross-platform, no Synapse.

Synapse for macOS does not support headsets. strop talks to the dongle's
vendor HID interface directly — no cloud, no Windows, no Razer account.

```
$ strop probe --open
BlackShark V3 Pro  1532:0577
  manufacturer  Razer Inc
  transport     dongle
  usage         0xFF14/0x01
```

## Why strop?

A **strop** is a strip of leather used to sharpen a razor — a fitting name
for a tool that replaces Razer's own Synapse software, especially on macOS
where Synapse has no headset support at all.

strop is a clean-slate rewrite in Rust with a proper cross-platform
architecture, a system tray GUI, and spatial audio support.

- **macOS:** No Synapse. No brew dependency. No Input Monitoring
  permission needed (filtered by VID=0x1532, never calls IOHIDManagerOpen).
- **Linux:** Works out of the box via hidraw.
- **Windows:** Works via the Win32 HID API.
- **Single binary:** One `strop` binary. CLI mode or `strop tray` for
  the system tray daemon with the egui settings window.

## Quick start

```sh
# Build from source
cargo build --release

# Plug in your BlackShark V3 Pro dongle
./target/release/strop probe --open

# Check battery
./target/release/strop get battery

# Read all 9 EQ slots
./target/release/strop profile

# Run the system tray app
./target/release/strop tray
```

## Installation

### From source

```sh
cargo install --git https://github.com/dappermint/strop
```

### From a release binary

Download the binary for your platform from the
[releases page](https://github.com/dappermint/strop/releases).

### System tray autostart (macOS)

```sh
strop agent install   # writes ~/Library/LaunchAgents/io.strop.agent.plist
```

### System tray autostart (Linux)

```sh
strop agent install   # writes ~/.config/systemd/user/io.strop.service
```

### System tray autostart (Windows)

```sh
strop agent install   # writes HKCU\Software\Microsoft\Windows\CurrentVersion\Run
```

**No brew involved.** strop does not use or depend on Homebrew.

## Features

| Feature | V3 Pro | V3 (non-Pro) |
|---|---|---|
| Device detection | ✅ | ✅ |
| Battery level | ✅ | ✅ *Verified* |
| Charging status | ✅ | ✅ |
| Serial number | ✅ | ✅ *Verified* |
| Firmware version | ✅ | ✅ *Verified* |
| Profile read (all 9 slots) | ✅ | ✅ *Verified* |
| Slot metadata | ✅ | ✅ *Verified* |
| **EQ write** | ✅ | ❌ (write guard verified) |
| **Profile apply** | ✅ | ❌ (write guard verified) |
| ANC mode / level | ✅ | Untested |
| Sidetone / mic monitoring | ✅ | Untested |
| Mic mute | ✅ | Untested |
| Dongle LED mode | ✅ | Untested |
| Auto power-off / sleep | ✅ | Untested |
| HyperSpeed (low-latency) | ✅ | Untested |
| Mic EQ / presets | ✅ | Untested |
| System tray GUI | ✅ | ✅ *Verified* |
| Spatial audio (macOS 14.2+) | ✅ | ✅ |

\* The V3 uses an additional HID collection (0xFF13/0x01) whose write
protocol has not yet been reverse-engineered. Read commands work fine.
Write commands are guarded and return a clear error message (verified
on BlackShark V3 dongle PID 0x057A).

## Hardware verification

strop has been verified against a **BlackShark V3** (non-Pro, dongle PID
`0x057A`) across all read commands, profile reading, save/apply round-trip,
write guards, system tray, and agent installation. All 119 unit tests pass.

| Test | Result |
|------|--------|
| `strop probe --open` | ✅ Detects all 4 HID collections |
| `strop get battery` | ✅ Returns 65% |
| `strop get serial` | ✅ Returns 15-byte serial hex |
| `strop get firmware` | ✅ Returns firmware version |
| `strop get charging` | ✅ Returns `[00]` (not charging) |
| `strop profile` | ✅ All 8 populated slots read correctly |
| `strop save --all` | ✅ Writes valid TOML to `~/.config/strop/profile.toml` |
| `strop eq` (V3 guard) | ✅ Rejected with clear error message |
| `strop apply` (V3 guard) | ✅ Rejected with clear error message |
| `strop presets` | ✅ Lists 41 Razer presets, filter and --footstep work |
| `strop presets valorant` | ✅ 11 matching presets |
| `strop tray` | ✅ Window opens, battery reads via background thread |
| `strop agent install` | ✅ Writes launchd plist |

Write commands and ANC/sidetone/mic features have not been tested on the V3
(non-Pro) due to the missing write protocol. These features are expected to
work on the V3 Pro (dongle PID `0x0577`) — if you have one, please open an
issue with your test results.

## Commands

```
strop probe          Enumerate HID interfaces, detect the dongle
strop get            Read device properties
    battery              Battery percentage
    charging             Charging status
    serial               Serial number
    firmware             Firmware version
    anc                  ANC mode and level
    mic                  Mic status
    balance              Game/chat balance
    led                  Dongle LED mode
    poweroff             Auto power-off timeout
    sidetone             Sidetone volume
    hyperspeed           Ultra-low latency mode
    micpreset            Mic EQ preset index
strop profile         Read all 9 EQ slots with metadata
strop eq              Write an EQ curve to a slot
    --bands "6,6,0,0,0,0,0,0,0,0"   10 band values
    --preset "Valorant"               Library preset name
    --slot 4                          Target slot (0–8)
    --dry-run                         Preview frames without writing
    --no-activate                     Don't select the slot as active
strop save            Save headset state to TOML profile
    --all               Save every slot, not just the active one
    --profile PATH      Custom profile path
strop apply           Apply saved profile to the headset
    --dry-run           Preview without writing
    --profile PATH      Custom profile path
strop presets         List Razer's built-in EQ library (41 presets)
    [filter]            Case-insensitive name filter
    --footstep          Show footstep scaling curves
strop watch           Re-apply profile on dongle reconnect
    --once              Apply immediately and exit
strop listen          Capture input reports (debugging)
strop agent           Manage the autostart service
    install             Install and load the service
    uninstall           Unload and remove
    status              Show service state
strop console         Unix socket REPL (experimental)
strop tray            Run the system tray daemon
```

## System tray (GUI)

`strop tray` opens a system tray icon and an egui/eframe settings window
with three tabs:

### Headset tab
- 10-band EQ sliders (31 Hz – 16 kHz, −6 dB to +6 dB)
- Slider changes are debounced at 250 ms and coalesced
- ANC mode picker (Off / Noise cancelling / Ambient) with level slider
- Dongle LED mode picker
- Auto power-off timeout picker
- Ultra-low latency (HyperSpeed) toggle
- Reads current device state on startup and refreshes every 30 s

### Microphone tab
- Mic mute toggle
- Sidetone (mic monitoring) level slider
- Mic EQ preset picker (Default / Flat / Broadcast / Esports)
- 10-band mic EQ sliders

### Spatial tab
- Spatial audio on/off toggle
- Speaker layout selector (7.1 / 7.1.4)
- macOS 14.2+ CoreAudio process tap required for spatial audio

The tray icon shows battery level and persists even when the window
is closed. All device writes happen on a dedicated background thread
to avoid macOS hidapi IOKit re-entrancy issues.

## Architecture

```
strop/
├── crates/
│   ├── strop-proto/          HID frame protocol, commands, EQ library
│   │                         41 Razer cloud presets
│   │                         Pure Rust, zero dependencies, fully tested
│   ├── strop-hid/            Cross-platform HID via hidapi
│   │                         macOS: IOHIDManager (no Input Monitoring)
│   │                         Linux: hidraw
│   │                         Windows: Win32 HID API
│   ├── strop-profile/        TOML profile persistence via serde
│   └── strop-dsp/            Portable binaural spatial audio DSP
│                             Upmix, allpass decorrelator, HRTF convolution
│                             SADIE II KU100 measured HRIRs (CC BY 4.0)
├── audio-backends/
│   └── coreaudio/            macOS CoreAudio process tap backend
│                             Requires macOS 14.2+
├── src/
│   ├── main.rs               CLI dispatch (clap)
│   ├── cli/                  12 CLI subcommands
│   └── ui/                   System tray (tray-icon) + egui/eframe window
│       └── tabs/             Headset, Microphone, Spatial tabs
└── legacy/                   Original occam (Go) codebase — reference only
```

### Key design decisions

| Decision | Choice | Rationale |
|---|---|---|
| CLI framework | clap v4 | The Rust standard; derive macros |
| UI framework | egui/eframe | Immediate-mode sliders fit 10-band EQ perfectly |
| System tray | tray-icon + muda | Cross-platform; NSStatusItem / libappindicator / Win32 |
| HID | hidapi | Mature, cross-platform, VID filtering avoids Input Monitoring |
| Audio I/O | coreaudio crate v0.6 | Safe Rust CoreAudio HAL with process tap support |
| Persistence | serde + toml | Industry standard |
| Async pattern | Background thread + channels | macOS hidapi re-entrancy requires dedicated HID thread |
| Installation | cargo install only | No Homebrew, no system package manager dependency |

## Spatial audio

strop includes a realtime binaural spatial audio renderer (ported from
the original occmixer):

1. **Capture:** A CoreAudio process tap captures system audio output
   (macOS 14.2+). The tap is wrapped in a private aggregate device so
   it appears as a normal input device.
2. **Upmix:** The stereo signal is split into mid (shared) and side
   (difference) components. Only the side signal is spread to surround
   and height speakers, preserving tonal balance.
3. **Decorrelate:** Each speaker pair goes through an allpass decorrelator
   to prevent phase cancellation.
4. **HRTF:** Each speaker feed is convolved with a measured head-related
   impulse response from the **SADIE II Database** (subject D1, Neumann
   KU100 dummy head, 48 kHz, 256 taps, CC BY 4.0).
5. **Limit:** A soft-knee limiter rounds off peaks without hard-clamping.

HRIR data: 13 measured directions (7.1.4 layout), 26 KB embedded in the
binary. No interpolation — each speaker position matches a real measurement.

On Linux and Windows, spatial audio is not available (no CoreAudio
process tap equivalent). The DSP pipeline (`strop-dsp` crate) is fully
portable and could be wired to PipeWire or WASAPI with a platform
backend.

## Protocol

The dongle exposes one HID interface with four top-level collections.
The audio DSP lives on vendor usage page `0xFF14`, report ID `0x02`,
63-byte payload. Frame layout is inherited from
[Ashesh3/razer-device-control](https://github.com/Ashesh3/razer-device-control)
(MIT), which solved the BlackShark V2 Pro on Windows. The V3 Pro moved
from usage page `0xFF00` to `0xFF14` and shifted command IDs.

Every SET command follows the rule `set = get | 0x80`, confirmed from
Synapse 4 USB captures (module `AudioDevice-0.0.340`).

## License

MIT. See `LICENSE`.

### HRIR data

SADIE II Database, subject D1 (Neumann KU100 dummy head).
Armstrong, C., Thresh, L., Murphy, D., & Kearney, G. (2018).
*A Perceptual Evaluation of Individual and Non-Individual HRTFs:
A Case Study of the SADIE II Database.* Applied Sciences, 8(11), 2029.
DOI: [10.3390/app8112029](https://doi.org/10.3390/app8112029)

Database: [SADIE Project](https://www.york.ac.uk/sadie-project/database.html)
· [Zenodo deposit](https://zenodo.org/records/12092466)
· [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/)
