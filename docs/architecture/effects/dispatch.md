# Dispatch

"Dispatch" means installing chip/breaker/cell effect trees onto entities. **Dispatch is not part of the effect domain.** Each entity domain handles its own initialization and calls `EffectCommandsExt` methods to install the effects into the effect storage components.

## Who dispatches

| Domain | System | Reads | Writes |
|---|---|---|---|
| **chips** | `dispatch_chip_effects` (`chips/systems/dispatch_chip_effects/system.rs`) | `ChipSelected` messages, `ChipCatalog` | `commands.stamp_effect` / `commands.fire_effect` |
| **breaker** | breaker initialization | `BreakerDefinition` | same |
| **cells** | cell initialization | `CellDefinition.effects` (optional) | same |

All three follow the same pattern. The chip dispatch is the canonical example because it handles the most cases.

## RootNode shape

Effect definitions are lists of `RootNode`:

```rust
pub enum RootNode {
    Stamp(StampTarget, Tree),
    Spawn(EntityKind, Tree),
}
```

The chip's `effects` field is `Vec<RootNode>`. Each root is dispatched independently.

## chip dispatch flow

`dispatch_chip_effects` reads `ChipSelected` messages, resolves each chip in the catalog, optionally records it in `ChipInventory`, and walks the chip's `effects: Vec<RootNode>`:

```rust
for root in &effects {
    match root {
        RootNode::Stamp(target, tree) => {
            if *target == StampTarget::Breaker {
                // Direct dispatch — breaker exists at chip-select time.
                let entities = resolve_target_entities(*target, &targets);
                for entity in entities {
                    dispatch_tree(entity, tree, &chip_name, &targets, &mut commands);
                }
            } else {
                // Deferred dispatch — non-Breaker entities don't exist now.
                // Stamp the tree to every Breaker; trigger bridges handle walking later.
                for breaker_entity in targets.breakers.iter() {
                    commands.stamp_effect(breaker_entity, chip_name.clone(), tree.clone());
                }
            }
        }
        RootNode::Spawn(_kind, _tree) => {
            // Spawn-based roots are not yet used in chips.
        }
    }
}
```

The branch on `StampTarget::Breaker` is the key behavior. Breakers exist at chip-select time (the player has already chosen a breaker), so the chip's effects can be stamped directly. Bolts, cells, and walls don't exist yet (chip selection happens between nodes), so the trees are stamped onto the breaker — which is the only entity that exists — and the trigger bridges handle the actual walking when nodes start.

## dispatch_tree helper

```rust
fn dispatch_tree(entity: Entity, tree: &Tree, chip_name: &str, _targets: &DispatchTargets, commands: &mut Commands) {
    // Ensure storage components exist before stamping
    commands.entity(entity).insert_if_new(BoundEffects::default());
    commands.entity(entity).insert_if_new(StagedEffects::default());

    match tree {
        Tree::Fire(effect) => {
            commands.fire_effect(entity, effect.clone(), chip_name.to_owned());
        }
        other => {
            commands.stamp_effect(entity, chip_name.to_owned(), other.clone());
        }
    }
}
```

The two cases:

1. **`Tree::Fire(effect)` at the root** — fire immediately. This is the path for chips that are pure passives: `Fire(SpeedBoost(multiplier: 1.5))` at the root just queues a single `FireEffectCommand` and the speed boost is applied without ever touching `BoundEffects`.
2. **Anything else** — stamp into `BoundEffects` for trigger evaluation. `When`, `Once`, `During`, `Until`, `Sequence`, `On` all go this route.

The `insert_if_new` calls ensure the storage components exist before the stamp lands. Entities don't carry `BoundEffects` / `StagedEffects` from spawn — they are inserted lazily on first effect dispatch.

## resolve_target_entities

```rust
fn resolve_target_entities(target: StampTarget, targets: &DispatchTargets) -> Vec<Entity> {
    match target {
        StampTarget::Breaker | StampTarget::ActiveBreakers | StampTarget::EveryBreaker => {
            targets.breakers.iter().collect()
        }
        StampTarget::Bolt | StampTarget::ActiveBolts | StampTarget::EveryBolt
        | StampTarget::PrimaryBolts | StampTarget::ExtraBolts => {
            targets.bolts.iter().collect()
        }
        StampTarget::ActiveCells | StampTarget::EveryCell => targets.cells.iter().collect(),
        StampTarget::ActiveWalls | StampTarget::EveryWall => targets.walls.iter().collect(),
    }
}
```

Note that `Active*` and `Every*` collapse to the same query result. The "future spawns" semantics of `Every*` is **not** implemented at the dispatch layer — it would belong in `SpawnStampRegistry`, but the current chip dispatch doesn't write to the registry for `Every*` targets. The `Spawn`-rooted RootNode path is the proper mechanism for "install on every future spawn."

## Spawn rooted trees and the registry

```rust
RootNode::Spawn(EntityKind, Tree)
```

A `Spawn` root says "register this tree to be installed on every entity of `kind` when it is added." The registry mechanism is built on the per-kind watcher systems in `effect_v3/storage/spawn_stamp_registry/watchers/`:

```rust
pub(crate) fn stamp_spawned_bolts(
    registry: Res<SpawnStampRegistry>,
    new_bolts: Query<Entity, Added<Bolt>>,
    mut commands: Commands,
) {
    const KIND: EntityKind = EntityKind::Bolt;
    if registry.entries.is_empty() { return; }
    for entity in &new_bolts {
        for (entry_kind, name, tree) in &registry.entries {
            if *entry_kind == KIND {
                commands.stamp_effect(entity, name.clone(), tree.clone());
            }
        }
    }
}
```

The four watchers (`stamp_spawned_bolts`, `stamp_spawned_cells`, `stamp_spawned_walls`, `stamp_spawned_breakers`) are registered into `EffectV3Systems::Bridge` by `EffectV3Plugin::build`. They run every tick and check `Added<T>` — since `Added` is reset each tick, each newly-spawned entity is processed exactly once.

`EntityKind::Any` entries in the registry are ignored by all four watchers — wildcarding is reserved for trigger-side matching, not spawn-time stamping.

The chip dispatch system does **not currently populate** `SpawnStampRegistry` from chip definitions — `RootNode::Spawn(_, _)` is matched but no-op. When chip definitions need spawn-rooted effects, the dispatch system will need to write to `SpawnStampRegistry.entries`.

## Same-tick dispatch ordering

Within a tick:

1. `dispatch_chip_effects` reads `ChipSelected` messages and queues `stamp_effect` / `fire_effect` commands.
2. Commands flush at the next sync point.
3. The next FixedUpdate tick: trigger bridges read game messages and walk the freshly-stamped trees.

There is no race between dispatch and walking because dispatch always happens before the first walk through the new entries — `stamp_effect` is a deferred command, not an immediate insertion. By the time the next bridge runs, the storage components have the new entries.

## Why dispatch lives outside the effect domain

The effect domain owns the storage, the walker, and the dispatch primitives (`EffectCommandsExt`). It does **not** own:

- The decision of "when does a chip get equipped" (that's the chip system).
- The `ChipDefinition` type (that's the chip system).
- The `BreakerDefinition` type (that's the breaker system).
- The `CellDefinition` type (that's the cells system).

If the effect domain owned chip dispatch, the chip system would have to expose its catalog and inventory types to the effect domain, which would couple the two heavily. Keeping dispatch in the chip domain means the effect domain only depends on `Entity`, the storage components, and `EffectCommandsExt` — nothing chip-specific.

The same logic applies to breaker and cell initialization.
