---
name: Phase 3 Stat Effects — Intentional Patterns
description: Patterns established in the Phase 3 feature/stat-effects branch that look like violations but are correct for this codebase
type: project
---

## Active*/Effective* component pair convention

Each stat effect is expressed as a pair of components:
- `Active*` (e.g. `ActiveSpeedBoosts(Vec<f32>)`) — accumulates fired values; can be a sum or product stack
- `Effective*` (e.g. `EffectiveSpeedMultiplier(f32)`) — read-only cached result, recomputed each `FixedUpdate` by a `recalculate_*` system

Consumers read **only** the `Effective*` component via `Option<&Effective*>` in queries, with the pattern `map_or(1.0, |e| e.0)` (or `map_or(0, |e| e.0)` for u32 additive types like piercing). Do NOT flag this as a redundant pattern — it is consistent project-wide by design.

## EffectiveDecelMultiplier and EffectiveBumpForce: exported but not yet consumed in Phase 3

`EffectiveDecelMultiplier` (from `quick_stop.rs`) and `EffectiveBumpForce` (from `bump_force.rs`) are re-exported from `effect/mod.rs` but as of Phase 3 their consumers in the breaker domain (deceleration system and bump velocity system) have not yet been migrated. This is not a dead-code issue — they are intentional scaffolding for Phase N. Do not flag these as unused if they are exported publicly.

## `multiplier()` method identical across Active* types

`ActiveSpeedBoosts`, `ActiveDamageBoosts`, `ActiveSizeBoosts`, `ActiveBumpForces`, `ActiveQuickStops` all have a `multiplier()` method with the same empty-returns-1.0 / product logic. This is intentional repetition per the plugin-per-domain convention rather than a shared trait. Do not flag without confirmation a shared trait is desired.

## `dispatch_breaker_effects` is a documented stub

`breaker/systems/init_breaker/system.rs` contains a `dispatch_breaker_effects` stub with a `// TODO: Wave 6` comment. This is a tracked in-progress item, not forgotten dead code. The TODO does not reference an issue number — flag the missing ticket reference.

## `unwrap()` in `bolt_wall_collision` face selection is safe

`faces.into_iter().min_by(...).unwrap()` at `bolt_wall_collision.rs:113` is safe: `faces` is a fixed 4-element array literal, so `min_by` on a non-empty iterator cannot return `None`. An invariant comment would strengthen this but it is not a bug.
