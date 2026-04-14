# Walking Algorithm

## What it is

A helper function called by bridge systems. Not a system itself. Bridges are regular systems that read messages, determine which entities to walk, and call this function for each entity.

## Signature

```rust
fn walk_effects(
    entity: Entity,
    trigger: &Trigger,
    context: &TriggerContext,
    trees: &[(String, Tree)],
    commands: &mut EffectCommands,
);
```

Bridges pass a flat slice of `(source, tree)` pairs from `BoundEffects`. All mutations happen through deferred commands (`FireEffectCommand`, `RemoveEffectCommand`, `DuringInstallCommand`, `UntilEvaluateCommand`, `RouteEffectCommand`). This means bridges are regular systems that can run in parallel — no exclusive world access needed.

> **Note**: The original design specified separate `BoundEffects` and `StagedEffects` parameters with different walking semantics (StagedEffects consumed first, BoundEffects retained except Once). The implementation simplified this to a flat slice model. `DuringActive` (a `HashSet<String>`) as a separate component replaces the original `condition_active` field on BoundEffects entries. The behavior achieved is equivalent.

## Walking BoundEffects

Iterate every entry in the slice. For each (source, tree):

1. Check if the tree's outermost node matches the trigger (see Trigger Matching below).
2. If it matches, evaluate the tree (see the per-node files).
3. DO NOT remove the entry — bound entries persist and re-arm.

Exception: Once nodes queue `RemoveEffectCommand` after matching. The per-node evaluation for Once handles this.

Exception: During nodes with an `#installed` suffix in their source are managed by `evaluate_conditions`, not by trigger walking. The `evaluate_during` function skips entries whose source contains `#installed`.

## When to call command extensions

All mutations are deferred via commands. The walker never mutates BoundEffects or the World directly.

| Situation | Command |
|-----------|---------|
| Fire leaf reached | `FireEffectCommand` |
| During/Until scoped effect needs reversing | via `reverse_scoped_tree` / `reverse_all_by_source_dispatch` |
| Route terminal reached (install tree on another entity) | `RouteEffectCommand` |
| During node encountered | `DuringInstallCommand` |
| Until node encountered | `UntilEvaluateCommand` |
| Once matched (remove from Bound) | `RemoveEffectCommand` |

## Trigger Matching

Exact equality match on the Trigger enum variant and its parameters.

- `Trigger::Bumped` matches only `Trigger::Bumped`
- `Trigger::Impacted(Cell)` matches only `Trigger::Impacted(Cell)`, not `Trigger::Impacted(Bolt)`
- `Trigger::TimeExpires(5.0)` matches only `Trigger::TimeExpires(5.0)` (OrderedFloat equality)
- `Trigger::DeathOccurred(Cell)` matches only `Trigger::DeathOccurred(Cell)`

No wildcards. No partial matching. The Trigger enum derives `PartialEq` and `Eq`.

## Entity Safety

Entities are never despawned during FixedUpdate. The death pipeline defers despawn to FixedPostUpdate via `process_despawn_requests`. The walker can safely iterate all entries without checking entity validity mid-walk.

## Constraints

- DO NOT mutate BoundEffects directly — use command extensions.
- DO NOT remove BoundEffects entries on match (except Once via RemoveEffectCommand).
- DO pass the trigger context through to On node evaluation so participants can be resolved.
