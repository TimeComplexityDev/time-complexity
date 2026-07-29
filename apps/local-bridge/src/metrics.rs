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

    // Per-tick rate for short window averaging
    short_window_rates: VecDeque<f64>,
    short_window_duration: f64,

    // Long-term EWMA
    long_ewma: Option<f64>,
    long_ewma_tau: f64,

    // Last known values for aggregate
    last_beat_error: f64,
    last_amplitude: f64,
    last_instant_rate: f64,

    last_aggregate_time: std::time::Instant,
    aggregate_interval_ms: u64,
}

impl MetricsEngine {
    pub fn new(session_id: String, sample_rate: f64, bph: u32) -> Self {
        let nominal_interval = 3600.0 / bph as f64;
        MetricsEngine {
            session_id,
            sample_rate,
            bph,
            nominal_interval,
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
        }
    }

    pub fn set_bph(&mut self, bph: u32) {
        self.bph = bph;
        self.nominal_interval = 3600.0 / bph as f64;
    }

    /// Process new ticks from the DSP pipeline.
    /// Returns messages to send over WebSocket.
    pub fn process_ticks(
        &mut self,
        ticks: &[TickEvent],
    ) -> (Vec<TickEventMessage>, Option<AggregateUpdate>) {
        let mut messages = Vec::new();
        let mut aggregate = None;

        for tick in ticks {
            if tick.tick_index <= self.last_sent_tick {
                continue;
            }
            self.last_sent_tick = tick.tick_index;

            let rate = self.compute_rate(tick.interval);
            self.last_instant_rate = rate;
            self.last_amplitude = tick.amplitude;

            // Accumulate half-periods for beat error
            self.half_periods.push_back(tick.interval);
            if self.half_periods.len() >= 2 {
                let h1 = self.half_periods[0];
                let h2 = self.half_periods[1];
                self.last_beat_error = (h1 - h2).abs();
                self.half_periods.pop_front(); // slide: keep h2, wait for next
            }

            // Short window: maintain rolling window of rates
            self.short_window_rates.push_back(rate);
            let window_samples = (self.short_window_duration * 6.0) as usize; // ~6 ticks/sec at 21600 BPH
            while self.short_window_rates.len() > window_samples {
                self.short_window_rates.pop_front();
            }

            // Long EWMA
            let alpha = 1.0 - (-tick.interval / self.long_ewma_tau).exp();
            self.long_ewma = Some(match self.long_ewma {
                Some(prev) => prev + alpha * (rate - prev),
                None => rate,
            });

            messages.push(TickEventMessage {
                r#type: "tick".to_string(),
                session_id: self.session_id.clone(),
                tick_index: tick.tick_index,
                timestamp: tick.timestamp,
                interval_s: tick.interval,
                rate_spd: rate,
                amplitude: tick.amplitude,
            });
        }

        // Check if it's time for an aggregate update
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

    fn compute_rate(&self, interval_s: f64) -> f64 {
        if self.nominal_interval <= 0.0 || interval_s <= 0.0 {
            return 0.0;
        }
        (interval_s - self.nominal_interval) / self.nominal_interval * 86400.0
    }
}

fn chrono_now_iso() -> String {
    // Simple ISO-8601 without pulling in chrono. Uses system clock via std.
    let dur = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs();
    // Format as ISO-8601: 2026-07-29T22:07:31Z
    // Compute date/time from secs since epoch (simple approach)
    let days = secs / 86400;
    let time_secs = secs % 86400;
    let hours = time_secs / 3600;
    let minutes = (time_secs % 3600) / 60;
    let seconds = time_secs % 60;

    // Convert days since epoch to date (only valid for dates after 1970)
    // Using a simplified algorithm for display purposes
    let mut y = 1970i64;
    let mut remaining_days = days as i64;

    loop {
        let days_in_year = if is_leap(y) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        y += 1;
    }

    let month_days = if is_leap(y) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut m = 0usize;
    let mut d = remaining_days;
    loop {
        if d < month_days[m] {
            break;
        }
        d -= month_days[m];
        m += 1;
    }

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        y,
        m + 1,
        d + 1,
        hours,
        minutes,
        seconds,
    )
}

fn is_leap(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
}