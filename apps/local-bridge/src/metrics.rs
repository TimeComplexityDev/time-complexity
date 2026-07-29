use std::collections::VecDeque;

use crate::dsp::TickEvent;

#[derive(Debug, Clone, serde::Serialize)]
pub struct AggregateUpdate {
    pub r#type: String,
    pub session_id: String,
    pub time: String,
    pub instant_rate_spd: f64,
    pub short_avg_spd: f64,
    pub long_ewma_spd: f64,
    pub beat_error_s: f64,
    pub amplitude: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TickEventMessage {
    pub r#type: String,
    pub session_id: String,
    pub tick_index: u64,
    pub timestamp: f64,
    pub interval_s: f64,
    pub rate_spd: f64,
    pub amplitude: f64,
}

pub struct MetricsEngine {
    session_id: String,
    sample_rate: f64,
    bph: u32,
    nominal_interval: f64,
    half_periods: VecDeque<f64>,
    last_sent_tick: u64,

    short_window_rates: VecDeque<f64>,
    short_window_duration: f64,

    long_ewma: Option<f64>,
    long_ewma_tau: f64,

    last_beat_error: f64,
    last_amplitude: f64,
    last_instant_rate: f64,

    last_aggregate_time: std::time::Instant,
    aggregate_interval_ms: u64,

    pending_messages: Vec<TickEventMessage>,
}

impl MetricsEngine {
    pub fn new(session_id: String, sample_rate: f64) -> Self {
        MetricsEngine {
            session_id,
            sample_rate,
            bph: 28800,
            nominal_interval: 3600.0 / 28800.0,
            half_periods: VecDeque::with_capacity(4),
            last_sent_tick: 0,
            short_window_rates: VecDeque::new(),
            short_window_duration: 10.0,
            long_ewma: None,
            long_ewma_tau: 600.0,
            last_beat_error: 0.0,
            last_amplitude: 0.0,
            last_instant_rate: 0.0,
            last_aggregate_time: std::time::Instant::now(),
            aggregate_interval_ms: 1000,
            pending_messages: Vec::new(),
        }
    }

    pub fn set_bph(&mut self, bph: u32) {
        self.bph = bph;
        self.nominal_interval = 3600.0 / bph as f64;
    }

    pub fn ingest_ticks(&mut self, ticks: &[TickEvent]) {
        let ticks_per_sec = self.bph as f64 / 3600.0;
        let window_samples = (self.short_window_duration * ticks_per_sec).ceil() as usize;

        for tick in ticks {
            if tick.tick_index <= self.last_sent_tick {
                continue;
            }
            self.last_sent_tick = tick.tick_index;

            let rate = self.compute_rate(tick.interval);
            self.last_instant_rate = rate;
            self.last_amplitude = tick.amplitude;

            self.half_periods.push_back(tick.interval);
            if self.half_periods.len() >= 2 {
                let h1 = self.half_periods[0];
                let h2 = self.half_periods[1];
                self.last_beat_error = (h1 - h2).abs();
                self.half_periods.pop_front();
            }

            self.short_window_rates.push_back(rate);
            while self.short_window_rates.len() > window_samples {
                self.short_window_rates.pop_front();
            }

            let alpha = 1.0 - (-tick.interval / self.long_ewma_tau).exp();
            self.long_ewma = Some(match self.long_ewma {
                Some(prev) => prev + alpha * (rate - prev),
                None => rate,
            });

            self.pending_messages.push(TickEventMessage {
                r#type: "tick".to_string(),
                session_id: self.session_id.clone(),
                tick_index: tick.tick_index,
                timestamp: tick.timestamp,
                interval_s: tick.interval,
                rate_spd: rate,
                amplitude: tick.amplitude,
            });
        }
    }

    pub fn drain_messages(&mut self) -> (Vec<TickEventMessage>, Option<AggregateUpdate>) {
        let messages = self.pending_messages.drain(..).collect();
        let mut aggregate = None;

        let elapsed = self.last_aggregate_time.elapsed();
        if elapsed.as_millis() >= self.aggregate_interval_ms as u128 {
            self.last_aggregate_time = std::time::Instant::now();
            let now = chrono_now_iso();
            let short_avg = if self.short_window_rates.is_empty() {
                self.last_instant_rate
            } else {
                self.short_window_rates.iter().sum::<f64>() / self.short_window_rates.len() as f64
            };

            aggregate = Some(AggregateUpdate {
                r#type: "aggregate".to_string(),
                session_id: self.session_id.clone(),
                time: now,
                instant_rate_spd: self.last_instant_rate,
                short_avg_spd: short_avg,
                long_ewma_spd: self.long_ewma.unwrap_or(self.last_instant_rate),
                beat_error_s: self.last_beat_error,
                amplitude: self.last_amplitude,
            });
        }

        (messages, aggregate)
    }

    pub fn current_rate(&self) -> f64 {
        self.last_instant_rate
    }

    pub fn current_beat_error(&self) -> f64 {
        self.last_beat_error
    }

    pub fn current_amplitude(&self) -> f64 {
        self.last_amplitude
    }

    fn compute_rate(&self, interval_s: f64) -> f64 {
        if self.nominal_interval <= 0.0 || interval_s <= 0.0 {
            return 0.0;
        }
        (interval_s - self.nominal_interval) / self.nominal_interval * 86400.0
    }
}

fn chrono_now_iso() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}