# Spawned Trigger

## Trigger
- `Spawned(EntityKind)` — a new entity of the specified type was added to the world

## EntityKind
```rust
enum EntityKind { Bolt, Cell, Wall, Breaker }
```

## Locality: BRIDGE (PostFixedUpdate)
Not local or global in the normal sense — fires via 4 dedicated bridge systems that query `Added<T>`.

## Source
`Added<Bolt>`, `Added<Cell>`, `Added<Wall>`, `Added<Breaker>` — Bevy's built-in change detection.

## Bridge Systems (4, in PostFixedUpdate)
```
fn bridge_bolt_added(
    new_bolts: Query<Entity, Added<Bolt>>,
    registry: Res<OnSpawnEffectRegistry>,
    mut bound_query: Query<&mut BoundEffects>,
) {
    for new_bolt in &new_bolts {
        if let Some(entries) = registry.entries.get(&EntityKind::Bolt) {
            for entry in entries {
                // Insert tree into new bolt's BoundEffects
                stamp_tree(&mut bound_query, new_bolt, &entry.source, &entry.tree);
            }
        }
    }
}
// bridge_cell_added, bridge_wall_added, bridge_breaker_added — same pattern
```

## OnSpawnEffectRegistry
```rust
#[derive(Resource, Default)]
struct OnSpawnEffectRegistry {
    entries: HashMap<EntityKind, Vec<SpawnedEntry>>,
}
```
Populated at chip equip time when `Stamp(EveryBolt, ...)` desugars to `ActiveBolts` + `Spawned(Bolt)`.

## Notes
- **NEW** — does not exist in current system
- `Spawned` stamps trees onto new entities' BoundEffects (permanent), NOT StagedEffects
- Key use case: `Stamp(EveryBolt, ...)` desugars to stamp existing bolts + register in OnSpawnEffectRegistry for future bolts
- Runs in PostFixedUpdate — after FixedUpdate spawn systems have run, before next frame
- Not a normal trigger dispatch — does not call walk_effects. Directly stamps trees.
