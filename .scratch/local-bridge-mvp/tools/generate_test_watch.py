"""
Synthetic mechanical watch tick audio generator.

Generates WAV files simulating mechanical movements with configurable
BPH, rate drift (s/day), and beat error (ms) for testing the DSP pipeline.

Usage:
    python generate_test_watch.py --bph 21600 --drift +12.0 --beat-error 1.2 --duration 10

Based on the Timegrapher DSP Architecture Plan (Requirement B).
"""

import argparse, math, struct, wave
from pathlib import Path


def generate_watch_test_audio(
    filename: str,
    bph: int = 21600,
    drift_s_per_day: float = +12.0,
    beat_error_ms: float = 1.2,
    sample_rate: int = 40000,
    duration_sec: float = 10.0,
):
    total_samples = int(sample_rate * duration_sec)
    audio = [0.0] * total_samples

    beats_per_sec = bph / 3600.0
    nominal_period = 1.0 / beats_per_sec
    time_scale_factor = 1.0 + (drift_s_per_day / 86400.0)
    actual_period = nominal_period * time_scale_factor
    beat_error_sec = beat_error_ms / 1000.0

    current_time = 0.0
    beat_count = 0

    while current_time < duration_sec:
        if beat_count % 2 == 0:
            interval = actual_period + (beat_error_sec / 2.0)
        else:
            interval = actual_period - (beat_error_sec / 2.0)

        sample_idx = int(round(current_time * sample_rate))
        if sample_idx < total_samples:
            tick_dur = int(sample_rate * 0.004)
            for j in range(tick_dur):
                t = j / sample_rate
                envelope = math.exp(-t * 1200)
                sample = math.sin(2 * math.pi * 5000 * t) * envelope * 0.8
                end_idx = sample_idx + j
                if end_idx < total_samples:
                    audio[end_idx] += sample

        current_time += interval
        beat_count += 1

    max_val = max(abs(s) for s in audio)
    if max_val > 0:
        audio = [int(s / max_val * 32000) for s in audio]

    with wave.open(filename, "w") as w:
        w.setnchannels(1)
        w.setsampwidth(2)
        w.setframerate(sample_rate)
        for s in audio:
            w.writeframes(struct.pack("<h", s))

    print(f"Generated: {filename}")
    print(f"  BPH: {bph} | Drift: {drift_s_per_day:+.1f} s/d | Beat error: {beat_error_ms} ms")
    print(f"  Duration: {duration_sec}s | Sample rate: {sample_rate} Hz")
    print(f"  Total beats: {beat_count}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Generate synthetic watch tick audio")
    parser.add_argument("--bph", type=int, default=21600, help="Beats per hour (default: 21600)")
    parser.add_argument("--drift", type=float, default=12.0, help="Rate drift in s/day (default: +12.0)")
    parser.add_argument("--beat-error", type=float, default=1.2, help="Beat error in ms (default: 1.2)")
    parser.add_argument("--duration", type=float, default=10.0, help="Duration in seconds (default: 10)")
    parser.add_argument("--sr", type=int, default=40000, help="Sample rate (default: 40000)")
    parser.add_argument("-o", "--output", default=None, help="Output filename")
    args = parser.parse_args()

    output = args.output or f"test_{args.bph}_drift{args.drift:+.0f}s_be{args.beat_error}ms.wav"
    generate_watch_test_audio(
        filename=output,
        bph=args.bph,
        drift_s_per_day=args.drift,
        beat_error_ms=args.beat_error,
        sample_rate=args.sr,
        duration_sec=args.duration,
    )