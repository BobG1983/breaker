# Name
Pulse

# Enum Variant
- `EffectType::Pulse(PulseConfig)`
- `ReversibleEffectType::Pulse(PulseConfig)`

# Config
`PulseConfig { base_range: OrderedFloat<f32>, range_per_level: OrderedFloat<f32>, stacks: u32, speed: OrderedFloat<f32>, interval: OrderedFloat<f32> }`

# Fire
1. Insert `PulseEmitter` component on the target entity with config values (`base_range`, `range_per_level`, `stacks`, `speed`, `interval`).
2. Initialize the internal timer to `interval`.
3. Fire does NOT emit shockwaves -- `tick_pulse` does.

# Reverse
1. Remove `PulseEmitter` from the target entity.
2. Active shockwaves already spawned continue to completion -- they are NOT despawned.

# Source Location
`src/effect/configs/pulse.rs`

# New Types
- `PulseEmitter` -- component storing pulse config and timer state. Fields: `base_range: f32`, `range_per_level: f32`, `stacks: u32`, `speed: f32`, `interval: f32`, `timer: f32`.

# New Systems

## tick_pulse
- **What it does**: For each entity with `PulseEmitter`, decrement timer by `dt`. When timer reaches 0 or below: spawn a shockwave at the entity's position using the pulse's config values (base_range, range_per_level, stacks, speed) and the entity's current damage snapshot. Reset timer to `interval`.
- **What it does NOT do**: Does not manage spawned shockwaves — shockwave systems handle their lifecycle. Does not modify PulseEmitter config values.
- **Schedule**: FixedUpdate.
