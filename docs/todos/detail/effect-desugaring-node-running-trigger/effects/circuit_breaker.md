# CircuitBreaker

## Config
```rust
struct CircuitBreakerConfig {
    /// Number of bumps required per cycle before reward
    bumps_required: u32,
    /// Number of extra bolts to spawn on reward
    spawn_count: u32,
    /// Whether spawned bolts inherit parent's BoundEffects
    inherit: bool,
    /// Shockwave maximum radius
    shockwave_range: f32,
    /// Shockwave expansion speed
    shockwave_speed: f32,
}
```
**RON**: `CircuitBreaker(bumps_required: 5, spawn_count: 2, inherit: false, shockwave_range: 64.0, shockwave_speed: 500.0)`

## Reversible: YES (removes counter component)

## Target: Bolt or Breaker (entity with bump count tracking)

## Component
```rust
#[derive(Component)]
struct CircuitBreakerCounter {
    remaining: u32,         // bumps left before reward
    bumps_required: u32,    // total per cycle (for reset)
    spawn_count: u32,
    inherit: bool,
    shockwave_range: f32,
    shockwave_speed: f32,
}
```

## Fire (each call = one bump toward countdown)
1. Guard: if entity despawned → return
2. If `CircuitBreakerCounter` exists: decrement `remaining` by 1
   - If remaining reaches 0: fire reward (spawn_bolts + shockwave), reset remaining to bumps_required
3. If absent: insert counter with `remaining = bumps_required - 1`
   - If bumps_required was 1: fire reward immediately and reset

## Reward Fires
- `SpawnBolts::fire(entity, spawn_count, None, inherit, source_chip, world)` — spawn extra bolts
- `Shockwave::fire(entity, shockwave_range, 0.0, 1, shockwave_speed, source_chip, world)` — spawn shockwave

## Reverse
Remove `CircuitBreakerCounter` component from entity.

## Notes
- Charge-and-release pattern: accumulate bumps → burst reward → repeat
- Reward fires inline (calls SpawnBolts and Shockwave fire functions directly)
- Counter persists across cycles — it resets rather than being removed
- No runtime systems needed — all logic in fire/reverse
