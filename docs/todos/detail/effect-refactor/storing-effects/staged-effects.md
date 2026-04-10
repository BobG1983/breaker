# Name
StagedEffects

# Struct
```rust
#[derive(Component, Default)]
struct StagedEffects(Vec<(String, Tree)>);
```

# Description
One-shot effect tree storage on an entity. Each entry is a (source, tree) pair. Same shape as BoundEffects, different behavior — entries here are consumed after their trigger matches once.

## How entries are added

- `route_effect(entity, source, tree, RouteType::Staged)` appends to this Vec.

## How entries are removed

- By consumption: when an entry's trigger matches, it is removed after evaluation.
- By source: when a chip is unequipped or a definition is removed, all entries with that source string are removed from the Vec.

## Pairing with BoundEffects

StagedEffects and BoundEffects are always inserted as a pair. If an entity has one, it has both. Command extensions ensure this by inserting both when either is absent.
