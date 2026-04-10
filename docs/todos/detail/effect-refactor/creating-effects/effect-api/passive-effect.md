# Passive Effect

Passive effects additionally implement PassiveEffect. See [rust-types/passive-effect.md](../../rust-types/effect-stacking/passive-effect.md) for the type definition.

```rust
trait PassiveEffect: Fireable + Reversible + Sized + Clone + PartialEq + Eq {
    fn aggregate(entries: &[(String, Self)]) -> f32;
}
```

PassiveEffect is a supertrait of Fireable + Reversible. Passive configs implement all three traits explicitly — no blanket impl.

## The pattern

Every passive effect follows the same fire/reverse pattern:

**fire**: Get or insert `EffectStack<Self>` on the entity. Call `stack.push(source, self.clone())`.

**reverse**: Get `EffectStack<Self>` on the entity. Call `stack.remove(source, self)`. Matches by (source, config) pair — exact match is possible because all configs derive Eq via `OrderedFloat<f32>`.

## aggregate

The only method that varies between passive effects. Defines how multiple entries combine into a single value.

| Aggregation | Identity (empty stack) | Examples |
|-------------|----------------------|----------|
| Multiplicative — product of all values | 1.0 | SpeedBoost, SizeBoost, DamageBoost, BumpForce, QuickStop, Vulnerable |
| Additive — sum of all values | 0 | Piercing, RampingDamage |

Systems that need the aggregated value read the EffectStack component and call `stack.aggregate()`.
