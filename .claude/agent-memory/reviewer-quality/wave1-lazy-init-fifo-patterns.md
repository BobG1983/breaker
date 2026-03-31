---
name: Wave 1 Lazy-Init Stat-Boost and FIFO Effects — Patterns
description: Patterns from feature/scenario-coverage Wave 1 (stat-boost lazy init, GravityWell FIFO, SpawnPhantom FIFO) that look like violations but are intentional
type: project
---

## Dual-guard initialization pattern in stat-boost `fire()` functions

All five stat-boost files (speed_boost, damage_boost, size_boost, bump_force, piercing) use a
two-step guard in `fire()`:
1. If `Active*` is absent, insert both `Active*` and `Effective*` as a pair.
2. If `Effective*` is absent (still), insert `Effective*` alone.

The second guard handles the "half-initialized entity" case: an entity that already has `Active*`
but no `Effective*` (e.g. inserted externally). The test `fire_on_half_initialized_entity_inserts_effective`
exercises this path. The redundancy is structural — it IS reachable — so do NOT flag the second
guard as dead code or "logically unreachable."

**Why:** Entities may have `Active*` injected externally (e.g. in tests or other init paths) without
`Effective*`. The second guard is a defensive correctness check, not dead code.

## `if let Some(mut counter_resource)` in SCOPE C of gravity_well/effect.rs

`gravity_well/effect.rs` SCOPE C uses `world.get_resource_mut::<GravityWellSpawnCounter>()` wrapped
in `if let Some(...)`. The resource is guaranteed present because SCOPE A called
`get_resource_or_insert_with`. This `if let Some` is redundant but defensive. Flag as [Nit] only.

## `if let Some(&(oldest, _)) = owned.first()` in spawn_phantom/effect.rs fire()

Inside the `while owned.len() >= max_active as usize` loop (line 58-63 of spawn_phantom/effect.rs),
`owned.first()` is wrapped in `if let Some`. The loop condition guarantees `owned` is non-empty, so
`owned.first()` cannot return `None` inside the loop. This `if let Some` is dead code. The inner
`owned.remove(0)` is always reached. Flag as [Nit].

## `value` terminology in size_boost.rs tests

The test function `fire_pushes_value_onto_active_size_boosts` (line 83) and
`reverse_removes_matching_value` (line 159) use the word "value" where the domain convention
(established in speed_boost.rs, damage_boost.rs) uses "multiplier". These names are inconsistent
with the domain pattern — flag as vocabulary inconsistency (test names).

## `GravityWellSpawnOrder` / `GravityWellSpawnCounter` visibility in mod.rs

`gravity_well/mod.rs` re-exports `GravityWellSpawnOrder` and `GravityWellSpawnCounter` via
`pub use effect::{ ... }` — meaning they are `pub` (crate-external). These types are internal
FIFO machinery and currently have no external consumers. They should be `pub(crate)` unless the
scenario runner or external tests need them. This is consistent with the previously noted `GravityWellMarker`/
`GravityWellConfig` visibility nit.

## `PhantomSpawnOrder` / `PhantomSpawnCounter` visibility in spawn_phantom

`PhantomSpawnOrder` and `PhantomSpawnCounter` are declared `pub(crate)` in effect.rs, re-exported
via `pub(crate) use effect::*` in mod.rs. This is correct — appropriately scoped to the crate.
Do NOT flag.

## Borrow-scope comments (SCOPE A / B / C) in gravity_well/effect.rs

The `// SCOPE A — ...`, `// SCOPE B — ...`, `// SCOPE C — ...` comments in `fire()` explain why
borrows are split across explicit blocks (same motivation as the chain_lightning borrow-scope comment).
This is an intentional safety comment pattern. Do NOT flag as noise.
