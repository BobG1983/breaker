# Name
evaluate_conditions

# Reads
- `BoundEffects` on all entities (for During entries)
- `DuringActive` on all entities (runtime condition state)
- World state for each condition (NodeState resource, ShieldWall query, HighlightTracker for combo)

# Dispatches
N/A — this is not a bridge. It directly calls fire/reverse helpers and installs/removes armed entries.

# Scope
All entities with BoundEffects that have During entries.

# Source Location
`src/effect_v3/conditions/evaluate_conditions.rs`

# Schedule
FixedUpdate, in `EffectV3Systems::Conditions`, after `EffectV3Systems::Tick`.

# Behavior

## Flat During (simple case)

1. Evaluate each condition once for the whole world:
   - `NodeActive`: call `is_node_active(world)` → bool
   - `ShieldActive`: call `is_shield_active(world)` → bool
   - `ComboActive(n)`: call `is_combo_active(world, n)` → bool (per threshold value; reads `HighlightTracker.consecutive_perfect_bumps`)

2. For each entity with BoundEffects, collect all `During(Condition, scoped_tree)` entries recursively (including nested Durings inside other Durings).

3. For each During entry (keyed by source string):
   - Look up whether source is in `DuringActive` (= was active last frame).
   - Evaluate the current condition value from step 1.
   - **Transition false → true:**
     - Fire each scoped effect in the ScopedTree (see walking-effects/during.md)
     - Insert source into `DuringActive`
   - **Transition true → false:**
     - Reverse each scoped effect in the ScopedTree (see walking-effects/during.md)
     - Remove source from `DuringActive`
   - **No change:** do nothing.

## Nested Shape Handling

### Shape C — `During(Cond, When(Trigger, Fire(reversible)))`

On condition-becomes-true: calls `install_armed_entry()` which appends the inner `When(Trigger, Fire(...))` to `BoundEffects` (or `StagedEffects`) under source key `{original_source}#armed[0]`.

On condition-becomes-false: removes the armed entry by source and calls `reverse_all_by_source_dispatch` on all effects that fired from that scope while the condition was active.

### Shape D — `During(Cond, On(Participant, Fire(reversible)))`

Same as Shape C, but targets a participant entity. `install_armed_entry()` appends an armed `On(Participant, ...)` entry. On condition-becomes-false, reversal targets the same participant.

### Nested Durings (Shape A/B sub-trees)

`evaluate_conditions` recursively collects During nodes from all depths of the BoundEffects tree, not just top-level entries. A `{source}#installed[0]` During entry installed by Shape A/B is treated identically to a top-level During — it participates in condition evaluation on the next frame after installation.

## What this system does NOT do

- Does NOT call `walk_effects`. Conditions are not triggers.
- Does NOT remove top-level During entries from BoundEffects. During entries are permanent (unless the outer Until removes the `#installed[0]` entry in Shape B).
- Does NOT fire effects for conditions that haven't changed. Only transitions produce fire/reverse calls.
