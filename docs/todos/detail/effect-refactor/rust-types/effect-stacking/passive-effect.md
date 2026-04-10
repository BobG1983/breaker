# Name
PassiveEffect

# Syntax
```rust
trait PassiveEffect: Fireable + Reversible + Sized + Clone + PartialEq + Eq {
    fn aggregate(entries: &[(String, Self)]) -> f32;
}
```

# Description
Trait bound for types that can live in an EffectStack. Extends Fireable + Reversible — every passive effect must implement fire and reverse explicitly.

- aggregate: Takes the full list of (source, config) entries and returns a single f32. Multiplicative effects return the product. Additive effects return the sum. Empty stack returns the identity value (1.0 for multiplicative, 0 for additive).

Requires Clone (fire clones config into the stack) and PartialEq + Eq (remove matches by (source, config) pair). Eq is possible because all config structs use `OrderedFloat<f32>` instead of raw f32.

Every passive config struct implements fire and reverse with the same pattern:
- fire: get or insert `EffectStack<Self>` on the entity, call `stack.push(source, self.clone())`
- reverse: get `EffectStack<Self>` on the entity, call `stack.remove(source, self)` — matches by (source, config) pair, removes the first exact match

Implemented by: SpeedBoostConfig, SizeBoostConfig, DamageBoostConfig, BumpForceConfig, QuickStopConfig, VulnerableConfig, PiercingConfig, RampingDamageConfig.

DO NOT implement PassiveEffect on config structs for non-passive effects (ShockwaveConfig, ExplodeConfig, etc.). Those effects don't stack — they fire once and are done. They implement Fireable directly.
