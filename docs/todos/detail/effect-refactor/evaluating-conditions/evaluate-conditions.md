# Name
evaluate_conditions

# Reads
- `BoundEffects` on all entities (for During entries)
- World state for each condition (NodeState resource, ShieldWall query, combo streak)

# Dispatches
N/A — this is not a bridge. It directly calls `fire_effect` and `reverse_effect`.

# Scope
All entities with BoundEffects that have During entries.

# Source Location
`src/effect/systems/evaluate_conditions.rs`

# Schedule
FixedUpdate, in `EffectSystems::Conditions`, after `EffectSystems::Tick`.

# Behavior

1. Evaluate each condition once for the whole world:
   - `NodeActive`: call `is_node_active(world)` → bool
   - `ShieldActive`: call `is_shield_active(world)` → bool
   - `ComboActive(n)`: call `is_combo_active(world, n)` → bool (per threshold value)

2. For each entity with BoundEffects, for each BoundEntry where `condition_active` is `Some(was_active)`:
   a. Read the During entry's Condition from the tree.
   b. Look up the current condition value from step 1.
   c. If `was_active == false` and now `true`:
      - Fire each scoped effect in the ScopedTree (see walking-effects/during.md "When the condition becomes true")
      - Set `condition_active` to `Some(true)`
   d. If `was_active == true` and now `false`:
      - Reverse each scoped effect in the ScopedTree (see walking-effects/during.md "When the condition becomes false")
      - Set `condition_active` to `Some(false)`
   e. If no change: do nothing.

This system does NOT:
- Call `walk_effects`. Conditions are not triggers.
- Remove During entries from BoundEffects. During entries are permanent.
- Evaluate non-During entries. Entries with `condition_active: None` are skipped.
- Fire effects for conditions that haven't changed. Only transitions produce fire/reverse calls.
