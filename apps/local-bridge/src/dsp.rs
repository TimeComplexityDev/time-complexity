use std::collections::VecDeque;

// ---------------------------------------------------------------------------
// Common BPH values used by mechanical watches
// ---------------------------------------------------------------------------

pub const COMMON_BPH: &[u32] = &[18000, 19800, 21600, 25200, 28800, 36000];
pub const DEFAULT_BPH: u32 = 28800;
pub const REFRACTORY_FRACTION: f64 = 0.25;

pub fn nearest_bph(interval_samples: f64, sample_rate: f64) -> u32 {
    if interval_samples <= 0.0 {
        return DEFAULT_BPH;
    }
    let interval_sec = interval_samples / sample_rate;
    let measured_bph = (3600.0 / interval_sec).round() as u32;
    *COMMON_BPH
        .iter()
        .min_by_key(|&&bph| (bph as i64 - measured_bph as i64).abs())
        .unwrap_or(&DEFAULT_BPH)
}

// ---------------------------------------------------------------------------
// Biquad filter — transposed direct form II
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct BiquadCoefs {
    pub b0: f64, pub b1: f64, pub b2: f64,
    pub a1: f64, pub a2: f64,
}

impl BiquadCoefs {
    fn normalize(b0: f64, b1: f64, b2: f64, a0: f64, a1: f64, a2: f64) -> Self {
        BiquadCoefs {
            b0: b0 / a0, b1: b1 / a0, b2: b2 / a0,
            a1: a1 / a0, a2: a2 / a0,
        }
    }
}

#[derive(Clone)]
pub struct BiquadFilter {
    coefs: BiquadCoefs,
    x1: f64, x2: f64,
    y1: f64, y2: f64,
}

impl BiquadFilter {
    fn new(coefs: BiquadCoefs) -> Self {
        BiquadFilter { coefs, x1: 0.0, x2: 0.0, y1: 0.0, y2: 0.0 }
    }

    pub fn bandpass(sample_rate: f64, freq: f64, q: f64) -> Self {
        let w0 = 2.0 * std::f64::consts::PI * freq / sample_rate;
        let alpha = w0.sin() / (2.0 * q);
        Self::new(BiquadCoefs::normalize(
            alpha, 0.0, -alpha,
            1.0 + alpha, -2.0 * w0.cos(), 1.0 - alpha,
        ))
    }

    pub fn lowpass(sample_rate: f64, freq: f64, q: f64) -> Self {
        let w0 = 2.0 * std::f64::consts::PI * freq / sample_rate;
        let alpha = w0.sin() / (2.0 * q);
        let cs = w0.cos();
        Self::new(BiquadCoefs::normalize(
            (1.0 - cs) / 2.0, 1.0 - cs, (1.0 - cs) / 2.0,
            1.0 + alpha, -2.0 * cs, 1.0 - alpha,
        ))
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.x1 = 0.0; self.x2 = 0.0;
        self.y1 = 0.0; self.y2 = 0.0;
    }

    pub fn process(&mut self, x: f64) -> f64 {
        let y = self.coefs.b0 * x + self.coefs.b1 * self.x1 + self.coefs.b2 * self.x2
            - self.coefs.a1 * self.y1 - self.coefs.a2 * self.y2;
        self.x2 = self.x1; self.x1 = x;
        self.y2 = self.y1; self.y1 = y;
        y
    }
}

// ---------------------------------------------------------------------------
// Sub-sample interpolation (quadratic/parabolic)
// Derived from the architecture plan formula:
//   δ = 0.5 · (α - γ) / (α - 2β + γ)
// where α, β, γ are the three envelope samples around the peak.
// Returns offset in fraction of a sample, clamped to [-0.5, 0.5].
// ---------------------------------------------------------------------------

pub fn parabolic_peak(y_prev: f64, y_curr: f64, y_next: f64) -> f64 {
    let numer = y_prev - y_next;
    let denom = y_prev - 2.0 * y_curr + y_next;
    if denom.abs() < 1e-12 { return 0.0; }
    let delta = 0.5 * numer / denom;
    delta.clamp(-0.5, 0.5)
}

// ---------------------------------------------------------------------------
// Tick event
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct TickEvent {
    pub tick_index: u64,
    #[allow(dead_code)]
    pub sample_index: u64,
    #[allow(dead_code)]
    pub fractional_offset: f64,
    pub amplitude: f64,
    pub interval: f64,
    pub timestamp: f64,
}

// ---------------------------------------------------------------------------
// Adaptive threshold — two alternating direction bins
// ---------------------------------------------------------------------------

struct AdaptiveThreshold {
    window_size: usize,
    warmup_samples: u64,
    sample_count: u64,
    bin_a: VecDeque<f64>,
    bin_b: VecDeque<f64>,
    threshold_a: f64,
    threshold_b: f64,
    use_bin_a: bool,
    alpha: f64,
}

impl AdaptiveThreshold {
    fn new(window_size: usize, warmup_sec: f64, sample_rate: f64) -> Self {
        AdaptiveThreshold {
            window_size,
            warmup_samples: (warmup_sec * sample_rate) as u64,
            sample_count: 0,
            bin_a: VecDeque::with_capacity(window_size),
            bin_b: VecDeque::with_capacity(window_size),
            threshold_a: 0.01,
            threshold_b: 0.01,
            use_bin_a: true,
            alpha: 0.3,
        }
    }

    fn in_warmup(&self) -> bool {
        self.sample_count < self.warmup_samples
    }

    fn base_threshold(&self) -> f64 {
        0.01
    }

    fn current_threshold(&self) -> f64 {
        if self.in_warmup() { return self.base_threshold(); }
        if self.use_bin_a { self.threshold_a } else { self.threshold_b }
    }

    fn record_peak(&mut self, amplitude: f64) -> f64 {
        self.sample_count = self.sample_count.saturating_add(1);

        let bin = if self.use_bin_a { &mut self.bin_a } else { &mut self.bin_b };
        bin.push_back(amplitude);
        if bin.len() > self.window_size { bin.pop_front(); }

        let max_val = bin.iter().cloned().fold(0.0_f64, f64::max);
        let new_threshold = (max_val * self.alpha).max(self.base_threshold());

        let result = if self.use_bin_a {
            self.threshold_a = new_threshold;
            self.threshold_a
        } else {
            self.threshold_b = new_threshold;
            self.threshold_b
        };

        self.use_bin_a = !self.use_bin_a;
        result
    }
}

// ---------------------------------------------------------------------------
// Full DSP pipeline
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct DspPipelineConfig {
    pub bandpass_freq: f64,
    pub bandpass_q: f64,
    pub envelope_cutoff: f64,
}

impl Default for DspPipelineConfig {
    fn default() -> Self {
        DspPipelineConfig {
            bandpass_freq: 2000.0,
            bandpass_q: 0.4,
            envelope_cutoff: 200.0,
        }
    }
}

pub struct DspPipeline {
    sample_rate: f64,
    config: DspPipelineConfig,
    bandpass: BiquadFilter,
    envelope_lowpass: BiquadFilter,
    adaptive: AdaptiveThreshold,
    refractory_samples: usize,

    sample_index: u64,
    prev_envelope: f64,
    prev_prev_envelope: f64,
    samples_since_peak: usize,
    peak_count: u64,
    last_peak_sample: u64,

    /// Observed intervals for BPH auto-detect (capped at 10)
    interval_history: VecDeque<f64>,
    pub detected_bph: u32,

    pub ticks: Vec<TickEvent>,
}

impl DspPipeline {
    pub fn new(sample_rate: f64) -> Self {
        Self::with_config(sample_rate, DspPipelineConfig::default())
    }

    pub fn with_config(sample_rate: f64, config: DspPipelineConfig) -> Self {
        let refractory = Self::compute_refractory(sample_rate);
        DspPipeline {
            sample_rate,
            config: config.clone(),
            bandpass: BiquadFilter::bandpass(sample_rate, config.bandpass_freq, config.bandpass_q),
            envelope_lowpass: BiquadFilter::lowpass(sample_rate, config.envelope_cutoff, 1.0),
            adaptive: AdaptiveThreshold::new(8, 0.5, sample_rate),
            refractory_samples: refractory.max(1),
            sample_index: 0,
            prev_envelope: 0.0,
            prev_prev_envelope: 0.0,
            samples_since_peak: usize::MAX,
            peak_count: 0,
            last_peak_sample: 0,
            interval_history: VecDeque::with_capacity(10),
            detected_bph: DEFAULT_BPH,
            ticks: Vec::new(),
        }
    }

    fn compute_refractory(sample_rate: f64) -> usize {
        let nominal_bph = DEFAULT_BPH as f64;
        let nominal_half_period = 3600.0 / nominal_bph;
        (REFRACTORY_FRACTION * nominal_half_period * sample_rate) as usize
    }

    pub fn set_sample_rate(&mut self, sample_rate: f64) {
        if (self.sample_rate - sample_rate).abs() > 1.0 {
            self.sample_rate = sample_rate;
            self.bandpass = BiquadFilter::bandpass(sample_rate, self.config.bandpass_freq, self.config.bandpass_q);
            self.envelope_lowpass = BiquadFilter::lowpass(sample_rate, self.config.envelope_cutoff, 1.0);
            self.refractory_samples = Self::compute_refractory(sample_rate).max(1);
        }
    }

    pub fn set_bandpass(&mut self, freq: f64, q: f64) {
        self.config.bandpass_freq = freq;
        self.config.bandpass_q = q;
        self.bandpass = BiquadFilter::bandpass(self.sample_rate, freq, q);
    }

    pub fn process_samples(&mut self, samples: &[f32]) {
        for &raw in samples {
            let bandpassed = self.bandpass.process(raw as f64);
            let envelope = self.envelope_lowpass.process(bandpassed.abs());
            self.detect_peak(envelope);
            self.prev_prev_envelope = self.prev_envelope;
            self.prev_envelope = envelope;
            self.sample_index += 1;
        }
    }

    fn detect_peak(&mut self, envelope: f64) {
        if self.samples_since_peak < self.refractory_samples {
            self.samples_since_peak += 1;
            return;
        }

        let is_peak = self.prev_prev_envelope < self.prev_envelope
            && self.prev_envelope > envelope
            && self.prev_envelope > self.adaptive.current_threshold();

        if is_peak {
            self.samples_since_peak = 0;
            let amplitude = self.prev_envelope;
            self.adaptive.record_peak(amplitude);

            let idx = self.sample_index - 1;
            let frac = parabolic_peak(
                self.prev_prev_envelope,
                self.prev_envelope,
                envelope,
            );

            let interval = if self.peak_count > 0 {
                (idx as f64 - self.last_peak_sample as f64) / self.sample_rate
            } else {
                0.0
            };

            self.peak_count += 1;
            self.last_peak_sample = idx;

            if interval > 0.0 {
                self.interval_history.push_back(interval);
                if self.interval_history.len() > 10 {
                    self.interval_history.pop_front();
                }
            }

            // Auto-detect BPH from first 10 intervals
            if self.peak_count == 10 {
                let avg_interval: f64 = self.interval_history.iter().sum::<f64>()
                    / self.interval_history.len() as f64;
                self.detected_bph = nearest_bph(
                    avg_interval * self.sample_rate,
                    self.sample_rate,
                );
                // Update refractory period to match detected BPH
                let half_period = 3600.0 / self.detected_bph as f64;
                self.refractory_samples = (REFRACTORY_FRACTION * half_period * self.sample_rate).max(1.0) as usize;
            }

            let timestamp = (idx as f64 + frac) / self.sample_rate;

            let tick = TickEvent {
                tick_index: self.peak_count,
                sample_index: idx,
                fractional_offset: frac,
                amplitude,
                interval,
                timestamp,
            };

            self.ticks.push(tick);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parabolic_peak_symmetric() {
        // Symmetric peak at index 1: values [1, 3, 1]
        let delta = parabolic_peak(1.0, 3.0, 1.0);
        assert!((delta).abs() < 0.01, "symmetric peak should give ~0 offset");
    }

    #[test]
    fn test_parabolic_peak_shifted_right() {
        // Peak shifted right: values [1, 3, 2]
        let delta = parabolic_peak(1.0, 3.0, 2.0);
        assert!(delta > 0.0, "higher right sample means peak is to the right");
        assert!(delta <= 0.5, "clamped to 0.5");
    }

    #[test]
    fn test_parabolic_peak_shifted_left() {
        // Peak shifted left: values [2, 3, 1]
        let delta = parabolic_peak(2.0, 3.0, 1.0);
        assert!(delta < 0.0, "higher left sample means peak is to the left");
        assert!(delta >= -0.5, "clamped to -0.5");
    }

    #[test]
    fn test_parabolic_peak_flat() {
        let delta = parabolic_peak(1.0, 1.0, 1.0);
        assert!((delta).abs() < 1e-9, "flat signal gives 0");
    }

    #[test]
    fn test_nearest_bph() {
        assert_eq!(nearest_bph(1.0 / 5.0 * 44100.0, 44100.0), 18000);
        assert_eq!(nearest_bph(1.0 / 6.0 * 44100.0, 44100.0), 21600);
        assert_eq!(nearest_bph(1.0 / 8.0 * 44100.0, 44100.0), 28800);
        let interval_28800 = 3600.0 / 28800.0 * 44100.0;
        assert_eq!(nearest_bph(interval_28800, 44100.0), 28800);
    }

    #[test]
    fn test_biquad_bandpass_stability() {
        let mut bp = BiquadFilter::bandpass(44100.0, 2000.0, 0.4);
        // Feed in a short sine burst — output should stay bounded
        for i in 0..1000 {
            let x = (2.0 * std::f64::consts::PI * 2000.0 * i as f64 / 44100.0).sin();
            let y = bp.process(x);
            assert!(y.is_finite(), "biquad output diverged at sample {}", i);
        }
    }

    #[test]
    fn test_dsp_pipeline_processes_samples() {
        let mut p = DspPipeline::new(44100.0);
        let samples: Vec<f32> = (0..44100)
            .map(|i| (2.0 * std::f64::consts::PI * 2000.0 * i as f64 / 44100.0).sin() as f32)
            .collect();
        p.process_samples(&samples);
        // 1 second of 2 kHz sine at 44100 Hz should produce detectable envelope peaks
        assert!(p.ticks.len() > 0, "expected at least 1 tick from 1s of audio, got 0");
    }

    /// Generate a synthetic mechanical watch tick track.
    ///
    /// Simulates the acoustic impulse of an escapement at the given BPH with
    /// optional rate drift and beat error. Each tick is a 5 kHz sine burst
    /// with exponential decay, placed at the precise sample position given
    /// by the drift/beat-error model.
    ///
    /// Parameters:
    ///   - `bph`: beats per hour (e.g. 21600 → 6 Hz)
    ///   - `drift_s_per_day`: rate error in seconds/day (positive = watch runs fast)
    ///   - `beat_error_ms`: asymmetry between tic/tok in milliseconds
    ///   - `sample_rate`: output sample rate in Hz
    ///   - `duration_sec`: length of generated audio
    ///   - `amplitude`: peak amplitude of each tick impulse
    fn synthetic_click_train(
        bph: u32,
        drift_s_per_day: f64,
        beat_error_ms: f64,
        sample_rate: u32,
        duration_sec: f64,
        amplitude: f32,
    ) -> Vec<f32> {
        let total_samples = (sample_rate as f64 * duration_sec) as usize;
        let mut audio = vec![0.0_f32; total_samples];

        let beats_per_sec = bph as f64 / 3600.0;
        let nominal_period = 1.0 / beats_per_sec;
        let time_scale = 1.0 + (drift_s_per_day / 86400.0);
        let actual_period = nominal_period * time_scale;
        let beat_error_sec = beat_error_ms / 1000.0;

        let mut current_time = 0.0;
        let mut beat_count = 0u64;

        while current_time < duration_sec {
            let interval = if beat_count % 2 == 0 {
                actual_period + beat_error_sec / 2.0
            } else {
                actual_period - beat_error_sec / 2.0
            };

            let sample_idx = (current_time * sample_rate as f64).round() as usize;
            if sample_idx < total_samples {
                let tick_dur = (sample_rate as f64 * 0.004) as usize;
                for j in 0..tick_dur {
                    let t = j as f64 / sample_rate as f64;
                    let envelope = (-t * 1200.0).exp();
                    let sample = (2.0 * std::f64::consts::PI * 5000.0 * t).sin() * envelope * amplitude as f64;
                    let idx = sample_idx + j;
                    if idx < total_samples {
                        audio[idx] += sample as f32;
                    }
                }
            }

            current_time += interval;
            beat_count += 1;
        }

        audio
    }

    #[test]
    fn test_synthetic_click_train_21600_bph() {
        let sr = 44100;
        let mut p = DspPipeline::new(sr as f64);
        let samples = synthetic_click_train(21600, 0.0, 0.0, sr, 5.0, 0.5);

        let total_expected = (21600.0_f64 / 3600.0 * 5.0).ceil() as usize;
        p.process_samples(&samples);

        // Should detect close to the expected number of ticks
        let detected = p.ticks.len();
        assert!(
            (detected as isize - total_expected as isize).abs() <= 2,
            "expected ~{} ticks from 21600 BPH @ 5s, got {}",
            total_expected,
            detected,
        );

        // Detected BPH should be 21600
        assert_eq!(p.detected_bph, 21600, "BPH should be detected as 21600");

        // Mean interval should be approximately 1/6 = 0.1667s
        let mean_interval: f64 = p.ticks.iter().skip(1).map(|t| t.interval).sum::<f64>()
            / (p.ticks.len().saturating_sub(1)) as f64;
        let expected_interval = 1.0 / (21600.0 / 3600.0);
        let tolerance = expected_interval * 0.02; // 2% tolerance
        assert!(
            (mean_interval - expected_interval).abs() < tolerance,
            "mean interval {:.6}s, expected {:.6}s ± {:.6}s",
            mean_interval,
            expected_interval,
            tolerance,
        );
    }

    #[test]
    fn test_synthetic_click_train_drift_detection() {
        // 21600 BPH with +12 s/day drift should produce a measurably
        // longer mean interval than the nominal 0.1667s.
        let sr = 44100;
        let mut p = DspPipeline::new(sr as f64);
        let samples = synthetic_click_train(21600, 12.0, 0.0, sr, 10.0, 0.5);
        p.process_samples(&samples);

        let nominal_interval = 1.0 / (21600.0 / 3600.0); // ≈ 0.1667
        let detected = p.ticks.len();
        assert!(detected > 50, "should detect many ticks, got {}", detected);

        let mean_interval: f64 = p.ticks.iter().skip(1).map(|t| t.interval).sum::<f64>()
            / (p.ticks.len().saturating_sub(1)) as f64;

        // With +12 s/day drift the interval should be slightly above nominal
        assert!(
            mean_interval > nominal_interval,
            "drift of +12 s/day should increase interval (got {:.6}s, nominal {:.6}s)",
            mean_interval,
            nominal_interval,
        );
    }
}