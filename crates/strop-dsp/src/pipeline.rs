//! Realtime spatial audio DSP pipeline.
//!
//! Runs on the render thread, so nothing here allocates after construction.

use crate::hrir::{Response, Set};

/// One speaker position in a surround layout.
#[derive(Clone, Copy)]
pub struct Speaker {
    pub azimuth: f32,
    pub elevation: f32,
    pub lfe: bool,
}

const fn spk(azimuth: f32) -> Speaker {
    Speaker { azimuth, elevation: 0.0, lfe: false }
}

const fn spk_up(azimuth: f32, elevation: f32) -> Speaker {
    Speaker { azimuth, elevation, lfe: false }
}

const LFE: Speaker = Speaker { azimuth: 0.0, elevation: 0.0, lfe: true };

/// 7.1 surround layout (no height channels).
pub const SURROUND_71: [Speaker; 8] = [
    spk(-30.0), spk(30.0), spk(0.0), LFE,
    spk(-150.0), spk(150.0), spk(-90.0), spk(90.0),
];

/// 7.1.4 surround layout with height channels.
pub const SURROUND_714: [Speaker; 12] = [
    spk(-30.0), spk(30.0), spk(0.0), LFE,
    spk(-150.0), spk(150.0), spk(-90.0), spk(90.0),
    spk_up(-45.0, 45.0), spk_up(45.0, 45.0),
    spk_up(-135.0, 45.0), spk_up(135.0, 45.0),
];

// ---- Decorrelator ----

/// Allpass decorrelator. Prevents several outputs carrying the same signal
/// from fusing into one phantom source.
struct Allpass {
    buf: Vec<f32>,
    at: usize,
    gain: f32,
}

impl Allpass {
    fn new(rate: f32, seed: usize) -> Self {
        let ms = 7.0 + seed as f32 * 3.5;
        let n = ((ms * rate / 1000.0) as usize).max(1);
        Self { buf: vec![0.0; n], at: 0, gain: 0.6 }
    }

    fn reset(&mut self) {
        self.buf.fill(0.0);
        self.at = 0;
    }

    #[inline]
    fn process(&mut self, x: f32) -> f32 {
        let d = self.buf[self.at];
        let y = -self.gain * x + d;
        self.buf[self.at] = x + self.gain * d;
        self.at = (self.at + 1) % self.buf.len();
        y
    }
}

// ---- FIR filter (direct convolution) ----

/// Direct FIR convolution. 256 taps × 7 speakers ≈ 3.6k MACs per frame,
/// well within a single render callback.
struct Fir {
    left: Vec<f32>,
    right: Vec<f32>,
    hist: Vec<f32>,
    at: usize,
}

impl Fir {
    fn new(r: &Response) -> Self {
        Self {
            left: r.left.clone(),
            right: r.right.clone(),
            hist: vec![0.0; r.left.len()],
            at: 0,
        }
    }

    fn reset(&mut self) {
        self.hist.fill(0.0);
        self.at = 0;
    }

    #[inline]
    fn process(&mut self, x: f32) -> (f32, f32) {
        let n = self.hist.len();
        self.hist[self.at] = x;

        let mut l = 0.0f32;
        let mut r = 0.0f32;
        let mut i = self.at;
        for k in 0..n {
            let v = self.hist[i];
            l += v * self.left[k];
            r += v * self.right[k];
            i = if i == 0 { n - 1 } else { i - 1 };
        }

        self.at += 1;
        if self.at == n {
            self.at = 0;
        }
        (l, r)
    }
}

// ---- Pipeline ----

/// The full spatial audio pipeline. Created once, called per audio frame.
pub struct Pipeline {
    speakers: Vec<Speaker>,
    firs: Vec<Option<Fir>>,
    decor: Vec<Option<Allpass>>,
    norm: f32,
    peak: f32,
    gain: f32,
}

impl Pipeline {
    /// Build a new pipeline for the given speaker layout.
    ///
    /// `gain` is a linear amplitude multiplier applied after normalisation.
    pub fn new(speakers: &[Speaker], set: &Set, rate: f32, gain: f32) -> Result<Self, String> {
        if (set.rate as f32 - rate).abs() > 0.5 {
            return Err(format!(
                "measured responses are {} Hz but the device is running at {} Hz",
                set.rate, rate as u32
            ));
        }

        let mut firs = Vec::with_capacity(speakers.len());
        let mut decor = Vec::with_capacity(speakers.len());
        let mut pairs: Vec<(i32, i32)> = Vec::new();

        for s in speakers {
            if s.lfe {
                firs.push(None);
                decor.push(None);
                continue;
            }
            let (r, off) = set.nearest(s.azimuth, s.elevation);
            if off > 1.0 {
                return Err(format!(
                    "no measured response within 1 degree of az {:+.0} el {:+.0}, nearest is {:.1} away",
                    s.azimuth, s.elevation, off
                ));
            }
            firs.push(Some(Fir::new(r)));
            decor.push(if is_frontal(s) {
                None
            } else {
                let key = (s.azimuth.abs() as i32, s.elevation as i32);
                let seed = match pairs.iter().position(|k| *k == key) {
                    Some(i) => i + 1,
                    None => {
                        pairs.push(key);
                        pairs.len()
                    }
                };
                Some(Allpass::new(rate, seed))
            });
        }

        let mut pipe = Self {
            speakers: speakers.to_vec(),
            firs,
            decor,
            norm: 1.0,
            peak: 1.0,
            gain,
        };
        let (norm, peak) = pipe.measure();
        pipe.norm = norm;
        pipe.peak = peak;
        Ok(pipe)
    }

    /// Measure the pipeline's loudness and peak characteristics.
    fn measure(&mut self) -> (f32, f32) {
        let user = std::mem::replace(&mut self.gain, 1.0);
        let mut seed = 0x2545_F491_4F6C_DD1Du64;
        let mut sum_in = 0.0f64;
        let mut sum_out = 0.0f64;
        let mut peak_in = 0.0f32;
        let mut peak_out = 0.0f32;

        const WARMUP: usize = 4096;
        const MEASURE: usize = 16384;

        for i in 0..(WARMUP + MEASURE) {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let x = ((seed >> 40) as i32 - 8388608) as f32 / 8388608.0 * 0.5;

            let (l, r) = self.frame(x, x);
            if i >= WARMUP {
                sum_in += (x * x) as f64 * 2.0;
                sum_out += (l * l + r * r) as f64;
                peak_in = peak_in.max(x.abs());
                peak_out = peak_out.max(l.abs()).max(r.abs());
            }
        }

        self.gain = user;

        // Reset state after measurement so its tail doesn't leak into real audio.
        for f in self.firs.iter_mut().flatten() {
            f.reset();
        }
        for d in self.decor.iter_mut().flatten() {
            d.reset();
        }

        if sum_out <= 0.0 || peak_in <= 0.0 {
            return (1.0, 1.0);
        }
        let norm = ((sum_in / sum_out).sqrt() as f32).clamp(0.25, 4.0);
        (norm, peak_out / peak_in * norm)
    }

    /// The loudness normalisation factor measured at construction.
    pub fn norm(&self) -> f32 {
        self.norm
    }

    /// Peak amplification after normalising, measured on noise.
    pub fn peak(&self) -> f32 {
        self.peak
    }

    /// Process one stereo frame through the pipeline.
    #[inline]
    pub fn frame(&mut self, l: f32, r: f32) -> (f32, f32) {
        let side = (l - r) * 0.5;

        let mut out_l = 0.0f32;
        let mut out_r = 0.0f32;

        for (i, s) in self.speakers.iter().enumerate() {
            if s.lfe || (s.azimuth == 0.0 && s.elevation == 0.0) {
                continue;
            }

            let mut v = if s.elevation > 0.0 {
                signed(side * 0.35, s.azimuth)
            } else if s.azimuth.abs() > 90.0 {
                signed(side * 0.5, s.azimuth)
            } else if s.azimuth.abs() == 90.0 {
                signed(side * 0.45, s.azimuth)
            } else if s.azimuth < 0.0 {
                l
            } else {
                r
            };

            if let Some(d) = self.decor[i].as_mut() {
                v = d.process(v);
            }
            if let Some(f) = self.firs[i].as_mut() {
                let (el, er) = f.process(v);
                out_l += el;
                out_r += er;
            }
        }

        let g = self.norm * self.gain;
        (out_l * g, out_r * g)
    }
}

fn signed(v: f32, azimuth: f32) -> f32 {
    if azimuth < 0.0 { -v } else { v }
}

fn is_frontal(s: &Speaker) -> bool {
    s.azimuth.abs() <= 90.0 && s.elevation == 0.0
}

/// Soft-knee limiter. Continuous in value and slope at the knee, asymptotic
/// to 1.0, so it never needs a hard clamp behind it.
#[inline]
pub fn soft_clip(x: f32) -> f32 {
    const KNEE: f32 = 0.7;
    const ROOM: f32 = 1.0 - KNEE;

    let mag = x.abs();
    if mag <= KNEE {
        return x;
    }
    let over = mag - KNEE;
    (KNEE + ROOM * over / (over + ROOM)) * x.signum()
}
