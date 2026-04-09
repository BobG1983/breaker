# RandomEffect

## Config
```rust
// No separate config struct — pool carried in EffectType enum
// In EffectType: RandomEffect(Vec<(f32, Box<EffectType>)>)
// Pool contains flat EffectType variants, not full effect trees.
// Box needed to avoid infinite enum size.
```
**RON**: `RandomEffect([(0.5, Shockwave(...)), (0.3, Explode(...))])`

## Reversible: NO (random selection makes reversal non-deterministic)

## Target: Any

## Fire
1. Read the weighted pool of (weight, effect_tree) pairs
2. Select one effect randomly based on weights using `GameRng`
3. Fire the selected effect on the entity

## Reverse
No-op — can't deterministically reverse a random selection.

## Notes
- Single random selection from a weighted pool
- Unlike EntropyEngine, fires exactly ONE effect per activation
- Pool weights are relative (normalized internally)
- Pool contains flat EffectType variants, not full effect trees. Box needed to avoid infinite enum size.
