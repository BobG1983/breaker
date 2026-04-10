# Walking Algorithm

## Signature

```rust
fn walk_effects(
    entity: Entity,
    trigger: &Trigger,
    context: &TriggerContext,
    world: &mut World,
);
```

Called by bridge systems. Walks the entity's StagedEffects then BoundEffects for matching trees.

When a trigger fires on an entity, walk that entity's effect trees in this order.

## Step 1: Walk StagedEffects

Iterate every entry in StagedEffects. For each (source, tree):

1. Check if the tree's outermost node matches the trigger.
2. If it matches, evaluate the tree (see the per-node files).
3. Mark the entry for removal — it has been consumed.

After iterating, remove all marked entries from StagedEffects.

**Why staged first:** Effects can be multi-stage with more than one trigger in the chain. When a staged entry matches, its inner tree may produce new entries in BoundEffects or StagedEffects (via Route). If we walked BoundEffects first, a tree like `When(Bumped, ...)` in BoundEffects could evaluate and route a `When(Bumped, ...)` into StagedEffects — and if we then walked StagedEffects, that inner When would match the same Bumped trigger in the same frame. Walking staged first prevents a single trigger from cascading through multiple stages in one pass.

## Step 2: Walk BoundEffects

Iterate every entry in BoundEffects. For each (source, tree):

1. Check if the tree's outermost node matches the trigger.
2. If it matches, evaluate the tree (see the per-node files).
3. DO NOT remove the entry — bound entries persist and re-arm.

Exception: Once nodes remove themselves from BoundEffects after matching. The per-node evaluation for Once handles this.

## When to call command extensions

- Call `fire_effect` when the walker reaches a Fire leaf and needs to execute an effect on an entity.
- Call `reverse_effect` when a During condition becomes false or an Until trigger fires and scoped effects need undoing.
- Call `route_effect` when the walker reaches a Route terminal inside On and needs to install a tree on another entity.
- DO NOT call `stamp_effect` during walking. Stamp is for initial installation from definitions, not for runtime tree evaluation. Route(Bound) handles the runtime equivalent.

## Constraints

- DO iterate StagedEffects before BoundEffects, always.
- DO NOT modify the Vec while iterating — collect removals and apply after.
- DO NOT remove BoundEffects entries on match (except Once).
- DO remove StagedEffects entries on match.
- DO pass the trigger context (which entities were involved in the event) through to On node evaluation so participants can be resolved.
