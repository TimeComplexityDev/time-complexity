use std::collections::VecDeque;

// ---------------------------------------------------------------------------
// Biquad filter — transposed direct form II
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct BiquadCoefs {
    pub b0: f64, pub b1: f64, pub b2: f64,
    pub a1: f64, pub a2: f64,
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
        let (b0, b1, b2, a0, a1, a2) = (alpha, 0.0, -alpha, 1.0 + alpha, -2.0 * w0.cos(), 1.0 - alpha);
        Self::new(BiquadCoefs {
            b0: b0 / a0, b1: b1 / a0, b2: b2 / a0,
            a1: a1 / a0, a2: a2 / a0,
        })
    }

    pub fn lowpass(sample_rate: f64, freq: f64, q: f64) -> Self {
        let w0 = 2.0 * std::f64::consts::PI * freq / sample_rate;
        let alpha = w0.sin() / (2.0 * q);
        let cs = w0.cos();
        let b0 = (1.0 - cs) / 2.0;
        let b1 = 1.0 - cs;
        let b2 = (1.0 - cs) / 2.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cs;
        let a2 = 1.0 - alpha;
        Self::new(BiquadCoefs {
            b0: b0 / a0, b1: b1 / a0, b2: b2 / a0,
            a1: a1 / a0, a2: a2 / a0,
        })
    }

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
// Parabolic interpolation for sub-sample peak location
// ---------------------------------------------------------------------------

pub fn parabolic_peak(y_prev: f64, y_curr: f64, y_next: f64) -> f64 {
    let denom = 2.0 * (2.0 * y_curr - y_prev - y_next);
    if denom.abs() < 1e-12 { return 0.0; }
    (y_prev - y_next) / denom
}

// ---------------------------------------------------------------------------
// Tick event
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct TickEvent {
    pub tick_index: u64,
    pub sample_index: u64,
    pub fractional_offset: f64,
    pub amplitude: f64,
    pub interval: f64,
}

// ---------------------------------------------------------------------------
// Adaptive threshold — two alternating direction bins
// ---------------------------------------------------------------------------

struct AdaptiveThreshold {
    window_size: usize,
    warmup_frames: u64,
    frame_count: u64,
    bin_a: VecDeque<f64>,
    bin_b: VecDeque<f64>,
    pub threshold_a: f64,
    pub threshold_b: f64,
    pub is_loud: bool,
    alpha: f64,
}

impl AdaptiveThreshold {
    fn new(window_size: usize, warmup_sec: f64, sample_rate: f64) -> Self {
        AdaptiveThreshold {
            window_size,
            warmup_frames: (warmup_sec * sample_rate) as u64,
            frame_count: 0,
            bin_a: VecDeque::with_capacity(window_size),
            bin_b: VecDeque::with_capacity(window_size),
            threshold_a: 0.01,
            threshold_b: 0.01,
            is_loud: true,
            alpha: 0.3,
        }
    }

    fn in_warmup(&self) -> bool {
        self.frame_count < self.warmup_frames
    }

    fn base_threshold(&self) -> f64 {
        0.01
    }

    fn current_threshold(&self) -> f64 {
        if self.in_warmup() { return self.base_threshold(); }
        if self.is_loud { self.threshold_a } else { self.threshold_b }
    }

    fn record_peak(&mut self, amplitude: f64) -> f64 {
        self.frame_count = self.frame_count.saturating_add(1);

        let bin = if self.is_loud { &mut self.bin_a } else { &mut self.bin_b };
        bin.push_back(amplitude);
        if bin.len() > self.window_size { bin.pop_front(); }

        let max_val = bin.iter().cloned().fold(0.0_f64, f64::max);
        let new_threshold = (max_val * self.alpha).max(self.base_threshold());

        let result = if self.is_loud {
            self.threshold_a = new_threshold;
            self.threshold_a
        } else {
            self.threshold_b = new_threshold;
            self.threshold_b
        };

        self.is_loud = !self.is_loud;
        result
    }
}

// ---------------------------------------------------------------------------
// Full DSP pipeline
// ---------------------------------------------------------------------------

pub struct DspPipeline {
    sample_rate: f64,
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

    pub ticks: Vec<TickEvent>,
}

impl DspPipeline {
    pub fn new(sample_rate: f64) -> Self {
        let nominal_bph = 28800.0;
        let nominal_half_period = 3600.0 / nominal_bph;
        let refractory = (0.25 * nominal_half_period * sample_rate) as usize;

        DspPipeline {
            sample_rate,
            bandpass: BiquadFilter::bandpass(sample_rate, 2000.0, 0.4),
            envelope_lowpass: BiquadFilter::lowpass(sample_rate, 200.0, 1.0),
            adaptive: AdaptiveThreshold::new(8, 0.5, sample_rate),
            refractory_samples: refractory.max(1),

            sample_index: 0,
            prev_envelope: 0.0,
            prev_prev_envelope: 0.0,
            samples_since_peak: usize::MAX,
            peak_count: 0,
            last_peak_sample: 0,

            ticks: Vec::new(),
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f64) {
        if (self.sample_rate - sample_rate).abs() > 1.0 {
            self.sample_rate = sample_rate;
            self.bandpass = BiquadFilter::bandpass(sample_rate, 1200.0, 2.0);
            self.envelope_lowpass = BiquadFilter::lowpass(sample_rate, 80.0, 1.0);
            let nominal_bph = 28800.0;
            let nominal_half_period = 3600.0 / nominal_bph;
            self.refractory_samples = (0.25 * nominal_half_period * sample_rate).max(1.0) as usize;
        }
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
            let threshold = self.adaptive.record_peak(amplitude);

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

            let tick = TickEvent {
                tick_index: self.peak_count,
                sample_index: idx,
                fractional_offset: frac,
                amplitude,
                interval,
            };

            if self.peak_count % 100 == 0 || self.peak_count <= 5 {
                println!(
                    "[tick {}] interval={:.4}s amp={:.4} threshold={:.4} frac={:.4}",
                    tick.tick_index, tick.interval, tick.amplitude, threshold, tick.fractional_offset,
                );
            }

            self.ticks.push(tick);
        }
    }
}