# Name
Reversible

# Syntax
```rust
pub trait Reversible: Fireable {
    fn reverse(&self, entity: Entity, source: &str, world: &mut World);

    /// Reverses all instances of this effect that were applied by `source`.
    /// Default implementation delegates to `reverse()` (reverses one instance).
    /// Override for passive effects that use EffectStack — they should remove
    /// ALL matching (source, config) entries in one call.
    fn reverse_all_by_source(&self, entity: Entity, source: &str, world: &mut World) {
        self.reverse(entity, source, world);
    }
}
```

# Description
The contract for undoing an effect on an entity. Config structs in `ReversibleEffectType` implement both `Fireable` and `Reversible`.

- `entity`: The entity to reverse the effect on.
- `source`: Must match the source used in the original fire call so the correct instance is identified when multiple sources apply the same effect.
- `world`: Exclusive world access.

## `reverse`

Undoes one application of the effect from the given source. What it does depends on the effect type:
- Passive effects (SpeedBoost, DamageBoost, Piercing, etc.): remove the matching (source, config) entry from the entity's `EffectStack<T>`. See [passive-effects](../creating-effects/passive-effects.md) and [effect-stacking/](effect-stacking/index.md).
- Spawned entities (Shield, Pulse, SecondWind): despawn what fire spawned.
- Boolean toggles (FlashStep): remove the marker component.
- Stateful effects (CircuitBreaker, EntropyEngine): reset internal state.

Called by the match dispatch inside `ReverseEffectCommand::apply`. Each arm unwraps the config and calls `config.reverse(entity, source, world)`.

## `reverse_all_by_source`

Reverses ALL instances of this effect that were applied by the given source string. Used when disarming a nested During scope (Shape C/D) — all effects that fired from that scope while it was active must be cleaned up in bulk.

**Default implementation**: delegates to `reverse()`. Appropriate for effects that are single-instance per source (spawned entities, toggles, stateful counters).

**Impls that override** (call `EffectStack::retain_by_source` to remove all entries for source):
- `SpeedBoostConfig`
- `DamageBoostConfig`
- `BumpForceConfig`
- `QuickStopConfig`
- `VulnerableConfig`
- `SizeBoostConfig`
- `PiercingConfig`
- `RampingDamageConfig`
- `AttractionConfig`
- `AnchorConfig`

**Impls that use the default** (single-instance per source, default `reverse()` is sufficient):
- `FlashStepConfig`
- `PulseConfig`
- `ShieldConfig`
- `SecondWindConfig`
- `CircuitBreakerConfig`
- `EntropyConfig`

## Invariants

DO NOT call `reverse` for effects that were never fired — the entity state will be wrong.
DO NOT implement `Reversible` on non-reversible configs (Shockwave, Explode, SpawnBolts, etc.). The type system enforces this via `ReversibleEffectType` containing only the reversible subset.
