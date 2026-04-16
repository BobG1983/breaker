# Commands Extension

The `EffectCommandsExt` trait extends Bevy's `Commands` with effect-system operations. Every cross-entity mutation and every effect application goes through this trait — bridge systems, walkers, and game systems queue commands, which then execute with `&mut World` access during command flush.

The trait and its impl live in `effect_v3/commands/ext/system.rs`. The concrete `Command` structs live in sibling files under `effect_v3/commands/` and are re-exported from `commands/mod.rs`.

## EffectCommandsExt Trait

```rust
pub trait EffectCommandsExt {
    fn fire_effect(&mut self, entity: Entity, effect: EffectType, source: String);
    fn reverse_effect(&mut self, entity: Entity, effect: ReversibleEffectType, source: String);

    fn route_effect(&mut self, entity: Entity, name: String, tree: Tree, route_type: RouteType);
    fn stamp_effect(&mut self, entity: Entity, name: String, tree: Tree);
    fn stage_effect(&mut self, entity: Entity, name: String, tree: Tree);

    fn remove_effect(&mut self, entity: Entity, name: &str);
    fn remove_staged_effect(&mut self, entity: Entity, name: String, tree: Tree);

    fn track_armed_fire(&mut self, owner: Entity, armed_source: String, participant: Entity);
}

impl EffectCommandsExt for Commands<'_, '_> { /* each method queues its concrete command */ }
```

All eight methods are deferred — each queues a concrete command that runs at the next command flush.

## Command Reference

### `fire_effect`

Queues `FireEffectCommand`. At flush time, calls `fire_dispatch(&effect, entity, &source, world)` which matches on the `EffectType` variant and calls the relevant `config.fire(entity, source, world)`.

Used by:
- `evaluate_fire` (walker `Tree::Fire` node)
- `evaluate_terminal` for `Terminal::Fire`
- Chip dispatch for bare `Tree::Fire` root children (passive stat boosts, etc.)

### `reverse_effect`

Queues `ReverseEffectCommand`. Calls `reverse_dispatch(&effect, entity, &source, world)`, which is the `ReversibleEffectType` analogue of `fire_dispatch`.

Used by:
- `evaluate_conditions` Shape D disarm (per-participant reversal of armed fires)
- Internal helpers when reversing single instances

The bulk of `Until`/`During` reversal does **not** go through `commands.reverse_effect` — it calls `reverse_dispatch` / `reverse_all_by_source_dispatch` directly from inside its command, because by then the system already has `&mut World`.

### `route_effect`

Queues `RouteEffectCommand` with an explicit `RouteType`. The general form behind `stamp_effect` and `stage_effect`. Used when the route type is data-driven (e.g. when reading a `Terminal::Route(route_type, tree)` from a `Sequence` or `On` terminal).

### `stamp_effect`

Sugar for `route_effect` with `RouteType::Bound`. Inserts the `(name, tree)` tuple into the entity's `BoundEffects`. Used by:
- Chip dispatch for non-`Fire` children (`commands.stamp_effect(entity, chip_name, tree)`)
- `SpawnStampRegistry` watcher systems on `Added<EntityKind>`
- `evaluate_when` and `evaluate_once` when arming a nested gate
- `evaluate_conditions` Shape A install (armed key under `#installed[0]`)

### `stage_effect`

Sugar for `route_effect` with `RouteType::Staged`. Inserts into `StagedEffects` instead of `BoundEffects`. Used by:
- `evaluate_when` when arming a nested gate (the inner gate becomes a one-shot staged entry)
- Effect implementations that need to install one-shot consequences

### `remove_effect`

Queues `RemoveEffectCommand`. Sweeps **both** `BoundEffects` and `StagedEffects` for any entry whose name matches and removes them. This is a broad cleanup — it removes all entries from a given source, not a specific tree.

Primary use: chip unequip (remove every entry stamped by the chip name).

The name-sweep behavior is load-bearing for `evaluate_once`: when a `Once` arms a nested gate by staging it under the same source, the order is **remove-first / stage-second**, so the remove clears the outer entry without touching the freshly-staged inner. See `evaluate_once`.

### `remove_staged_effect`

Queues `RemoveStagedEffectCommand`. Removes the **first** `StagedEffects` entry whose `(name, tree)` tuple matches exactly. **Does not touch `BoundEffects`.** Single-entry removal, not a sweep.

Used exclusively by `walk_staged_effects` to consume a staged entry that just matched its trigger. The exact-tuple match preserves any fresh same-name stages that were queued earlier in the same command flush — for example, when a `When` re-arms its inner gate during the same trigger event, the freshly-armed inner has a different `Tree` value than the original outer, so the outer is consumed and the inner survives.

### `track_armed_fire`

Queues `TrackArmedFireCommand`. Records `participant` in the owner's `ArmedFiredParticipants` component under the `armed_source` key. Used by `evaluate_on` when firing through an armed Shape-D scoped tree, so that the disarm path can later reverse effects on the exact participants they were fired on.

The participant vec is intentionally not deduplicated — N fires produce N reverses, matching `commands.reverse_effect`'s single-instance semantics.

## Why Defer Through Commands

Walker functions (`evaluate_when`, `evaluate_once`, `evaluate_fire`, etc.) run inside trigger bridge systems. Those bridge systems use `Query<(Entity, &BoundEffects, &mut StagedEffects)>` — they hold immutable and mutable component borrows, so they cannot also take `&mut World`. The `Fireable::fire` contract takes `&mut World` because effects need to query arbitrary components and spawn entities.

Deferring through `Commands` solves this: walkers queue commands cheaply (no world access required); commands flush at the next sync point with `&mut World` available. The walker stays parallelizable, and effect implementations get the world access they need.

The `evaluate_during` and `evaluate_until` walkers go a step further — they queue full Bevy `Command` structs (`DuringInstallCommand`, `UntilEvaluateCommand`) rather than calling an `EffectCommandsExt` method, because their state machines need world access and command-internal logic that doesn't fit a simple "fire this" call.

## Source Strings

The `source` parameter on `fire_effect` / `reverse_effect` is owned (`String`, not `&str`) because the command must outlive the borrow. It is stored on `EffectStack` entries (for stack-based passives) and on spawned effect entities via `EffectSourceChip` so that damage-application systems can attribute damage back to the originating chip.

Empty source strings (`""`) are a convention meaning "not chip-sourced" — `EffectSourceChip::from_source("")` produces `EffectSourceChip(None)`. See `core_types.md` for the helper.

Stamp/stage/route/remove use `name: String` rather than `source: String`. The two are conceptually the same — the chip or definition name that owns this entry — but `name` reads better for "remove all entries called X" than "remove all sources called X".
