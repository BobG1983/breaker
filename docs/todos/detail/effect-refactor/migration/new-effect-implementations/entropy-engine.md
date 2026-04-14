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
`src/effect_v3/effects/entropy_engine/config.rs`

# New Types
- `EntropyCounter` -- component tracking accumulated activations. Fields: `count: u32`, `max_effects: u32`, `pool: Vec<(OrderedFloat<f32>, Box<EffectType>)>` (clone of config pool).

# New Systems

## reset_entropy_counter
- **What it does**: For each entity with `EntropyCounter`, set `count` to 0.
- **What it does NOT do**: Does not remove the component. Does not modify the pool or max_effects.
- **Schedule**: `OnEnter(NodeState::Loading)`, registered in `EffectV3Systems::Reset`.

## tick_entropy_engine
- **What it does**: For each `BumpPerformed` message this frame, increments all `EntropyCounter` counts (capped at `max_effects`) and fires N random effects from the pool where N = current count.
- **What it does NOT do**: Does not reset the counter. Does not fire effects directly — delegates to `fire_dispatch`.
- **Schedule**: FixedUpdate, in `EffectV3Systems::Tick`.
