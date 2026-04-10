# Name
Piercing

# Enum Variant
- `EffectType::Piercing(PiercingConfig)`
- `ReversibleEffectType::Piercing(PiercingConfig)`

# Config
`PiercingConfig { charges: u32 }`

# Fire
1. Get or insert `EffectStack<PiercingConfig>` on the target entity.
2. Push `(source, config)` onto the stack.
3. Calculate the new aggregate (sum of all charges on the stack).
4. If `PiercingRemaining` is not present on the entity, insert `PiercingRemaining` initialized to the new aggregate.
5. Fire does NOT handle collision piercing logic -- that is bolt_cell_collision's job.

Note: aggregation is additive (sum of charges), not multiplicative.

# Reverse
1. Get `EffectStack<PiercingConfig>` on the target entity.
2. Remove the first entry matching `(source, config)`.
3. Recalculate the aggregate (sum of remaining charges on the stack).
4. If the aggregate is 0, remove `PiercingRemaining` from the entity.
5. Reverse does NOT handle collision piercing logic -- that is bolt_cell_collision's job.

# Source Location
`src/effect/effects/piercing/config.rs`

# New Types
- `PiercingRemaining(u32)` -- component that tracks remaining piercing charges consumed during gameplay. Reset at node start.

# New Systems
None -- `PiercingRemaining` is managed by collision and node reset systems, not by the effect itself.
