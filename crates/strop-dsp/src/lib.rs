//! Portable spatial audio DSP pipeline.
//!
//! Extracted from the legacy occmixer binary. Zero platform dependencies —
//! works on macOS, Linux, and Windows.
//!
//! The pipeline: stereo → mid/side upmix → decorrelator → HRTF convolution → sum.
//!
//! HRIR data: SADIE II Database, subject D1 (Neumann KU100), 48 kHz, 256 taps,
//! CC BY 4.0. See `hrir/LICENSE-SADIE.md`.

mod hrir;
mod pipeline;

pub use hrir::{Response, Set};
pub use pipeline::{soft_clip, Pipeline, Speaker, SURROUND_71, SURROUND_714};

#[cfg(test)]
mod tests {
    use super::*;

    fn test_pipeline(speakers: &[Speaker]) -> Pipeline {
        let set = Set::load().expect("hrir blob");
        Pipeline::new(speakers, &set, 48000.0, 1.0).expect("pipeline")
    }

    fn tone(n: usize, f: f32) -> Vec<f32> {
        (0..n)
            .map(|i| 0.5 * (2.0 * std::f32::consts::PI * f * i as f32 / 48000.0).sin())
            .collect()
    }

    #[test]
    fn centre_stays_centred() {
        let mono = tone(8192, 440.0);
        for speakers in [&SURROUND_71[..], &SURROUND_714[..]] {
            let mut p = test_pipeline(speakers);
            for i in 0..8192 {
                let (l, r) = p.frame(mono[i], mono[i]);
                assert!(i < 1024 || (l - r).abs() < 1e-6, "sample {i}: {l} left, {r} right");
            }
        }
    }

    #[test]
    fn swapping_inputs_swaps_outputs() {
        let a = tone(8192, 440.0);
        let b = tone(8192, 661.0);
        for speakers in [&SURROUND_71[..], &SURROUND_714[..]] {
            let mut p = test_pipeline(speakers);
            let mut q = test_pipeline(speakers);
            for i in 0..8192 {
                let (l, _) = p.frame(a[i], b[i]);
                let (_, r) = q.frame(b[i], a[i]);
                assert!(i < 1024 || (l - r).abs() < 1e-6, "sample {i}: {l} vs mirrored {r}");
            }
        }
    }

    #[test]
    fn centred_content_is_not_coloured() {
        let speaker_pair: &[Speaker] = &[Speaker { azimuth: -30.0, elevation: 0.0, lfe: false },
                                           Speaker { azimuth: 30.0, elevation: 0.0, lfe: false }];
        let mut bare = test_pipeline(speaker_pair);
        let mut full = test_pipeline(&SURROUND_71);
        let (bn, fnorm) = (bare.norm(), full.norm());

        let mono = tone(8192, 440.0);
        for i in 0..8192 {
            let (bl, _) = bare.frame(mono[i], mono[i]);
            let (fl, _) = full.frame(mono[i], mono[i]);
            assert!(
                i < 1024 || (bl / bn - fl / fnorm).abs() < 1e-6,
                "sample {i}: front pair {} vs 7.1 {}", bl / bn, fl / fnorm
            );
        }
    }

    #[test]
    fn soft_clip_is_transparent_below_the_knee() {
        for i in 0..=70 {
            let x = i as f32 / 100.0;
            assert_eq!(soft_clip(x), x);
            assert_eq!(soft_clip(-x), -x);
        }
    }

    #[test]
    fn soft_clip_never_reaches_full_scale() {
        for x in [0.71, 1.0, 1.5, 2.12, 10.0, 1e6] {
            assert!(soft_clip(x) < 1.0, "{x} mapped to {}", soft_clip(x));
            assert!(soft_clip(-x) > -1.0);
        }
    }

    #[test]
    fn soft_clip_is_monotonic_and_continuous() {
        let mut prev = soft_clip(0.0);
        let mut x = 0.0f32;
        while x < 4.0 {
            x += 0.001;
            let y = soft_clip(x);
            assert!(y >= prev, "not monotonic at {x}");
            assert!(y - prev < 0.002, "jump at {x}: {prev} -> {y}");
            prev = y;
        }
    }

    #[test]
    fn set_loads_with_correct_rate() {
        let set = Set::load().expect("hrir blob");
        assert_eq!(set.rate, 48000);
        assert_eq!(set.taps, 256);
        assert!(set.responses.len() >= 13, "expected at least 13 HRIR positions");
    }

    #[test]
    fn nearest_returns_reasonable_match() {
        let set = Set::load().expect("hrir blob");
        let (r, off) = set.nearest(-30.0, 0.0);
        assert!(off <= 1.0, "nearest to front-left is {off}° away");
        assert_eq!(r.left.len(), set.taps);
        assert_eq!(r.right.len(), set.taps);
    }

    #[test]
    fn pipeline_has_non_zero_gain() {
        let mut p = test_pipeline(&SURROUND_71);
        let (l, r) = p.frame(0.5, -0.5);
        // Should produce output on both ears for side content.
        assert!(l.abs() > 0.0 || r.abs() > 0.0, "pipeline produced silence");
    }

    #[test]
    fn pipeline_measures_norm() {
        let p = test_pipeline(&SURROUND_71);
        let n = p.norm();
        assert!(n > 0.0 && n < 10.0, "norm {n} is outside expected range");
    }
}
