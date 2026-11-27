# Strop — Task List

> Cross-platform Razer BlackShark V3 Pro / V3 headset control in Rust.
> See `tasks/plan.md` for full architecture decisions and risk analysis.

**Method:** TDD for every task. RED (failing test) → GREEN (make it pass) → REFACTOR.
See `.skills/test-driven-development/SKILL.md`.

**Legacy golden vectors:** Before implementing each module, capture `occam {command}`
output from the working binary and commit it as a test fixture.

---

## Phase 1: Protocol Foundation

- [x] **Task 1:** strop-proto — Frame protocol (frame.rs, CRC, Message, Status)
- [x] **Task 2:** strop-proto — Commands + EQ (commands.rs, eq.rs, library.rs — 41 presets)
- [x] **Task 3:** strop-profile — TOML persistence (Profile, Slot, serde round-trip)

### Checkpoint: Foundation
- [ ] `cargo test --workspace` passes
- [ ] `cargo build` succeeds
- [ ] Frame encode/decode matches legacy golden test vectors
- [ ] TOML round-trip is stable (save → load → save = identical output)

---

## Phase 2: HID Layer + Read-Only CLI

- [x] **Task 4:** strop-hid — Cross-platform HID (hidapi, open/set_report/request/listen)
- [x] **Task 5:** CLI tree + `strop probe` (clap app, HID enumeration, --open)
- [x] **Task 6:** `strop get` — Read commands (battery, serial, firmware, anc, mic, etc.)
- [x] **Task 7:** `strop profile` — Read all 9 EQ slots with metadata

### Checkpoint: Read-Only Hardware
- [x] `strop probe --open` detects the dongle
- [x] `strop get battery` returns battery percentage
- [x] `strop profile` lists all 9 slots
- [x] `cargo test --workspace` still passes

---

## Phase 3: Write Commands + Profile Round-Trip

- [x] **Task 8:** `strop eq` — EQ write (--bands, --preset, --slot, --dry-run)
- [x] **Task 9:** `strop apply` + `strop save` — Profile round-trip
- [x] **Task 10:** `strop presets` — Library listing and filtering

### Checkpoint: Full Device Control
- [x] `strop eq` audibly changes headset EQ
- [x] `strop save --all` + `strop apply` round-trips correctly
- [x] V3 read-only guard works (write commands print error)

---

## Phase 4: Platform Services

- [x] **Task 11:** strop-hid — present() polling + (future IOKit notify)
- [x] **Task 12:** `strop watch` — Reconnect watcher (poll + re-apply)
- [x] **Task 13:** `strop agent` — Autostart service (launchd plist)
- [x] **Task 14:** `strop listen` + `strop console` — Listen + REPL stub

### Checkpoint: Daemon Infrastructure
- [x] `strop watch --once` applies profile and exits
- [x] Dongle replug triggers profile re-apply
- [x] `strop agent install` creates a running launchd job
- [x] `strop console` works as a REPL

---

## Phase 5: System Tray + GUI

- [x] **Task 15:** Tray icon + context menu (tray-icon, muda, battery in tooltip)
- [x] **Task 16:** Headset tab — eframe window (EQ sliders, ANC, balance, LED, sleep)
- [x] **Task 17:** Microphone tab — Mic mute, sidetone, mic EQ sliders
- [x] **Task 18:** Window lifecycle — Close → tray, background refresh (30s timer)

### Checkpoint: GUI Parity
- [x] `strop tray` shows system tray icon with battery + slot
- [x] Settings window opens with all three tabs functional
- [x] EQ slider updates headset after 250ms debounce
- [ ] Window close hides to tray; Quit exits
- [x] Battery refreshes every 30 seconds

---

## Phase 6: Spatial Audio

- [x] **Task 19:** strop-dsp — Portable DSP crate (extract from legacy occmixer)
- [x] **Task 20:** CoreAudio backend — Process tap + aggregate + IOProc scaffold
- [x] **Task 21:** Spatial tab UI — Toggle, layout selector, status display

### Checkpoint: Spatial Audio
- [ ] `strop-dsp` crate compiles on Linux (no macOS deps)
- [ ] On macOS: spatial toggle works, binaural audio plays
- [ ] Toggle off restores original audio
- [ ] HRTF attribution documented (SADIE II, CC BY 4.0)

---

## Phase 7: Cross-Platform Ports

- [x] **Task 22:** Linux agent — systemd user service (cfg-gated)
- [x] **Task 23:** Windows agent — registry Run key (cfg-gated)
- [x] **Task 24:** Cross-platform CI + release workflow (GitHub Actions)

### Checkpoint: Cross-Platform
- [ ] `strop probe --open` works on Ubuntu 24.04
- [ ] `strop get battery` works on Windows 11
- [ ] `strop tray` shows tray on all three platforms
- [ ] CI green on macOS, Linux, Windows
