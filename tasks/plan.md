# Implementation Plan: Strop

> Cross-platform Razer BlackShark V3 Pro / V3 headset control in Rust.
> Legacy reference: `legacy/occam` (Go + AppKit, macOS only)

## Overview

Port the existing `occam` application (Go, macOS-only, cgo) to Rust as **strop** — a single
binary that works as both a CLI tool and a system tray daemon. The architecture is a Cargo
workspace with four library crates (`strop-proto`, `strop-hid`, `strop-profile`, `strop-dsp`),
a platform services layer, and an egui/eframe settings window. The spatial audio feature
(CoreAudio process tap → HRTF binaural rendering) remains macOS-only; the DSP pipeline
is extracted as a portable crate.

**Development method:** Every task below follows Test-Driven Development
(`.skills/test-driven-development/SKILL.md`). Write the failing test first (RED),
implement the minimum code to pass it (GREEN), then refactor. Phases 1–2 are pure
logic with no hardware — they are textbook unit-test territory. Phase 3 onward uses
the prove-it pattern for device behaviour.

**Legacy golden vectors:** Where possible, capture `occam eq --dry-run` and
`occam profile` output from the working legacy binary as test vectors. The first
test in each module should assert against these known values (RED), then the
implementation makes them pass (GREEN).

## Architecture Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| UI framework | **egui/eframe** | Immediate-mode sliders are a perfect fit for 10-band EQ; pure Rust, no web tech |
| CLI framework | **clap** v4 | The Rust standard; derive macros for subcommands |
| HID (cross-platform) | **hidapi** v2 | Mature, wraps IOHIDManager/hidraw/HID.dll; VID=0x1532 filter avoids Input Monitoring |
| HID (Linux) | **hidapi** with `hidraw` feature | Direct kernel hidraw access |
| HID (Windows) | **hidapi** with WinRT | Standard Windows HID API |
| System tray | **tray-icon** + **muda** | Cross-platform; wraps NSStatusItem/Win32 tray/libappindicator |
| Audio I/O (macOS) | **coreaudio** v0.5+ | Safe Rust CoreAudio HAL; AppTapAggregateDevice, AudioObject\<T\> |
| Persistence | **serde** + **toml** | Industry standard |
| Async runtime | **tokio** | Dongle watcher, debounced slider writes, timer-based refresh |
| Homebrew | **Removed** | `cargo install` / direct binary download; `strop tray --autostart` handles launchd |
| Legacy naming | **occam** retired | Reputational issue; "strop" is the new brand |
| Spatial audio | **macOS-only** | CoreAudio process tap has no Linux/Windows equivalent; DSP is a portable library |

## Dependency Graph

```
crates/strop-proto              (pure Rust, zero deps)
   │
   ├── crates/strop-hid         (needs proto for frame building)
   ├── crates/strop-profile     (independent)
   └── crates/strop-dsp         (independent, pure math)
            │
            ├── audio-backends/coreaudio  (macOS only, depends on dsp)
            │
            └── src/platform/    (notify, service — cfg-gated per OS)
                     │
                     └── src/cli/    (combines proto + hid + profile + platform)
                              │
                              └── src/ui/    (egui tray + window)
```

## Task List

### Phase 1: Protocol Foundation (Pure Rust, no hardware)

These crates are fully testable offline. Build them first so everything downstream
has solid types to depend on.

- [ ] **Task 1: strop-proto — Frame protocol**
      Port `legacy/internal/proto/frame.go`. Frame struct, Encode/Decode, CRC,
      Message type, Status codes.
      *Files:* crates/strop-proto/src/frame.rs, crates/strop-proto/Cargo.toml
      *Scope:* Small (2-3 files)

- [ ] **Task 2: strop-proto — Commands and EQ**
      Port `commands.go`, `eq.go`, `library.go`. All command IDs, EQ curve
      encoding (sign-magnitude), ANC/sidetone/mic/LED message builders, 41
      Razer presets.
      *Files:* commands.rs, eq.rs, library.rs
      *Scope:* Medium (3-4 files)

- [ ] **Task 3: strop-profile — TOML persistence**
      Port `legacy/internal/profile/profile.go`. Profile/Slot types, serde
      derives, load/save/validate round-trip.
      *Files:* crates/strop-profile/src/lib.rs, Cargo.toml
      *Scope:* Small (2 files)

#### Checkpoint: Foundation
- [ ] `cargo test --workspace` passes
- [ ] `cargo build` succeeds
- [ ] Frame encode/decode matches legacy output (golden test vectors from `occam eq --dry-run`)
- [ ] TOML round-trip: save → load → save produces identical output

### Phase 2: HID Layer + Read-Only CLI

Now we talk to the actual headset. Read-only first — no risk of corrupting device state.

- [ ] **Task 4: strop-hid — macOS backend**
      Port `legacy/internal/hid/`. Implement open/enum/set_report/get_report/request/listen
      via `iohidmanager` crate. Interrupt-IN reply loop with CFRunLoop + timeout.
      Filter by VID=0x1532 to avoid Input Monitoring.
      *Files:* crates/strop-hid/src/lib.rs, crates/strop-hid/src/macos.rs, Cargo.toml
      *Scope:* Medium (3-4 files)

- [ ] **Task 5: CLI tree + `strop probe`**
      Set up clap App with all subcommands. Implement `strop probe` (HID enumeration,
      --open, --all, --descriptor). Wire help, version, error output.
      *Files:* src/main.rs, src/cli/mod.rs, src/cli/probe.rs
      *Scope:* Medium (3-4 files)

- [ ] **Task 6: `strop get` — Read commands**
      Implement all read commands: battery, charging, serial, firmware, anc, mic,
      balance, led, poweroff, sidetone, hyperspeed, micpreset.
      *Files:* src/cli/get.rs, some proto additions if missing
      *Scope:* Small (1-2 files)

- [ ] **Task 7: `strop profile` — Slot read**
      Read all 9 EQ slots with metadata (cloud ID, active, custom flag). Format
      output matching legacy `occam profile`.
      *Files:* src/cli/profile.rs
      *Scope:* Small (1-2 files)

#### Checkpoint: Read-Only Hardware
- [ ] `strop probe --open` detects the dongle
- [ ] `strop get battery` returns battery percentage
- [ ] `strop profile` lists all 9 slots with bands and names
- [ ] `cargo test --workspace` still passes

### Phase 3: Write Commands + Profile Round-Trip

Two-way device control. The most critical phase — write verification is real.

- [ ] **Task 8: `strop eq` — EQ write**
      Port the EQ write sequence (activate slot → write bands → commit → select).
      Support --bands, --preset, --slot, --activate, --dry-run.
      *Files:* src/cli/eq_cmd.rs
      *Scope:* Medium (3-4 files including proto additions for write commands)

- [ ] **Task 9: `strop apply` + `strop save` — Profile round-trip**
      `strop save` reads device state into TOML. `strop apply` writes TOML back.
      `--all`, `--profile PATH`, `--dry-run`. Never overwrites user-set slot names.
      *Files:* src/cli/apply.rs, profile additions for device read-then-serialize
      *Scope:* Medium (2-3 files)

- [ ] **Task 10: `strop presets` — Library listing**
      List/filter Razer's built-in presets. `--footstep` flag for footstep scaling.
      *Files:* src/cli/presets.rs
      *Scope:* Small (1 file)

#### Checkpoint: Full Device Control
- [ ] `strop eq --bands "6,6,0,0,0,0,0,0,0,0" --slot 4` audibly changes headset EQ
- [ ] `strop save --all` produces valid TOML
- [ ] `strop apply` writes it back and `strop profile` confirms the write
- [ ] V3 (non-Pro) read-only guard works — write commands print error instead of corrupting

### Phase 4: Platform Services

Background daemon infrastructure. Needed before the tray app can be useful.

- [ ] **Task 11: strop-hid — Attach/detach notifications (macOS)**
      Port IOKit `IOServiceAddMatchingNotification` via objc2. Fire callback when
      the USB dongle appears or disappears.
      *Files:* crates/strop-hid/src/macos/notify.rs, platform/notify.rs trait
      *Scope:* Medium (3 files)

- [ ] **Task 12: `strop watch` — Reconnect watcher**
      Re-apply profile on dongle attach. Use IOKit notifications (not polling).
      `--once` flag for one-shot. `--profile PATH`.
      *Files:* src/cli/watch.rs
      *Scope:* Small (1-2 files)

- [ ] **Task 13: `strop agent` — Autostart service**
      `strop agent install` writes launchd plist and loads it. `uninstall`, `status`,
      `repair` subcommands. No brew involvement.
      *Files:* src/cli/agent.rs, src/platform/service.rs, src/platform/macos/launchd.rs
      *Scope:* Medium (3 files)

- [ ] **Task 14: `strop listen` + `strop console`**
      `listen`: capture unprompted input reports for N seconds. `console`: hold the
      device open and expose it on a Unix socket REPL with proto/device facades.
      *Files:* src/cli/listen.rs, src/cli/console.rs
      *Scope:* Medium (3-4 files)

#### Checkpoint: Daemon Infrastructure
- [ ] `strop watch --once` applies profile and exits
- [ ] Unplugging and replugging the dongle triggers profile re-apply
- [ ] `strop agent install` creates a running launchd job
- [ ] `strop console` + `nc -U /tmp/strop.sock` works as a REPL

### Phase 5: System Tray + GUI

The flagship feature — a single `strop tray` command replaces the legacy `occam menu`.

- [ ] **Task 15: Tray icon + context menu**
      `strop tray` creates a system tray icon via `tray-icon`. Context menu via
      `muda` shows battery level, active slot name, slot switching, "Settings…",
      "Quit". Left-click handler re-opens the window if hidden.
      *Files:* src/ui/mod.rs (StropApp struct, tray entry point)
      *Scope:* Medium (3-4 files)

- [ ] **Task 16: Headset tab — egui settings window**
      eframe window with three tabs. Headset tab: preset picker (dropdown),
      10-band EQ sliders (31Hz–16kHz, -6dB to +6dB), ANC mode+level, game/chat
      balance, dongle LED mode, sleep timer, low-latency toggle. Slider drag
      coalesced with 250ms debounce.
      *Files:* src/ui/tabs/headset.rs, src/ui/widgets.rs
      *Scope:* Medium (3-4 files)

- [ ] **Task 17: Microphone tab**
      Mic mute toggle, sidetone slider, mic preset picker, 10-band mic EQ sliders.
      *Files:* src/ui/tabs/microphone.rs
      *Scope:* Small (2 files)

- [ ] **Task 18: Window lifecycle + background refresh**
      Close button minimises to tray (doesn't quit). Background timer refreshes
      battery + slot state every 2 minutes. Tray menu open triggers immediate
      refresh.
      *Files:* src/ui/mod.rs updates
      *Scope:* Small (1-2 files)

#### Checkpoint: GUI Parity
- [ ] `strop tray` shows a system tray icon
- [ ] Right-click menu shows battery, slot, Settings…, Quit
- [ ] Settings window opens with all three tabs functional
- [ ] Moving an EQ slider writes the band to the headset after 250ms
- [ ] Closing the window hides it to tray; Quit exits
- [ ] Battery in tray menu refreshes every 2 minutes

### Phase 6: Spatial Audio

The existing occmixer (Rust) DSP code is already in the repo. Extract it as a
portable crate and build a clean CoreAudio backend.

- [ ] **Task 19: strop-dsp — Portable DSP crate**
      Extract `legacy/occmixer/src/dsp.rs` and `hrir.rs` into `crates/strop-dsp/`.
      Pipeline, Speaker, Allpass, Fir, soft_clip, SURROUND_71, SURROUND_714.
      Embed HRIR blob via `include_bytes!`. No platform deps.
      *Files:* crates/strop-dsp/src/lib.rs, crates/strop-dsp/src/hrir.rs,
              crates/strop-dsp/hrir/sadie48.bin, Cargo.toml
      *Scope:* Small (3-4 files, pure extraction)

- [ ] **Task 20: CoreAudio backend**
      Port `legacy/occmixer/src/sys.rs`, `main.rs`, `shim.m` to use the
      `coreaudio` crate v0.5+ APIs (AppTapAggregateDevice, AudioObject).
      Implements the render callback: capture → dsp pipeline → output.
      Conditionally compiled with `#[cfg(target_os = "macos")]`.
      *Files:* audio-backends/coreaudio/src/lib.rs, Cargo.toml
      *Scope:* Medium (3 files)

- [ ] **Task 21: Spatial tab UI**
      Toggle spatial audio on/off via the CoreAudio backend. Layout selector
      (7.1 / 7.1.4). Status display (running, sample rate, buffer size).
      *Files:* src/ui/tabs/spatial.rs
      *Scope:* Small (2 files)

#### Checkpoint: Spatial Audio
- [ ] `strop-dsp` crate compiles on Linux (no macOS headers needed)
- [ ] On macOS: `strop tray` → Spatial tab → toggle on → binaural audio plays
- [ ] Toggle off restores original system audio
- [ ] HRTF attribution (SADIE II, CC BY 4.0) is documented

### Phase 7: Cross-Platform Ports

Linux and Windows support. Each OS gets its own HID backend, tray backend,
and device notification mechanism.

- [ ] **Task 22: Linux HID + dongle watcher**
      hidraw backend for strop-hid (open by VID/PID on `/dev/hidraw*`). udev
      monitor for dongle attach/detach. systemd user service for autostart.
      *Files:* crates/strop-hid/src/linux.rs, src/platform/linux.rs
      *Scope:* Medium (3-4 files)

- [ ] **Task 23: Windows HID + tray + watcher**
      Win32 HID API backend for strop-hid. Windows system tray via tray-icon.
      RegisterDeviceNotification for USB attach/detach. Task Scheduler for autostart.
      *Files:* crates/strop-hid/src/windows.rs, src/platform/windows.rs
      *Scope:* Medium (3-4 files)

- [ ] **Task 24: Cross-platform CI + release**
      GitHub Actions for macOS, Linux, Windows builds. `cargo dist` or manual
      release workflow. Binary code signing (macOS), installer (Windows .msi).
      *Files:* .github/workflows/ci.yml, .github/workflows/release.yml
      *Scope:* Medium (2-3 files)

#### Checkpoint: Cross-Platform
- [ ] `strop probe --open` works on Ubuntu 24.04
- [ ] `strop get battery` works on Windows 11
- [ ] `strop tray` shows system tray on all three platforms
- [ ] CI green on all three platforms

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| macOS Input Monitoring permission blocks HID access | **High** — breaks the core feature | Use `iohidmanager` with targeted VID=0x1532 filter; never call `IOHIDManagerOpen`; test on a launchd agent (no terminal TCC grant) |
| V3 (non-Pro) write protocol is unknown | **Medium** — V3 users get read-only | Guard write commands with `IsV3(pid)`; plan has no V3 write dependency; Phase 3 write tests are run against V3 Pro only |
| CoreAudio process tap API changes or is removed | **Medium** — spatial audio breaks | Use `coreaudio` crate which abstracts over API changes; spatial audio is a separate feature, not critical for headset control |
| `tray-icon` / `muda` don't support Wayland properly | **Low** — Linux tray is v2 feature | Linux tray can fall back to GTK status icon; CLI-only mode is always available |
| egui immediate-mode is awkward for complex sliders | **Low** — sliders are what egui does best | Prototype the slider panel in Phase 5; if immediate-mode doesn't work, fall back to iced (Elm architecture, native widgets) |
| Workspace is too large for single compile | **Low** — development iteration speed | Cargo workspace with separate crates enables incremental compilation; `cargo test -p strop-proto` tests just the protocol crate in seconds |

## Open Questions

1. **V3 write path** — Should we prioritise reverse-engineering the V3 protocol
   (0xFF13/0x01 collection) before or after the initial macOS launch?
   *Suggested: after. V3 Pro is the primary target; V3 read-only is acceptable.*

2. **Binary distribution** — `cargo install` is the minimum. Do we want
   `.pkg`/`.msi`/`.deb` installers for Phase 7?
   *Suggested: defer to Phase 7, use `cargo dist` or `tauri-bundler`-style tooling.*

3. **Spatial audio scope** — The current occmixer uses a process tap with
   macOS 14.2+ APIs. Should the strop v1 require macOS 14.2+ or support
   a fallback (e.g. BlackHole virtual device) on older macOS?
   *Suggested: require 14.2+ for now. The installed base of BlackShark V3 Pro
   users on older macOS is likely negligible.*

4. **Name verification** — "strop" confirmed as the product name for all user-facing
   text (binary name, repo, docs, config paths)?
   *Flagged as agreed — using throughout.*
