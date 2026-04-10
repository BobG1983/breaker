# Name
Anchor

# Enum Variant
- `EffectType::Anchor(AnchorConfig)`
- `ReversibleEffectType::Anchor(AnchorConfig)`

# Config
`AnchorConfig { bump_force_multiplier: OrderedFloat<f32>, perfect_window_multiplier: OrderedFloat<f32>, plant_delay: OrderedFloat<f32> }`

# Fire
1. Insert `AnchorActive` component on the target entity with config values (`bump_force_multiplier`, `perfect_window_multiplier`, `plant_delay`).
2. Fire does NOT plant the breaker immediately -- planting requires standing still for `plant_delay` seconds.
3. Fire does NOT modify bump force.
4. Fire does NOT check if the breaker is moving.

# Reverse
1. Remove `AnchorActive`, `AnchorTimer`, and `AnchorPlanted` from the target entity.
2. If `AnchorPlanted` was present, remove the bump force boost from `EffectStack<BumpForceConfig>`.

# Source Location
`src/effect/configs/anchor.rs`

# New Types
- `AnchorActive` -- component storing anchor config. Fields: `bump_force_multiplier: f32`, `perfect_window_multiplier: f32`, `plant_delay: f32`.
- `AnchorTimer(f32)` -- component tracking time spent stationary before planting.
- `AnchorPlanted` -- marker component indicating the breaker is anchored in place.

# New Systems
- `tick_anchor` -- state machine based on breaker velocity. Runs in `FixedUpdate`.
  1. Moving (nonzero velocity or dashing): remove `AnchorTimer` and `AnchorPlanted`. If was planted, remove bump force boost from `EffectStack<BumpForceConfig>`.
  2. Stationary + no timer + not planted: insert `AnchorTimer(plant_delay)`.
  3. Stationary + timer active: decrement by `dt`. When timer reaches 0: remove timer, insert `AnchorPlanted`, push bump force boost to `EffectStack<BumpForceConfig>`.
  4. Stationary + planted: no-op.
  5. Does NOT modify perfect window directly -- bump timing system reads `AnchorActive.perfect_window_multiplier` when `AnchorPlanted` is present.
