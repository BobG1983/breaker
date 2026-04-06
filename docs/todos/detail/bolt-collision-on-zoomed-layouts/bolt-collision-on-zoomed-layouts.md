# Bolt Collision on Zoomed-Out Layouts

## Summary
Extra bolts spawned by effects during gameplay don't receive `NodeScalingFactor`, causing them to appear full-size and collide at unscaled radius on zoomed-out layouts.

## Context
During a play session (2026-04-06) with the Prism breaker on a zoomed-out island layout, extra bolts appeared oversized relative to cells. Research confirmed:

- **Collision is radius-based** (not point-based) — `cast_circle` does proper Minkowski expansion
- **Primary bolt scales correctly** — `apply_node_scale_to_bolt` inserts `NodeScalingFactor` on `OnEnter(NodeState::Loading)`
- **Extra bolts spawned by effects are the bug** — they never get `NodeScalingFactor` because the system only runs at node start, not when new bolts are spawned mid-gameplay
- **Bolt Aabb2D is stale** (spawned at unscaled radius, never updated) — no current impact since collision systems compute radius on the fly, but would break future quadtree-based bolt lookups

## Root Cause
`apply_node_scale_to_bolt` runs once on `OnEnter(NodeState::Loading)`. Bolts spawned later by effects (e.g., `spawn_bolts`, which Prism uses) don't have `NodeScalingFactor`. The collision fallback `node_scale.map_or(1.0, |s| s.0)` treats them as full-size.

## Scope
- In: Ensure all bolts (including effect-spawned extras) get `NodeScalingFactor`
- In: Fix bolt `Aabb2D` to reflect scaled radius (future-proofing)
- Out: Cell scaling (already correct via `compute_grid_scale`)

## Fix Options
1. **Option A**: Make `apply_node_scale_to_bolt` also run in `FixedUpdate` to catch newly-spawned bolts (query `Without<NodeScalingFactor>`)
2. **Option B**: Have the bolt builder insert `NodeScalingFactor` at spawn time by reading `ActiveNodeLayout`
3. **Option C**: Have effect `spawn_bolts` explicitly copy `NodeScalingFactor` from the source bolt to spawned bolts

Option A is simplest — one system change, no effect code touched.

## Dependencies
- Depends on: nothing
- Related to: `sync_bolt_scale` (visual), `apply_node_scale_to_bolt`

## Research
- [Bolt-cell collision flow](research/bolt-cell-collision-flow.md)
- [Node-scale bolt radius](research/node-scale-bolt-radius.md)

## Status
`ready`
