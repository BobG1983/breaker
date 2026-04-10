# Name
Die

# Enum Variant
- `EffectType::Die(DieConfig)`
- NOT in `ReversibleEffectType`

# Config
`DieConfig {}` (empty struct)

# Fire
1. Determine the entity's `GameEntity` type by inspecting components (`Bolt`, `Cell`, `Wall`, `Breaker`).
2. Send the appropriate `KillYourself<T>` message with `victim = entity`, `killer` from `TriggerContext` (the entity that caused this trigger chain, if any).
3. If the entity has no `GameEntity` component, do nothing.
4. Fire does NOT despawn the entity.
5. Fire does NOT check invulnerability.
6. Fire does NOT fire death triggers.
7. All of that is the death pipeline's job (`KillYourself` -> domain handler -> `Destroyed` -> `on_destroyed` -> triggers -> `DespawnEntity`).

# Reverse
Not applicable -- Die is not reversible.

# Source Location
`src/effect/effects/die/config.rs`

# New Types
None -- uses `KillYourself<T>` from the unified death pipeline.

# New Systems
None.
