# Name
EffectStack\<T\>

# Syntax
```rust
#[derive(Component, Default)]
struct EffectStack<T: PassiveEffect> {
    entries: Vec<(String, T)>,
}

impl<T: PassiveEffect> EffectStack<T> {
    fn push(&mut self, source: String, config: T);
    fn remove(&mut self, source: &str, config: &T);
    fn aggregate(&self) -> f32;
}
```

# Description
Generic stack component for passive effects. Each entry is a (source, config) pair. The source string identifies which chip or definition added the entry.

Monomorphized per config type — `EffectStack<SpeedBoostConfig>` and `EffectStack<DamageBoostConfig>` are independent Bevy components.

## push

Append a (source, config) entry to the stack. Called by fire implementations.

## remove

Find and remove the first entry matching (source, config) exactly. Called by reverse implementations. If no match is found, do nothing. Exact match is possible because config structs derive Eq via `OrderedFloat<f32>`.

## aggregate

Delegate to `T::aggregate(&self.entries)` to compute the combined value. Returns the identity value (1.0 for multiplicative, 0 for additive) when the stack is empty.

Systems that need the current value call this — e.g. the bolt velocity system calls `stack.aggregate()` to get the total speed multiplier.

## Constraints

DO insert EffectStack with default (empty) when the entity is created, so it's always present for query access.
DO NOT store non-passive effect configs in EffectStack. Only configs that implement PassiveEffect belong here.
