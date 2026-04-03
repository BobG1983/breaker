# Slow-Motion Implementation

Uses `Time<Virtual>::set_relative_speed()` — fully supported in Bevy 0.18.

## How It Works

Setting `virt.set_relative_speed(0.3)` means every real second → 0.3 virtual seconds. This directly affects `Time<Fixed>` because `run_fixed_main_schedule` reads `Time<Virtual>.delta()` to accumulate overstep. At 0.3x speed, FixedUpdate runs ~19 steps per real second instead of 64.

Each fixed step is still the same timestep duration (~15.6ms). You get **fewer steps**, not slower steps.

## Smooth Ramp

No built-in ramp. A system in Update interpolates `relative_speed` each frame.

**Critical: use `Time<Real>` for the ramp timer, not `Time<Virtual>`.** Virtual would cause recursive slowdown.

```rust
fn apply_dilation_ramp(
    real: Res<Time<Real>>,
    mut virt: ResMut<Time<Virtual>>,
    mut ramp: ResMut<DilationRamp>,
) {
    if !ramp.active { return; }
    ramp.elapsed += real.delta_secs();
    let t = (ramp.elapsed / ramp.duration).clamp(0.0, 1.0);
    let t_smooth = t * t * (3.0 - 2.0 * t);  // smoothstep
    let speed = ramp.start + (ramp.target - ramp.start) * t_smooth;
    virt.set_relative_speed(speed);
    if t >= 1.0 { ramp.active = false; }
}
```

## Gotchas

- `set_relative_speed(0.0)` is NOT `pause()` — `is_paused()` returns false
- `effective_speed()` returns 0.0 when paused; `relative_speed()` returns configured ratio even when paused
- Audio is NOT automatically slowed — pitch/playback rate requires separate handling
- One-frame latency on ramp writes — imperceptible at 60fps
- `max_delta` cap (default 250ms) applies before speed scaling
