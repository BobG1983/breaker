# Name
EntropyEngine

# Enum Variant
- `EffectType::EntropyEngine(EntropyConfig)`
- `ReversibleEffectType::EntropyEngine(EntropyConfig)`

# Config
`EntropyConfig { max_effects: u32, pool: Vec<(OrderedFloat<f32>, Box<EffectType>)> }`

# Fire
This effect is designed to be inside a `When(Killed(Cell), ...)` -- each fire call equals one cell killed.

1. If `EntropyCounter` is present on the target entity: increment `count` (capped at `max_effects`).
2. If `EntropyCounter` is absent: insert with `count = 1`.
3. For `count` times: pick a random effect from `pool` (weighted by the `OrderedFloat<f32>` values) and call its `fire`.
4. Each activation can fire a different random effect.
5. Fire does NOT track which effects were fired -- they are fire-and-forget.

# Reverse
1. Remove `EntropyCounter` from the target entity.
2. Does NOT reverse the individual effects that were fired.

# Source Location
`src/effect/effects/entropy_engine/config.rs`

# New Types
- `EntropyCounter` -- component tracking accumulated activations. Fields: `count: u32`, `max_effects: u32`, `pool: Vec<(OrderedFloat<f32>, Box<EffectType>)>` (clone of config pool).

# New Systems

## No separate reset system
EntropyEngine resets its counter to 0 internally when `fire()` is called — after all effects are fired for the current count, the counter resets. No separate `reset_entropy_counter` system is needed.
