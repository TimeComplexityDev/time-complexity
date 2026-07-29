# DSP Tradeoffs: Matched Filter vs Envelope Detection

## What we're detecting

A mechanical watch tick is a short mechanical impulse — the escapement hitting the pallet fork. It's not a pure sine wave. It has a broadband "click" shape: a fast attack, a brief sustain, and a decay. We want to find every tick, timestamp it precisely, and ignore everything else (room noise, table bumps, speech).

## The signal problem

The raw microphone signal is messy:
- Low-frequency rumble (HVAC, footsteps)
- Mid-frequency noise (room tone, clothing rustle)
- High-frequency hiss (electronics, digital noise)
- The tick itself is often buried near the noise floor, especially through a contact mic that also picks up case vibration

We need a way to amplify "tick-like" things and suppress "not-tick-like" things.

## Envelope detection (simpler path)

### How it works

1. **Bandpass filter** — keep only the frequency range where ticks live (e.g., 800–6000 Hz). Kill rumble and hiss.
2. **Hilbert transform** — compute the analytic signal, which gives us an instantaneous amplitude envelope.
3. **Lowpass the envelope** — smooth out the fast oscillations, leaving a slow "loudness" curve.
4. **Peak detect** — find where this envelope crosses a threshold, respecting a refractory period (don't detect again for `0.25 × nominal_half_period`).

### Pros

- **No calibration required.** Works out of the box.
- **Deterministic latency** — every tick goes through the same fixed filter chain.
- **Easy to tune** — just adjust threshold and refractory.
- **Low CPU** — biquads + envelope + peak detector is lightweight.

### Cons

- **Noisy in bad acoustics.** If there are other transient sounds in the 800–6000 Hz band (clicks, taps, keyboard), they'll produce envelope peaks that look like ticks.
- **Threshold is fragile.** Too low → false positives. Too high → missed ticks on low-amplitude watches.
- **No "tick shape" memory.** Every peak is treated the same; it can't distinguish a watch tick from a desk tap by shape.

### When it struggles

- Multiple watches in the room (different tick pitches overlap)
- Contact mic picking up case vibration that looks like a double-tick
- Very quiet watches where the tick SNR is low

## Matched filter (smarter path)

### How it works

1. **Capture a template** — the user taps the watch once (or the system auto-detects a clean isolated tick) and a few milliseconds of audio around it are saved as the "template."
2. **Cross-correlate** — slide this template over the incoming audio, sample by sample. At each position, compute how well the incoming signal matches the template.
3. **Peak detect on the correlation output** — high correlation = "this looks like a tick."

Mathematically: if `h` is the template and `x` is the incoming signal, the matched filter output at sample `n` is:

```
y[n] = Σ h[k] · x[n + k]
```

This is convolution with a time-reversed template — the optimal linear filter for detecting a known signal in noise.

### Pros

- **Exploits the tick's actual shape.** A desk tap has a different frequency contour and decay than a watch tick; the matched filter naturally suppresses it.
- **Better SNR.** Mathematically optimal for additive white noise; real-world improvement is significant — often 3–6 dB gain over plain envelope.
- **Adaptable.** Capture a template from a specific watch and that watch's ticks stand out more.

### Cons

- **Requires calibration.** Someone has to provide a clean tick sample. This is an extra UI step.
- **Templates degrade.** If the user changes watches, mic position, or BPH, the old template is suboptimal.
- **More CPU.** Convolution over a multi-millisecond template at 96 kHz is non-trivial (though still fine on modern hardware).
- **Template drift.** A worn movement or different lift angle changes the tick sound; the template becomes stale.

### When it struggles

- User skips calibration (uses a bad template)
- Multiple simultaneous tick sources (different templates needed)
- Watch changes mid-session without re-calibration

## The compromise: FFT-based fast convolution

At 96 kHz with a 6 ms template, you need ~576 samples of convolution per window. You can do this in the time domain (which gets expensive for long templates) or use overlap-add/save FFT convolution to batch it into chunks. Rust crates like `rustfft` handle this well. The cost is still well within budget for a single-client server.

## Practical recommendation for MVP

**Start with envelope detection** for MVP. Here's why:

1. It works immediately — no calibration UI, no "please tap your watch" flow. You can capture ticks on day one.
2. It teaches you the rest of the pipeline: audio capture → bandpass → peak detection → beat pairing → metrics. Every subsequent algorithm sits on top of this.
3. The refractory period + bandpass gives you ~80% of the benefit for ~20% of the complexity.
4. Once you have a working pipeline, adding matched filter is a drop-in replacement for step 3 of the DSP chain. The API contracts (tick events, aggregate updates) don't change.

**Then add matched filter as an optional mode** once you've validated the end-to-end flow with real watches. You'll have ground-truth tick data from the envelope mode to validate the matched filter against.
