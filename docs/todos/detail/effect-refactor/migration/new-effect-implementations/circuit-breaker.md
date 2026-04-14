# Name
CircuitBreaker

# Enum Variant
- `EffectType::CircuitBreaker(CircuitBreakerConfig)`
- `ReversibleEffectType::CircuitBreaker(CircuitBreakerConfig)`

# Config
`CircuitBreakerConfig { bumps_required: u32, spawn_count: u32, inherit: bool, shockwave_range: OrderedFloat<f32>, shockwave_speed: OrderedFloat<f32> }`

# Fire
1. Insert `CircuitBreakerCounter` on the target entity with `remaining` set to `bumps_required` and all config values copied.
2. If a `CircuitBreakerCounter` already exists, fire() overwrites it — resetting progress to full `bumps_required`.
3. Fire does NOT handle bump counting or reward firing — `tick_circuit_breaker` does.

# Reverse
1. Remove `CircuitBreakerCounter` from the target entity.

# Source Location
`src/effect_v3/effects/circuit_breaker/config.rs`

# New Types
- `CircuitBreakerCounter` -- component tracking progress toward the next reward. Fields: `remaining: u32`, `bumps_required: u32`, `spawn_count: u32`, `inherit: bool`, `shockwave_range: f32`, `shockwave_speed: f32`, `source_chip: Option<EffectSourceChip>`.

# New Systems

## tick_circuit_breaker
- **What it does**: Reads `BumpPerformed` messages. For each entity with `CircuitBreakerCounter`, decrements `remaining` on each bump. When `remaining` reaches 0: fires the reward (`SpawnBoltsConfig.fire` for `spawn_count` bolts + `ShockwaveConfig.fire` for shockwave), then resets `remaining` to `bumps_required`.
- **What it does NOT do**: Does not detect bump type or timing — reads any `BumpPerformed` message.
- **Schedule**: FixedUpdate, in `EffectV3Systems::Tick`.
