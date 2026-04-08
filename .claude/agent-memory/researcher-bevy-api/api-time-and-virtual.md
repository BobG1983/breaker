---
name: Bevy 0.18.1 Time API
description: Time<Virtual>/Real/Fixed, time dilation, pause/unpause, ramp systems, plain Res<Time> context
type: reference
---

# Time API (Bevy 0.18.1)

Verified against docs.rs/bevy/0.18.1 and official source.

## `Time<Virtual>` — controlling game speed

```rust
// Access:
fn my_system(mut virtual_time: ResMut<Time<Virtual>>) { ... }

// Set speed (panics if negative or non-finite):
virtual_time.set_relative_speed(0.3_f32);   // 30% speed slow-motion
virtual_time.set_relative_speed_f64(0.3_f64);

// Read current speed:
let speed: f32 = virtual_time.relative_speed();
let effective: f32 = virtual_time.effective_speed(); // 0.0 when paused

// Pause / unpause:
virtual_time.pause();
virtual_time.unpause();
let paused: bool = virtual_time.is_paused();
```

## `Time<Virtual>` affects `Time<Fixed>` / FixedUpdate

Confirmed from source: `run_fixed_main_schedule` reads `Time<Virtual>.delta()` and
accumulates it into `Time<Fixed>`. Setting `set_relative_speed(0.3)` means FixedUpdate
runs at ~30% of normal frequency. Each fixed step still has the same `timestep()` duration
(default 64 Hz ≈ 15.6ms) — you get fewer steps, not slower steps.

## `Time<Real>` — always wall-clock, unaffected by speed/pause

```rust
fn my_system(real_time: Res<Time<Real>>) {
    let wall_delta: f32 = real_time.delta_secs();
    let wall_elapsed: f32 = real_time.elapsed_secs();
}
```

Always use `Time<Real>` for: ramp timers, UI animations, audio timing, anything that
must not be affected by game speed changes.

## Plain `Res<Time>` — context-dependent alias

- In `Update`: behaves like `Time<Virtual>`
- In `FixedUpdate`: behaves like `Time<Fixed>`

## Smooth ramp-in/ramp-out for time dilation

No built-in ramp. Implement with a system in `Update` that reads `Time<Real>` (not `Time<Virtual>`!)
and calls `set_relative_speed()` each frame. Using `Time<Virtual>` for the ramp timer creates a
recursive slow-down bug where the ramp itself slows as speed decreases.

```rust
// Use Time<Real> for the ramp timer:
fn ramp_system(real: Res<Time<Real>>, mut virt: ResMut<Time<Virtual>>, mut state: ResMut<Ramp>) {
    state.elapsed += real.delta_secs();  // real time, not virtual!
    let t = (state.elapsed / state.duration).clamp(0.0, 1.0);
    let t_smooth = t * t * (3.0 - 2.0 * t); // smooth-step
    virt.set_relative_speed(state.start + (state.target - state.start) * t_smooth);
}
```
