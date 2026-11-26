//! CoreAudio spatial audio backend for strop.
//!
//! Captures macOS system audio via a process tap (macOS 14.2+), renders it
//! through the strop-dsp binaural pipeline, and plays to the headset.
//!
//! ## Architecture
//!
//! 1. Find the headset output device by name substring
//! 2. Create a ProcessTap excluding our PID
//! 3. Wrap the tap in a private AppTapAggregateDevice
//! 4. Register an IOProc on the aggregate
//! 5. Start audio delivery
//! 6. On drop, everything cleans up automatically

#![cfg(target_os = "macos")]

use coreaudio::app_tap_aggregate::AppTapAggregateDevice;
use coreaudio::object::{AudioObject, Device, System};
use coreaudio::tap::{ProcessTap, TapMute};
use coreaudio::Scope;
use strop_dsp::{Pipeline, SURROUND_71, SURROUND_714};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Error type.
#[derive(Debug)]
pub enum SpatialError {
    CoreAudio(String),
    Dsp(String),
    DeviceNotFound(String),
    Unsupported(String),
}

impl std::fmt::Display for SpatialError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CoreAudio(s) => write!(f, "coreaudio: {s}"),
            Self::Dsp(s) => write!(f, "dsp: {s}"),
            Self::DeviceNotFound(s) => write!(f, "device not found: {s}"),
            Self::Unsupported(s) => write!(f, "unsupported: {s}"),
        }
    }
}

impl std::error::Error for SpatialError {}

impl From<coreaudio::errors::CoreAudioError> for SpatialError {
    fn from(e: coreaudio::errors::CoreAudioError) -> Self {
        Self::CoreAudio(e.to_string())
    }
}

/// Speaker layout.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AudioLayout {
    Surround71,
    Surround714,
}

/// Spatial audio controller. Call [`start`]/[`stop`] to toggle rendering.
pub struct SpatialAudio {
    guard: Option<SpatialGuard>,
    output_name: String,
    layout: AudioLayout,
}

impl SpatialAudio {
    /// Create a new controller for the given headset output device.
    pub fn new(output_device_name: impl Into<String>) -> Self {
        Self {
            guard: None,
            output_name: output_device_name.into(),
            layout: AudioLayout::Surround71,
        }
    }

    pub fn set_layout(&mut self, layout: AudioLayout) {
        self.layout = layout;
    }

    pub fn is_running(&self) -> bool {
        self.guard.is_some()
    }

    /// Start spatial audio rendering.
    pub fn start(&mut self) -> Result<(), SpatialError> {
        if self.guard.is_some() {
            return Err(SpatialError::CoreAudio("already running".into()));
        }
        let guard = SpatialGuard::new(&self.output_name, self.layout)?;
        self.guard = Some(guard);
        Ok(())
    }

    /// Stop spatial audio rendering.
    pub fn stop(&mut self) {
        self.guard = None;
    }
}

/// RAII guard: dropping stops audio and cleans up CoreAudio resources.
struct SpatialGuard {
    _tap: ProcessTap,
    _agg: AppTapAggregateDevice,
    _ioproc: coreaudio::io_proc::IOProc,
    _running: Arc<AtomicBool>,
}

impl SpatialGuard {
    fn new(output_name: &str, layout: AudioLayout) -> Result<Self, SpatialError> {
        // 1. Find headset output device by name.
        let headset = find_device(output_name)?;
        let headset_id = headset.id();

        // Print the headset name for diagnostics.
        println!("  device   found (id {headset_id})");

        // 2. Create process tap excluding our process.
        let our_pid = std::process::id();
        let system = AudioObject::<System>::default();
        let processes = system
            .processes()
            .map_err(|e| SpatialError::CoreAudio(format!("processes: {e}")))?;
        let our_proc = processes
            .into_iter()
            .find(|p| {
                let pid: i32 = p
                    .get_property(coreaudio::PROCESS_PID)
                    .unwrap_or(0);
                pid > 0 && pid == our_pid as i32
            })
            .ok_or_else(|| {
                SpatialError::CoreAudio("our process not found in CoreAudio process list".into())
            })?;

        let tap =
            ProcessTap::stereo_mixdown(&[our_proc], "strop-spatial", TapMute::Muted)
                .map_err(|e| SpatialError::CoreAudio(format!("create tap: {e}")))?;
        let tap_uid = tap.uid().to_string();
        println!("  tap      {tap_uid}");

        // 3. Wrap in aggregate device.
        let agg = AppTapAggregateDevice::with_tap(
            "strop-spatial",
            "io.strop.spatial",
            &tap_uid,
        )
        .map_err(|e| SpatialError::CoreAudio(format!("aggregate: {e}")))?;
        let agg_dev = agg.device();
        let _agg_id = agg_dev.id();
        println!("  agg      created");

        // 4. Build DSP pipeline at 48 kHz (standard CoreAudio rate).
        let sample_rate = 48000.0_f32;
        let speakers = match layout {
            AudioLayout::Surround71 => &SURROUND_71[..],
            AudioLayout::Surround714 => &SURROUND_714[..],
        };
        let dsp_set =
            strop_dsp::Set::load().map_err(|e| SpatialError::Dsp(e))?;
        let mut pipeline = Pipeline::new(speakers, &dsp_set, sample_rate, 1.0)
            .map_err(|e| SpatialError::Dsp(e))?;
        println!("  dsp      {} speakers, {:.2}x norm", speakers.len(), pipeline.norm());

        // 5. Register IOProc on the aggregate device.
        // The aggregate's input carries the tap audio.
        // The callback reads from input, runs DSP, writes to output.
        let running = Arc::new(AtomicBool::new(true));
        let r = running.clone();

        let mut ioproc = agg_dev
            .add_io_proc(Scope::Output, move |buffers| {
                if !r.load(Ordering::SeqCst) {
                    return;
                }
                if let Some(buf) = buffers.first_mut() {
                    let outp = &mut *buf.data;
                    let n = outp.len() / 2;
                    for i in 0..n {
                        let (l, r) = pipeline.frame(outp[i * 2], outp[i * 2 + 1]);
                        let (l, r) = (strop_dsp::soft_clip(l), strop_dsp::soft_clip(r));
                        outp[i * 2] = l;
                        outp[i * 2 + 1] = r;
                    }
                }
            })
            .map_err(|e| SpatialError::CoreAudio(format!("ioproc: {e}")))?;

        ioproc
            .play()
            .map_err(|e| SpatialError::CoreAudio(format!("play: {e}")))?;

        println!("  running  pipeline active");
        Ok(Self {
            _tap: tap,
            _agg: agg,
            _ioproc: ioproc,
            _running: running,
        })
    }
}

fn find_device(name_substring: &str) -> Result<AudioObject<Device>, SpatialError> {
    let system = AudioObject::<System>::default();
    let devices = system
        .devices_with_scope(Scope::Output)
        .map_err(|e| SpatialError::CoreAudio(format!("enum devices: {e}")))?;
    for device in &devices {
        if let Ok(name) = device.get_property(coreaudio::DEVICE_NAME) {
            if name.contains(name_substring) {
                return Ok(device.clone());
            }
        }
    }
    Err(SpatialError::DeviceNotFound(format!(
        "no output device matching \"{name_substring}\""
    )))
}
