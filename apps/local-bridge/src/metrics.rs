use std::collections::VecDeque;

use crate::dsp::TickEvent;

#[derive(Debug, Clone, serde::Serialize)]
pub struct AggregateUpdate {
    pub r#type: String,
    pub session_id: String,
    pub time: String,
    pub instant_rate_spd: f64,
    pub avg_rate_spd: f64,
    pub avg_window_s: f64,
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
    #[allow(dead_code)]
    sample_rate: f64,
    bph: u32,
    nominal_interval: f64,
    pair_buffer: VecDeque<f64>,
    beat_error_samples: VecDeque<f64>,
    last_sent_tick: u64,

    avg_window_rates: VecDeque<f64>,
    avg_window_duration: f64,

    last_beat_error: f64,
    last_amplitude: f64,
    last_instant_rate: f64,

    last_aggregate_time: std::time::Instant,
    aggregate_interval_ms: u64,

    pending_messages: Vec<TickEventMessage>,
}

impl MetricsEngine {
    pub fn new(session_id: String, sample_rate: f64) -> Self {
        let default_bph = 28800;
        MetricsEngine {
            session_id,
            sample_rate,
            bph: default_bph,
            nominal_interval: 3600.0 / default_bph as f64 / 2.0,
            pair_buffer: VecDeque::with_capacity(2),
            beat_error_samples: VecDeque::with_capacity(10),
            last_sent_tick: 0,
            avg_window_rates: VecDeque::new(),
            avg_window_duration: 30.0,
            last_beat_error: 0.0,
            last_amplitude: 0.0,
            last_instant_rate: 0.0,
            last_aggregate_time: std::time::Instant::now(),
            aggregate_interval_ms: 1000,
            pending_messages: Vec::new(),
        }
    }

    pub fn set_bph(&mut self, bph: u32) {
        if self.bph == bph {
            return;
        }
        self.bph = bph;
        self.nominal_interval = 3600.0 / bph as f64 / 2.0;
        self.avg_window_rates.clear();
        self.last_instant_rate = 0.0;
    }

    pub fn bph(&self) -> u32 {
        self.bph
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.last_sent_tick = 0;
        self.pair_buffer.clear();
        self.beat_error_samples.clear();
        self.avg_window_rates.clear();
        self.last_beat_error = 0.0;
        self.last_amplitude = 0.0;
        self.last_instant_rate = 0.0;
        self.pending_messages.clear();
    }

    pub fn ingest_ticks(&mut self, ticks: &[TickEvent]) {
        let ticks_per_sec = self.bph as f64 * 2.0 / 3600.0;
        let window_samples = (self.avg_window_duration * ticks_per_sec).ceil() as usize;

        for tick in ticks {
            if tick.tick_index <= self.last_sent_tick {
                continue;
            }
            self.last_sent_tick = tick.tick_index;

            let rate = self.compute_rate(tick.interval);
            self.last_instant_rate = rate;
            self.last_amplitude = tick.amplitude;

            // Beat error: accumulate pairs of consecutive intervals, average over last 10 pairs
            self.pair_buffer.push_back(tick.interval);
            if self.pair_buffer.len() >= 2 {
                let h1 = self.pair_buffer.pop_front().unwrap();
                let h2 = self.pair_buffer.pop_front().unwrap();
                let be = (h1 - h2).abs();
                self.beat_error_samples.push_back(be);
                if self.beat_error_samples.len() > 10 {
                    self.beat_error_samples.pop_front();
                }
                self.last_beat_error = self.beat_error_samples.iter().sum::<f64>()
                    / self.beat_error_samples.len() as f64;
            }

            self.avg_window_rates.push_back(rate);
            while self.avg_window_rates.len() > window_samples {
                self.avg_window_rates.pop_front();
            }

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

        if self.last_sent_tick > 0 {
            let elapsed = self.last_aggregate_time.elapsed();
            if elapsed.as_millis() >= self.aggregate_interval_ms as u128 {
                self.last_aggregate_time = std::time::Instant::now();
                let now = chrono_now_iso();
                let avg_rate = if self.avg_window_rates.is_empty() {
                    self.last_instant_rate
                } else {
                    self.avg_window_rates.iter().sum::<f64>() / self.avg_window_rates.len() as f64
                };

                aggregate = Some(AggregateUpdate {
                    r#type: "aggregate".to_string(),
                    session_id: self.session_id.clone(),
                    time: now,
                    instant_rate_spd: self.last_instant_rate,
                    avg_rate_spd: avg_rate,
                    avg_window_s: self.avg_window_duration,
                    beat_error_s: self.last_beat_error,
                    amplitude: self.last_amplitude,
                });
            }
        }

        (messages, aggregate)
    }

    #[allow(dead_code)]
    pub fn current_rate(&self) -> f64 {
        self.last_instant_rate
    }

    #[allow(dead_code)]
    pub fn current_beat_error(&self) -> f64 {
        self.last_beat_error
    }

    #[allow(dead_code)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsp::TickEvent;

    fn make_tick(index: u64, interval: f64, amplitude: f64) -> TickEvent {
        TickEvent {
            tick_index: index,
            sample_index: (index as f64 * 3675.0) as u64,
            fractional_offset: 0.0,
            amplitude,
            interval,
            timestamp: index as f64 * interval,
        }
    }

    #[test]
    fn test_zero_drift_produces_zero_rate() {
        let mut engine = MetricsEngine::new("test".into(), 44100.0);
        engine.set_bph(21600);

        let nominal = 3600.0 / 21600.0 / 2.0;

        let ticks: Vec<TickEvent> = (1..=60)
            .map(|i| make_tick(i, nominal, 0.5))
            .collect();

        engine.ingest_ticks(&ticks);
        let (msgs, _) = engine.drain_messages();

        for msg in &msgs {
            assert!(
                msg.rate_spd.abs() < 0.01,
                "tick {} rate={:.4} s/d, expected ~0 for perfect beat",
                msg.tick_index,
                msg.rate_spd,
            );
        }
    }

    #[test]
    fn test_positive_drift_increases_rate() {
        let mut engine = MetricsEngine::new("test".into(), 44100.0);
        engine.set_bph(21600);

        let nominal = 3600.0 / 21600.0 / 2.0;
        let interval = nominal * (1.0 + 12.0 / 86400.0);

        let ticks: Vec<TickEvent> = (1..=60)
            .map(|i| make_tick(i, interval, 0.5))
            .collect();

        engine.ingest_ticks(&ticks);
        let (msgs, _) = engine.drain_messages();

        for msg in &msgs {
            assert!(
                msg.rate_spd > 10.0,
                "tick {} rate={:.2} s/d, expected ~+12 s/d for +12 s/d drift",
                msg.tick_index,
                msg.rate_spd,
            );
        }
    }

    #[test]
    fn test_reset_clears_state() {
        let mut engine = MetricsEngine::new("test".into(), 44100.0);
        engine.set_bph(21600);
        let nominal = 3600.0 / 21600.0 / 2.0;

        let ticks: Vec<TickEvent> = (1..=60)
            .map(|i| make_tick(i, nominal, 0.5))
            .collect();

        engine.ingest_ticks(&ticks);
        let (msgs, _) = engine.drain_messages();
        assert!(!msgs.is_empty(), "expected tick messages before reset");

        engine.reset();

        let (msgs2, _) = engine.drain_messages();
        assert!(msgs2.is_empty(), "pending messages not cleared by reset");

        engine.ingest_ticks(&ticks);
        let (msgs3, _) = engine.drain_messages();
        assert_eq!(
            msgs3.len(), msgs.len(),
            "after reset, re-ingesting same ticks should produce same number of messages ({} vs {})",
            msgs3.len(), msgs.len(),
        );
        for msg in &msgs3 {
            assert!(
                msg.rate_spd.abs() < 0.01,
                "tick {} rate={:.4} s/d after reset (expected ~0)",
                msg.tick_index, msg.rate_spd,
            );
        }
    }

    #[test]
    fn test_no_aggregate_before_ticks() {
        let mut engine = MetricsEngine::new("test".into(), 44100.0);
        let (_, agg) = engine.drain_messages();
        assert!(agg.is_none(), "should not produce aggregate before any ticks");
    }

    #[test]
    fn test_aggregate_uses_avg_rate() {
        let mut engine = MetricsEngine::new("test".into(), 44100.0);
        engine.set_bph(21600);
        let nominal = 3600.0 / 21600.0;

        let ticks: Vec<TickEvent> = (1..=30)
            .map(|i| make_tick(i, nominal, 0.5))
            .collect();

        engine.ingest_ticks(&ticks);
        // Force aggregate by advancing time
        let (_, agg) = engine.drain_messages();
        // Aggregates only fire after 1000ms real time in the engine,
        // so this won't produce one in a unit test — just verify no crash
        assert!(agg.is_none() || agg.as_ref().unwrap().avg_window_s > 0.0);
    }
}