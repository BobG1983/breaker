# Pulse

## Config
```rust
struct PulseConfig {
    /// Base radius per shockwave emission
    base_range: f32,
    /// Extra radius per stack
    range_per_level: f32,
    /// Current stack count
    stacks: u32,
    /// Expansion speed
    speed: f32,
    /// Seconds between ring emissions (default 0.5)
    interval: f32,
}
```
**RON**: `Pulse(base_range: 32.0, range_per_level: 4.0, stacks: 1, speed: 300.0, interval: 0.5)`

## Reversible: YES (removes emitter component)

## Target: Bolt (attaches periodic emitter)

## Component
```rust
#[derive(Component)]
struct PulseEmitter {
    base_range: f32,
    range_per_level: f32,
    stacks: u32,
    speed: f32,
    interval: f32,
    timer: f32,  // countdown to next emission, starts at 0
}
```

## Fire
1. Insert `PulseEmitter` component on entity with timer at 0 (fires immediately on first tick)

## Reverse
1. Remove `PulseEmitter` component from entity

## Runtime System: `tick_pulse`
**Schedule**: FixedUpdate, run_if NodeState::Playing

1. For each entity with `PulseEmitter`:
   - Tick timer down by dt
   - When timer <= 0: spawn shockwave at entity's position using the pulse's config, reset timer to interval

## Notes
- Periodic shockwave emitter — fires a shockwave every `interval` seconds
- Each emitted shockwave is an independent entity (same as Shockwave effect)
- Timer starts at 0 → first emission happens immediately
- Stacking: range increases with stacks (same formula as Shockwave)
