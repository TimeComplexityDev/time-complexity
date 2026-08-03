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
    last_frac: f64,

    /// Observed intervals for BPH auto-detect (capped at 10)
    interval_history: VecDeque<f64>,
    pub detected_bph: u32,
    pub bph_detected: bool,

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
            last_frac: 0.0,
            interval_history: VecDeque::with_capacity(10),
            detected_bph: DEFAULT_BPH,
            bph_detected: false,
            ticks: Vec::new(),
        }
    }

    fn compute_refractory(sample_rate: f64) -> usize {
        let nominal_bph = DEFAULT_BPH as f64;
        let half_period = 3600.0 / nominal_bph / 2.0;
        (REFRACTORY_FRACTION * half_period * sample_rate) as usize
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

    pub fn reset(&mut self) {
        self.bandpass = BiquadFilter::bandpass(self.sample_rate, self.config.bandpass_freq, self.config.bandpass_q);
        self.envelope_lowpass = BiquadFilter::lowpass(self.sample_rate, self.config.envelope_cutoff, 1.0);
        self.adaptive = AdaptiveThreshold::new(8, 0.5, self.sample_rate);
        self.refractory_samples = Self::compute_refractory(self.sample_rate).max(1);
        self.sample_index = 0;
        self.prev_envelope = 0.0;
        self.prev_prev_envelope = 0.0;
        self.samples_since_peak = usize::MAX;
        self.peak_count = 0;
        self.last_peak_sample = 0;
        self.last_frac = 0.0;
        self.interval_history.clear();
        self.detected_bph = DEFAULT_BPH;
        self.bph_detected = false;
        self.ticks.clear();
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
                let curr_pos = idx as f64 + frac;
                let prev_pos = self.last_peak_sample as f64 + self.last_frac;
                (curr_pos - prev_pos) / self.sample_rate
            } else {
                0.0
            };

            self.peak_count += 1;
            self.last_peak_sample = idx;
            self.last_frac = frac;

            if interval > 0.0 {
                self.interval_history.push_back(interval);
                if self.interval_history.len() > 10 {
                    self.interval_history.pop_front();
                }
            }

            // Auto-detect BPH from first 10 intervals (half-periods)
            if self.peak_count == 10 {
                let avg_interval: f64 = self.interval_history.iter().sum::<f64>()
                    / self.interval_history.len() as f64;
                // Double half-period to full period for BPH calculation
                self.detected_bph = nearest_bph(
                    avg_interval * self.sample_rate * 2.0,
                    self.sample_rate,
                );
                self.bph_detected = true;
                // Update refractory period to match detected BPH
                let half_period = 3600.0 / self.detected_bph as f64 / 2.0;
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

// ---------------------------------------------------------------------------
// Tick simulator — generates synthetic watch tick audio on the fly
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub struct SimulatorParams {
    pub bph: u16,
    pub drift_s_per_day: f64,
    pub beat_error_ms: f64,
}

impl Default for SimulatorParams {
    fn default() -> Self {
        SimulatorParams {
            bph: 21600,
            drift_s_per_day: 0.0,
            beat_error_ms: 0.0,
        }
    }
}

pub struct TickSimulator {
    sample_rate: u32,
    current_sample: u64,
    beat_count: u64,
    actual_period: f64,
    beat_error_sec: f64,
    next_tick_exact: f64,
    burst_remaining: usize,
    tick_duration_samples: usize,
}

impl TickSimulator {
    pub fn new(params: SimulatorParams, sample_rate: u32) -> Self {
        let beats_per_sec = params.bph as f64 / 3600.0;
        let nominal_half_period = 1.0 / beats_per_sec / 2.0;
        let time_scale = 1.0 + (params.drift_s_per_day / 86400.0);
        let actual_half_period = nominal_half_period * time_scale;
        let beat_error_sec = params.beat_error_ms / 1000.0;
        let tick_duration_samples = (sample_rate as f64 * 0.004).round() as usize;

        TickSimulator {
            sample_rate,
            current_sample: 0,
            beat_count: 0,
            actual_period: actual_half_period,
            beat_error_sec,
            next_tick_exact: 0.0,
            burst_remaining: 0,
            tick_duration_samples,
        }
    }

    pub fn next_sample(&mut self) -> Option<f32> {
        if self.burst_remaining > 0 {
            let burst_pos = self.tick_duration_samples - self.burst_remaining;
            let t = burst_pos as f64 / self.sample_rate as f64;
            let envelope = (-t * 1200.0).exp();
            let sample = (2.0 * std::f64::consts::PI * 5000.0 * t).sin() * envelope * 0.8;
            self.burst_remaining -= 1;
            self.current_sample += 1;
            return Some(sample as f32);
        }

        if (self.current_sample as f64) >= self.next_tick_exact.round() {
            self.burst_remaining = self.tick_duration_samples;
            let interval = if self.beat_count % 2 == 0 {
                self.actual_period + self.beat_error_sec / 2.0
            } else {
                self.actual_period - self.beat_error_sec / 2.0
            };
            self.beat_count += 1;
            self.next_tick_exact += interval * self.sample_rate as f64;
            return self.next_sample();
        }

        self.current_sample += 1;
        Some(0.0)
    }

    pub fn generate_samples(&mut self, buffer: &mut [f32]) -> usize {
        for i in 0..buffer.len() {
            match self.next_sample() {
                Some(s) => buffer[i] = s,
                None => return i,
            }
        }
        buffer.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::{MetricsEngine, TickEventMessage};

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

    #[test]
    fn test_simulator_21600_bph() {
        let sr = 44100;
        let mut sim = TickSimulator::new(
            SimulatorParams { bph: 21600, drift_s_per_day: 0.0, beat_error_ms: 0.0 },
            sr,
        );
        let mut p = DspPipeline::new(sr as f64);

        let total_samples = (sr as f64 * 5.0) as usize;
        let mut samples = vec![0.0f32; total_samples];
        sim.generate_samples(&mut samples);
        p.process_samples(&samples);

        let total_expected = (21600.0_f64 / 3600.0 * 2.0 * 5.0).ceil() as usize;
        let detected = p.ticks.len();
        assert!(
            (detected as isize - total_expected as isize).abs() <= 2,
            "expected ~{} ticks from 21600 BPH @ 5s, got {}",
            total_expected,
            detected,
        );

        assert_eq!(p.detected_bph, 21600, "BPH should be detected as 21600");
    }

    #[test]
    fn test_simulator_zero_drift_rate() {
        let sr = 44100;
        let mut sim = TickSimulator::new(
            SimulatorParams { bph: 21600, drift_s_per_day: 0.0, beat_error_ms: 0.0 },
            sr,
        );
        let mut p = DspPipeline::new(sr as f64);

        let total_samples = (sr as f64 * 5.0) as usize;
        let mut samples = vec![0.0f32; total_samples];
        sim.generate_samples(&mut samples);
        p.process_samples(&samples);

        let nominal_interval = 1.0 / (21600.0 / 3600.0) / 2.0;

        for tick in p.ticks.iter().skip(1) {
            let rate_spd = (tick.interval - nominal_interval) / nominal_interval * 86400.0;
            assert!(
                rate_spd.abs() < 0.1,
                "tick {} interval {:.6}s > nominal {:.6}s, rate={:+.2} s/d (expected ~0)",
                tick.tick_index,
                tick.interval,
                nominal_interval,
                rate_spd,
            );
        }
    }

    #[test]
    fn test_simulator_28800_bph_zero_drift() {
        let sr = 48000;
        let mut sim = TickSimulator::new(
            SimulatorParams { bph: 28800, drift_s_per_day: 0.0, beat_error_ms: 0.0 },
            sr,
        );
        let mut p = DspPipeline::new(sr as f64);

        let total_samples = (sr as f64 * 5.0) as usize;
        let mut samples = vec![0.0f32; total_samples];
        sim.generate_samples(&mut samples);
        p.process_samples(&samples);

        assert!(p.ticks.len() >= 70, "expected at least 70 ticks for 28800 BPH @ 5s");

        let nominal_interval = 1.0 / (28800.0 / 3600.0) / 2.0;

        for tick in p.ticks.iter().skip(1) {
            let rate_spd = (tick.interval - nominal_interval) / nominal_interval * 86400.0;
            assert!(
                rate_spd.abs() < 0.1,
                "tick {}: interval {:.6}s, rate={:+.2} s/d",
                tick.tick_index,
                tick.interval,
                rate_spd,
            );
        }
    }

    #[test]
    fn test_simulator_drift_detection() {
        let sr = 44100;
        let mut sim = TickSimulator::new(
            SimulatorParams { bph: 21600, drift_s_per_day: 12.0, beat_error_ms: 0.0 },
            sr,
        );
        let mut p = DspPipeline::new(sr as f64);

        let total_samples = (sr as f64 * 10.0) as usize;
        let mut samples = vec![0.0f32; total_samples];
        sim.generate_samples(&mut samples);
        p.process_samples(&samples);

        let nominal_interval = 1.0 / (21600.0 / 3600.0) / 2.0;
        let detected = p.ticks.len();
        assert!(detected > 100, "should detect many ticks, got {}", detected);

        let mean_interval: f64 = p.ticks.iter().skip(1).map(|t| t.interval).sum::<f64>()
            / (p.ticks.len().saturating_sub(1)) as f64;

        assert!(
            mean_interval > nominal_interval,
            "drift of +12 s/day should increase interval (got {:.6}s, nominal {:.6}s)",
            mean_interval,
            nominal_interval,
        );

        let mean_rate = (mean_interval - nominal_interval) / nominal_interval * 86400.0;
        assert!(
            (mean_rate - 12.0).abs() < 3.0,
            "expected mean rate ~+12 s/d, got {:.2}",
            mean_rate,
        );
    }

    // -----------------------------------------------------------------------
    // Helpers for comprehensive integration tests
    // -----------------------------------------------------------------------

    /// Pick a sample rate where the half-period in samples is an integer.
    fn compatible_sr(bph: u16) -> u32 {
        match bph {
            18000 => 44100,  // 44100/10 = 4410 samples/half-period
            19800 => 44000,  // 44000/11 = 4000 samples/half-period
            21600 => 44100,  // 44100/12 = 3675 samples/half-period
            25200 => 44100,  // 44100/14 = 3150 samples/half-period
            28800 => 48000,  // 48000/16 = 3000 samples/half-period
            36000 => 44100,  // 44100/20 = 2205 samples/half-period
            _ => 44100,
        }
    }

    struct SimResult {
        ticks: Vec<TickEvent>,
        detected_bph: u32,
        bph_detected: bool,
    }

    fn run_simulation(bph: u16, drift_spd: f64, beat_error_ms: f64, sr: u32, duration_sec: f64) -> SimResult {
        let mut sim = TickSimulator::new(
            SimulatorParams { bph, drift_s_per_day: drift_spd, beat_error_ms },
            sr,
        );
        let mut p = DspPipeline::new(sr as f64);
        let total_samples = (sr as f64 * duration_sec) as usize;
        let mut samples = vec![0.0f32; total_samples];
        sim.generate_samples(&mut samples);
        p.process_samples(&samples);
        SimResult {
            ticks: p.ticks,
            detected_bph: p.detected_bph,
            bph_detected: p.bph_detected,
        }
    }

    /// Full-pipeline simulation result processed through MetricsEngine.
    struct MetricsSimResult {
        tick_msgs: Vec<TickEventMessage>,
    }

    fn run_simulation_with_metrics(
        bph: u16, drift_spd: f64, beat_error_ms: f64,
        sr: u32, duration_sec: f64,
    ) -> MetricsSimResult {
        let sim = run_simulation(bph, drift_spd, beat_error_ms, sr, duration_sec);
        let mut engine = MetricsEngine::new("test-sim".into(), sr as f64);
        engine.set_bph(sim.detected_bph);
        engine.ingest_ticks(&sim.ticks);
        let (tick_msgs, _aggregate) = engine.drain_messages();
        MetricsSimResult { tick_msgs }
    }

    // -----------------------------------------------------------------------
    // BPH auto-detection — every common BPH
    // -----------------------------------------------------------------------

    #[test]
    fn test_bph_auto_detect_all_common() {
        for &bph in &[18000u16, 19800, 21600, 25200, 28800, 36000] {
            let sr = compatible_sr(bph);
            let result = run_simulation(bph, 0.0, 0.0, sr, 5.0);
            assert!(result.bph_detected, "BPH auto-detect should complete for {} BPH", bph);
            assert_eq!(
                result.detected_bph, bph as u32,
                "BPH auto-detect mismatch for {} BPH (got {})",
                bph, result.detected_bph,
            );
            assert!(result.ticks.len() >= 5, "too few ticks for {} BPH: {}", bph, result.ticks.len());
        }
    }

    // -----------------------------------------------------------------------
    // Zero drift — all common BPH values, per-tick rate accuracy
    // -----------------------------------------------------------------------

    #[test]
    fn test_zero_drift_all_common_bph() {
        for &bph in &[18000u16, 19800, 21600, 25200, 28800, 36000] {
            let sr = compatible_sr(bph);
            let result = run_simulation(bph, 0.0, 0.0, sr, 5.0);
            let nominal = 3600.0 / bph as f64 / 2.0;

            for tick in result.ticks.iter().skip(1) {
                let rate = (tick.interval - nominal) / nominal * 86400.0;
                assert!(
                    rate.abs() < 0.5,
                    "{} BPH zero drift: tick {} rate={:+.2} s/d (interval {:.6}s, nominal {:.6}s)",
                    bph, tick.tick_index, rate, tick.interval, nominal,
                );
            }
        }
    }

    // -----------------------------------------------------------------------
    // Known drift values — all common BPH, per-tick rate matches
    // -----------------------------------------------------------------------

    #[test]
    fn test_drift_rates_all_common_bph() {
        // Use higher sample rates to minimize simulator quantization error.
        // At sr/tick_samples, the quantization is ±0.5 samples per tick,
        // which translates to ±0.5 / tick_samples * 86400 s/d.
        let drifts: &[(f64, f64)] = &[(5.0, 3.0), (-5.0, 3.0), (12.0, 5.0), (-10.0, 5.0)];

        for &bph in &[18000u16, 21600, 28800, 36000] {
            // Use 96000 Hz for all drift tests — better quantization precision
            let sr = match bph {
                18000 => 96000u32,  // 96000/5 = 19200 samples/tick
                19800 => 88000u32,  // 88000/5.5 = 16000 samples/tick
                21600 => 96000u32,  // 96000/6 = 16000 samples/tick
                25200 => 88200u32,  // 88200/7 = 12600 samples/tick
                28800 => 96000u32,  // 96000/8 = 12000 samples/tick
                36000 => 96000u32,  // 96000/10 = 9600 samples/tick
                _ => 96000,
            };
            let nominal = 3600.0 / bph as f64 / 2.0;
            let samples_per_tick = sr as f64 / (bph as f64 / 3600.0) / 2.0;

            for &(drift_spd, base_tol) in drifts {
                let result = run_simulation(bph, drift_spd, 0.0, sr, 8.0);
                let ticks = &result.ticks;

                let data_ticks: Vec<&TickEvent> = ticks.iter().skip(5).collect();
                if data_ticks.is_empty() {
                    continue;
                }

                let mean_interval: f64 = data_ticks.iter().map(|t| t.interval).sum::<f64>()
                    / data_ticks.len() as f64;
                let mean_rate = (mean_interval - nominal) / nominal * 86400.0;

                // Tolerance: base tolerance + quantization uncertainty
                // (±0.5 sample per tick expressed in s/d)
                let quanta = 0.5 / samples_per_tick * 86400.0;
                let tol = base_tol + quanta * 1.5;

                assert!(
                    (mean_rate - drift_spd).abs() < tol,
                    "{} BPH + {:.0} s/d drift: mean_rate={:.2} s/d, expected ~{:.0} (tol {:.1}, quanta {:.2})",
                    bph, drift_spd, mean_rate, drift_spd, tol, quanta,
                );
            }
        }
    }

    // -----------------------------------------------------------------------
    // Beat error detection — known beat error, measured beat error matches
    // -----------------------------------------------------------------------

    #[test]
    fn test_beat_error_detection() {
        for &bph in &[18000u16, 21600, 28800] {
            let sr = compatible_sr(bph);
            let result = run_simulation(bph, 0.0, 1.5, sr, 8.0);
            let ticks = &result.ticks;

            let diffs: Vec<f64> = ticks.iter().skip(1)
                .map(|t| t.interval)
                .collect();

            if diffs.len() < 4 {
                continue;
            }

            // Beat error = |tic - toc| = |even_interval - odd_interval|
            let mut beat_errors = Vec::new();
            for pair_ix in (0..diffs.len() - 1).step_by(2) {
                if pair_ix + 1 < diffs.len() {
                    beat_errors.push((diffs[pair_ix] - diffs[pair_ix + 1]).abs());
                }
            }

            if beat_errors.is_empty() {
                continue;
            }
            let mean_beat_error_s: f64 = beat_errors.iter().sum::<f64>() / beat_errors.len() as f64;
            let mean_beat_error_ms = mean_beat_error_s * 1000.0;

            assert!(
                (mean_beat_error_ms - 1.5).abs() < 1.0,
                "{} BPH beat error: measured {:.2} ms, expected ~1.5 ms",
                bph, mean_beat_error_ms,
            );
        }
    }

    // -----------------------------------------------------------------------
    // Measurement cycle with reset — simulate full position readings
    // -----------------------------------------------------------------------

    #[test]
    fn test_reset_clears_pipeline_state() {
        // Measurement 1: 21600 BPH, +5 s/d
        let sr = 44100;
        let mut sim = TickSimulator::new(
            SimulatorParams { bph: 21600, drift_s_per_day: 5.0, beat_error_ms: 0.0 },
            sr,
        );
        let mut p = DspPipeline::new(sr as f64);
        let mut buf = vec![0.0f32; (sr as f64 * 5.0) as usize];
        sim.generate_samples(&mut buf);
        p.process_samples(&buf);

        assert_eq!(p.detected_bph, 21600);
        assert!(p.bph_detected);
        assert!(p.ticks.len() >= 50);

        // Reset — as if user started a new measurement
        p.reset();

        assert!(p.ticks.is_empty(), "ticks not cleared");
        assert!(!p.bph_detected, "bph_detected not reset");
        assert_eq!(p.detected_bph, 28800, "detected_bph not reset to default");
        assert_eq!(p.peak_count, 0, "peak_count not reset");

        // Measurement 2: 28800 BPH, -3 s/d drift
        let mut sim2 = TickSimulator::new(
            SimulatorParams { bph: 28800, drift_s_per_day: -3.0, beat_error_ms: 0.0 },
            sr,
        );
        sim2.generate_samples(&mut buf);
        p.process_samples(&buf);

        assert_eq!(
            p.detected_bph, 28800,
            "after reset, second measurement should detect 28800 BPH, got {}",
            p.detected_bph,
        );
        assert!(p.bph_detected);

        let nominal = 3600.0 / 28800.0 / 2.0;
        let mean_interval: f64 = p.ticks.iter().skip(1).map(|t| t.interval).sum::<f64>()
            / (p.ticks.len().saturating_sub(1)) as f64;
        let mean_rate = (mean_interval - nominal) / nominal * 86400.0;
        assert!(
            mean_rate < 0.0,
            "after reset, -3 s/d drift should give negative rate, got {:.2}",
            mean_rate,
        );
    }

    // -----------------------------------------------------------------------
    // Full-pipeline through MetricsEngine — validates rate computation
    // -----------------------------------------------------------------------

    #[test]
    fn test_full_pipeline_through_metrics_engine() {
        let result = run_simulation_with_metrics(21600, 0.0, 0.0, 44100, 8.0);

        assert!(!result.tick_msgs.is_empty(), "should produce tick messages");

        // With zero drift, every tick message's rate should be ~0
        for msg in &result.tick_msgs {
            assert!(
                msg.rate_spd.abs() < 0.5,
                "tick {}: rate={:+.2} s/d (expected ~0 for zero-drift)",
                msg.tick_index,
                msg.rate_spd,
            );
        }
    }

    #[test]
    fn test_full_pipeline_drift_through_metrics_engine() {
        let result = run_simulation_with_metrics(21600, 8.0, 0.0, 96000, 10.0);

        assert!(!result.tick_msgs.is_empty(), "should produce tick messages");

        let mut rates: Vec<f64> = result.tick_msgs.iter().map(|m| m.rate_spd).collect();
        rates.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median_rate = rates[rates.len() / 2];

        assert!(
            (median_rate - 8.0).abs() < 4.0,
            "median rate={:.2} s/d, expected ~8 s/d",
            median_rate,
        );
    }
}