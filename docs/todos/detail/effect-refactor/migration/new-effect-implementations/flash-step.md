# Name
FlashStep

# Enum Variant
- `EffectType::FlashStep(FlashStepConfig)`
- `ReversibleEffectType::FlashStep(FlashStepConfig)`

# Config
`FlashStepConfig {}` (empty struct)

# Fire
1. Insert `FlashStepActive` marker component on the target entity (breaker).
2. If `FlashStepActive` is already present, do nothing.
3. Fire does NOT implement the dash teleport -- that is the breaker movement system's job. It reads `FlashStepActive` to decide whether to teleport.

# Reverse
1. Remove `FlashStepActive` marker from the target entity.
2. If `FlashStepActive` is not present, do nothing.

# Source Location
`src/effect_v3/effects/flash_step/config.rs`

# New Types
- `FlashStepActive` -- marker component indicating the breaker has flash-step enabled. Read by the breaker movement system to decide whether to teleport instead of slide.

# New Systems
None -- breaker movement reads the marker.
