# Name
LoseLife

# Enum Variant
- `EffectType::LoseLife(LoseLifeConfig)`
- NOT in `ReversibleEffectType`

# Config
`LoseLifeConfig {}` (empty struct)

# Fire
1. Send `DamageDealt<Breaker>` with `dealer: None`, `target: entity`, `amount: 1.0`, `source_chip: Some(source)`.
2. The `apply_damage` system handles decrementing Hp. If Hp reaches 0, the death pipeline handles the rest.
3. Fire does NOT decrement Hp directly.
4. Fire does NOT check if lives are zero.
5. Fire does NOT trigger game over.

# Reverse
Not applicable -- LoseLife is not reversible.

# Source Location
`src/effect_v3/effects/lose_life/config.rs`

# New Types
None -- uses `DamageDealt<Breaker>` from the unified death pipeline.

# New Systems
None -- `apply_damage<Breaker>` handles it.
