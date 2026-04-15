---
name: Volatile Wave 1 performance patterns
description: spawn-time patterns for VolatileCell archetype, Vec alloc, STAMP_SOURCE.to_owned(); ExplodeConfig::fire() QueryState-per-call pattern confirmed as pre-existing concern
type: project
---

## Patterns confirmed as acceptable at current scale

- `Vec<(Entity, Tree)>` in `spawn_inner()`: `Vec::new()` is zero-heap at zero length; only volatile cells push to it; spawn-time only.
- `STAMP_SOURCE.to_owned()` per volatile cell: String alloc at spawn-time only. One call per volatile cell per node load. Non-issue.
- `VolatileCell` + `VolatileDamage` + `VolatileRadius` archetype: Cells are ~50–200 total; volatile is a fraction of that. New archetype is correct and cheap. Components are storage-only (docs confirm no system reads them at runtime).
- Death bridge schedule change (`.after(DeathPipelineSystems::HandleKill)` instead of `in_set(EffectV3Systems::Bridge)`): pure ordering change, zero per-frame cost change.

## Pre-existing concern (flagged as IMPORTANT follow-up)

`ExplodeConfig::fire()` at `/Users/bgardner/dev/brickbreaker-2/breaker-game/src/effect_v3/effects/explode/config.rs` line 31:

```rust
world.query_filtered::<(Entity, &Position2D), (With<Cell>, Without<Dead>)>()
    .iter(world)
    .filter(...)
    .collect()
```

- Creates a transient `QueryState` (full archetype table scan) on every call.
- Called from `FireEffectCommand::apply()` — i.e., inside the command flush, directly against `&mut World`.
- In a chain of N volatile cells dying in the same tick or across ticks, this is N separate full-world cell scans.
- Violates `rantzsoft-crates.md` rule: game code must use `rantzsoft_spatial2d` or `rantzsoft_physics2d` for spatial queries.
- At current scale (50–200 cells, 1–few volatile per node) this is not a hitch, but it is the correct pattern to fix before Phase 3 when cell counts and chain reactions may scale.

**Recommended follow-up ticket**: Convert `ExplodeConfig::fire()` to use spatial query API from `rantzsoft_physics2d` (quadtree/radius query) instead of a full-world scan. Also cache `QueryState` or restructure as a proper system if Bevy's system executor can drive it.
