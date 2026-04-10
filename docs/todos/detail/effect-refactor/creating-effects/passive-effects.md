# Passive Effects

A passive effect is an effect that modifies an entity's stats by adding an entry to a stack component. Multiple sources can contribute to the same stat — the component aggregates them.

## The Pattern

Each passive effect has:
- A config struct (e.g. `SpeedBoostConfig`) — implements Fireable + Reversible
- A stack component (e.g. `EffectStack<SpeedBoostConfig>`) — lives on the entity
- An aggregation method on the stack — computes the total effect from all entries

## EffectStack\<T\>

```rust
#[derive(Component, Default)]
struct EffectStack<T: PassiveEffect> {
    entries: Vec<(String, T)>,
}
```

Each entry is a `(source, config)` pair. The source string identifies which chip or definition added this entry so it can be removed on reverse.

`T` must implement the `PassiveEffect` trait, which defines how entries aggregate.

## Fire

When a passive effect fires:
1. Look up or insert `EffectStack<T>` on the entity.
2. Push `(source, config)` onto the entries Vec.

That's it. The stat system reads the stack and recomputes on the next tick.

## Reverse

When a passive effect is reversed:
1. Look up `EffectStack<T>` on the entity.
2. Find and remove the first entry matching `(source, config)`.

If no matching entry is found (already reversed, or entity changed), do nothing.

## PassiveEffect Trait

```rust
trait PassiveEffect: Sized {
    fn aggregate(entries: &[(String, Self)]) -> f32;
}
```

Each config struct implements this to define how its entries combine:
- **Multiplicative** (SpeedBoost, SizeBoost, DamageBoost, BumpForce, QuickStop, Vulnerable): product of all multipliers
- **Additive** (Piercing): sum of all charges
- **Accumulator** (RampingDamage): sum of all increments (but accumulates per-activation, not per-entry — different pattern)

## Reading the Stack

Systems that need the aggregated value call `EffectStack::<T>::aggregate()` or a helper method. For example, the bolt velocity system reads `EffectStack<SpeedBoostConfig>` and multiplies base speed by the aggregate.

## Which Effects Are Passive

| Effect | Aggregation | Stack Component |
|--------|------------|-----------------|
| SpeedBoost | Multiplicative | `EffectStack<SpeedBoostConfig>` |
| SizeBoost | Multiplicative | `EffectStack<SizeBoostConfig>` |
| DamageBoost | Multiplicative | `EffectStack<DamageBoostConfig>` |
| BumpForce | Multiplicative | `EffectStack<BumpForceConfig>` |
| QuickStop | Multiplicative | `EffectStack<QuickStopConfig>` |
| Vulnerable | Multiplicative | `EffectStack<VulnerableConfig>` |
| Piercing | Additive | `EffectStack<PiercingConfig>` |
| RampingDamage | Additive | `EffectStack<RampingDamageConfig>` |

Effects NOT in this table are not passive — they spawn entities, send messages, or toggle markers. They have their own fire/reverse logic.
