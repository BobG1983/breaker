# Name
BoundEffects

# Struct
```rust
#[derive(Component, Default)]
struct BoundEffects(Vec<BoundEntry>);

struct BoundEntry {
    source: String,
    tree: Tree,
    condition_active: Option<bool>,
}
```

# Description
Permanent effect tree storage on an entity. Each entry is a BoundEntry with a source string, a tree, and optional condition tracking state.

- `source`: identifies which chip, breaker definition, or other origin installed the tree.
- `tree`: the effect tree.
- `condition_active`: runtime state for During entries only. Tracks whether the condition was active last frame. `None` for non-During entries. Set to `Some(false)` when a During entry is first installed. The `evaluate_conditions` system reads and updates this field to detect transitions.

## How entries are added

- `stamp_effect(entity, source, tree)` appends a BoundEntry with `condition_active: None` (or `Some(false)` if tree root is During).
- `route_effect(entity, source, tree, RouteType::Bound)` appends the same way.
- During initial effect dispatch, `Stamp(target, tree)` resolves to entities and appends here.

## How entries are removed

- By source: when a chip is unequipped or a definition is removed, all entries with that source string are removed from the Vec.

## Pairing with StagedEffects

BoundEffects and StagedEffects are always inserted as a pair. If an entity has one, it has both. Command extensions ensure this by inserting both when either is absent.
