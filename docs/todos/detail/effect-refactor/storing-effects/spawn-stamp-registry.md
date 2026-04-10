# Name
SpawnStampRegistry

# Struct
```rust
#[derive(Resource, Default)]
struct SpawnStampRegistry(Vec<(String, EntityKind, Tree)>);
```

# Description
Global registry of spawn watchers. A resource, not a component — it lives on the world, not on any specific entity.

Each entry is a (source, entity_kind, tree) triple. When a new entity of the matching kind appears in the world, the tree is stamped onto it via stamp_effect.

## How entries are added

- During initial effect dispatch, `Spawn(kind, tree)` root nodes register a watcher here.
- `Every*` stamp targets (EveryBolt, EveryCell, EveryWall, EveryBreaker) register a watcher here as part of their resolution — the Active* half stamps onto existing entities, the Spawn half registers here for future entities.

## How entries are watched

When a new entity is spawned with a Bolt, Cell, Wall, or Breaker component, the system checks the registry for matching EntityKind entries. For each match, the tree is cloned and stamped onto the new entity via stamp_effect. EntityKind::Any matches all entity types.

Each spawned entity gets its own independent copy of the tree.

## How entries are removed

By source: when a chip is unequipped or a definition is removed, all entries with that source string are removed from the Vec. Trees already stamped onto existing entities by this watcher are not removed by deregistration — they follow the normal removal path through BoundEffects source cleanup.
