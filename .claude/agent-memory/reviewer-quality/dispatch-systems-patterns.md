---
name: Effect Dispatch Systems — Intentional Patterns
description: Patterns from dispatch_cell_effects, dispatch_chip_effects, dispatch_breaker_effects that look like violations but are intentional
type: project
---

## `.clone()` on non_do_children / bound_children per target entity in dispatch systems

In `dispatch_cell_effects/system.rs` (line 89) and `dispatch_breaker_effects/system.rs` (line 68),
`non_do_children.clone()` and `bound_children.clone()` are called for each resolved target entity.
This is required because `push_bound_effects` takes `Vec<(String, EffectNode)>` by value (Commands
consume owned data). The per-target clone is bounded by the entity count and effects count, both
of which are small in practice. Do NOT flag as unnecessary clone.

## `GravityWellMarker` and `GravityWellConfig` are `pub` but only used internally

`GravityWellMarker` and `GravityWellConfig` in `gravity_well.rs` are declared `pub` but have no
external consumers (only one file references them). They should ideally be `pub(crate)`. This is
a visibility width issue, not an intentional pattern. Flag as [Nit].

## `value` parameter name in `size_boost.rs::fire()` and `reverse()`

The parameter is named `value` instead of `multiplier` (as used in `damage_boost.rs`, `speed_boost.rs`,
`quick_stop.rs`). `value` is vague. The correct name is `multiplier` to match the domain convention.

## Duplicated test infra in `track_cells_destroyed.rs`

`TestMessages`/`enqueue_messages`/`test_app` and `TestCellDestroyedAtMsgs`/`enqueue_cell_destroyed_at`/
`test_app_cell_destroyed_at` are duplicate test infrastructure left from a C7 migration (CellDestroyed ->
CellDestroyedAt). The two test functions test identical behavior via identical message types.
`track_cells_destroyed_reads_cell_destroyed_at` can be removed — it duplicates `increments_cells_destroyed_for_each_message`.
