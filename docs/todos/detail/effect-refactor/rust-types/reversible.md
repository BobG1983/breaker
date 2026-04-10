# Name
Reversible

# Syntax
```rust
trait Reversible: Fireable {
    fn reverse(&self, entity: Entity, source: &str, world: &mut World);
}
```

# Description
The contract for undoing an effect on an entity. Config structs in ReversibleEffectType implement both Fireable and Reversible.

- entity: The entity to reverse the effect on.
- source: Must match the source used in the original fire call so the correct instance is identified when multiple sources apply the same effect.
- world: Exclusive world access.

What reverse does depends on the effect type:
- Passive effects (SpeedBoost, DamageBoost, Piercing, etc.): remove the matching (source, config) entry from the entity's EffectStack\<T\>. See [passive-effects](../creating-effects/passive-effects.md) and [effect-stacking/](effect-stacking/index.md).
- Spawned entities (Shield, Pulse, SecondWind): despawn what fire spawned.
- Boolean toggles (FlashStep): remove the marker component.
- Stateful effects (CircuitBreaker, EntropyEngine): reset internal state.

Called by the match dispatch inside ReverseEffectCommand::apply. Each arm unwraps the config and calls `config.reverse(entity, source, world)`.

Reversible extends Fireable — every reversible config can also be fired. This is enforced by the trait bound `Reversible: Fireable`.

DO NOT call reverse for effects that were never fired — the entity state will be wrong.
DO NOT implement Reversible on non-reversible configs (Shockwave, Explode, SpawnBolts, etc.). The type system enforces this via ReversibleEffectType containing only the reversible subset.
