# Name
CircuitBreaker

# Enum Variant
- `EffectType::CircuitBreaker(CircuitBreakerConfig)`
- `ReversibleEffectType::CircuitBreaker(CircuitBreakerConfig)`

# Config
`CircuitBreakerConfig { bumps_required: u32, spawn_count: u32, inherit: bool, shockwave_range: OrderedFloat<f32>, shockwave_speed: OrderedFloat<f32> }`

# Fire
This effect is designed to be inside a `When(PerfectBumped, ...)` -- each fire call equals one bump toward the countdown.

1. If `CircuitBreakerCounter` is present on the target entity: decrement `remaining`.
   - If `remaining` reaches 0: fire the reward (`SpawnBoltsConfig.fire` for `spawn_count` bolts + `ShockwaveConfig.fire` for shockwave), reset `remaining` to `bumps_required`.
2. If `CircuitBreakerCounter` is absent: insert with `remaining = bumps_required - 1` (first bump counts).
   - If `bumps_required` is 1: fire reward immediately and reset.
3. Fire does NOT handle bump detection -- it is triggered by the `When(PerfectBumped, ...)` wrapper.
4. Fire does NOT manage spawned bolts or shockwaves -- those are handled by their own systems.

# Reverse
1. Remove `CircuitBreakerCounter` from the target entity.

# Source Location
`src/effect/configs/circuit_breaker.rs`

# New Types
- `CircuitBreakerCounter` -- component tracking progress toward the next reward. Fields: `remaining: u32`, `bumps_required: u32`, `spawn_count: u32`, `inherit: bool`, `shockwave_range: f32`, `shockwave_speed: f32`.

# New Systems
None -- all logic is in fire/reverse.
