# Name
BoundEffects

# Struct
```rust
#[derive(Component, Default)]
struct BoundEffects(Vec<(String, Tree)>);
```

# Description
Permanent effect tree storage on an entity. Each entry is a (source, tree) pair where source identifies the chip, breaker definition, or other origin that installed the tree.

## How entries are added

- `stamp_effect(entity, source, tree)` appends to this Vec.
- `route_effect(entity, source, tree, RouteType::Bound)` appends to this Vec.
- During initial effect dispatch, `Stamp(target, tree)` resolves to entities and appends here.

## How entries are removed

- By source: when a chip is unequipped or a definition is removed, all entries with that source string are removed from the Vec.

## Pairing with StagedEffects

BoundEffects and StagedEffects are always inserted as a pair. If an entity has one, it has both. Command extensions ensure this by inserting both when either is absent.
