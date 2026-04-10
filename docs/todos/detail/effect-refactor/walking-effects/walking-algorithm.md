# Walking Algorithm

## What it is

A helper function called by bridge systems. Not a system itself. Bridges are regular systems that read messages, determine which entities to walk, and call this function for each entity.

## Signature

```rust
fn walk_effects(
    entity: Entity,
    trigger: &Trigger,
    context: &TriggerContext,
    bound: &BoundEffects,
    staged: &StagedEffects,
    commands: &mut Commands,
);
```

Bridges have `&BoundEffects` and `&StagedEffects` (read-only). All mutations happen through deferred commands (`fire_effect`, `reverse_effect`, `route_effect`, `stage_effect`, `remove_effect`). This means bridges are regular systems that can run in parallel — no exclusive world access needed.

## Step 1: Walk StagedEffects

Iterate every entry in StagedEffects. For each (source, tree):

1. Check if the tree's outermost node matches the trigger (see Trigger Matching below).
2. If it matches, evaluate the tree (see the per-node files).
3. Queue `remove_effect(entity, Staged, source, tree)` — the entry has been consumed.

All matching entries fire and are consumed, not just the first. If two chips both staged a `When(Bumped, ...)`, both fire on the same bump.

**Exception:** During nodes in StagedEffects have special lifecycle handling — see `walking-effects/during.md`. They are NOT consumed on first match.

**Why staged first:** Effects can be multi-stage with more than one trigger in the chain. Walking staged first prevents a single trigger from cascading through multiple stages in one pass. If a BoundEffects entry matches and routes a new `When(Bumped, ...)` into StagedEffects, that inner When does NOT match the same Bumped trigger this frame — it will match on the next Bumped.

## Step 2: Walk BoundEffects

Iterate every entry in BoundEffects. For each BoundEntry { source, tree, condition_active }:

1. If `condition_active` is `Some(_)`, skip — this is a During entry handled by `evaluate_conditions`, not by trigger walking.
2. Check if the tree's outermost node matches the trigger.
3. If it matches, evaluate the tree (see the per-node files).
4. DO NOT remove the entry — bound entries persist and re-arm.

Exception: Once nodes queue `remove_effect(entity, Bound, source, tree)` after matching. The per-node evaluation for Once handles this.

## When to call command extensions

All mutations are deferred via commands. The walker never mutates BoundEffects, StagedEffects, or the World directly.

| Situation | Command |
|-----------|---------|
| Fire leaf reached | `fire_effect(entity, effect_type, source)` |
| During/Until scoped effect needs reversing | `reverse_effect(entity, effect_type, source)` |
| Route terminal reached (install tree on another entity) | `route_effect(target_entity, source, tree, route_type)` |
| Nested trigger gate needs arming | `stage_effect(entity, source, inner_tree)` |
| Once matched (remove from Bound) | `remove_effect(entity, Bound, source, tree)` |
| Staged entry consumed | `remove_effect(entity, Staged, source, tree)` |

## Trigger Matching

Exact equality match on the Trigger enum variant and its parameters.

- `Trigger::Bumped` matches only `Trigger::Bumped`
- `Trigger::Impacted(Cell)` matches only `Trigger::Impacted(Cell)`, not `Trigger::Impacted(Bolt)`
- `Trigger::TimeExpires(5.0)` matches only `Trigger::TimeExpires(5.0)` (OrderedFloat equality)
- `Trigger::DeathOccurred(Cell)` matches only `Trigger::DeathOccurred(Cell)`

No wildcards. No partial matching. The Trigger enum derives `PartialEq` and `Eq`.

## Entity Safety

Entities are never despawned during FixedUpdate. The death pipeline defers despawn to PostFixedUpdate via `process_despawn_requests`. The walker can safely iterate all entries without checking entity validity mid-walk.

## Constraints

- DO iterate StagedEffects before BoundEffects, always.
- DO NOT mutate BoundEffects or StagedEffects directly — use command extensions.
- DO NOT remove BoundEffects entries on match (except Once via remove_effect command).
- DO remove StagedEffects entries on match (via remove_effect command).
- DO pass the trigger context through to On node evaluation so participants can be resolved.
