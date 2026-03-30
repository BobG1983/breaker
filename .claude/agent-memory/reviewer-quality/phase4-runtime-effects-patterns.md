---
name: Phase 4 Runtime Effects — Intentional Patterns
description: Patterns established in the Phase 4 feature/runtime-effects branch that look like violations but are correct for this codebase
type: project
---

## Nested tuple spawning in spawn_phantom.rs and chain_bolt.rs

`world.spawn(((A, B, C), (D, E, F)))` — splitting a large component bundle across two inner tuples. This is a Bevy workaround for tuple-arity limits (max 15 items) on `Bundle`. Do not flag as odd grouping without first counting the total component count.

## `owned.remove(0)` in spawn_phantom.rs fire()

The max_active eviction loop uses `owned.remove(0)` to despawn the oldest phantom. This is Vec<Entity> (not Vec<T: Clone+sized>), collected in insertion order. The loop terminates when `owned.len() < max_active`, so the O(n) shift is bounded by max_active (typically 1–3). Not a hot path. Do not flag as a performance issue.

## `_entity` param in second_wind.rs fire() / reverse()

`fire(_entity: Entity, ...)` and `reverse(_entity: Entity, ...)` — the entity parameter is part of the uniform `fire/reverse` interface for all effect handlers but is not used by second_wind (position comes from PlayfieldConfig, not the entity). The underscore prefix is correct.

## `PhantomTimer` — REMOVED (feature/runtime-effects)

`PhantomTimer` was a backward-compatibility stub component. It has been removed entirely.
`spawn_phantom.rs` now uses `BoltLifespan(Timer)` from the bolt domain. Do NOT flag the absence of `PhantomTimer` as a missing component — it is intentionally gone.

## `unwrap()` calls in bolt_lost/tests.rs

The `unwrap()` calls at test lines (e.g., `query.iter().next().unwrap()`) are all inside `#[cfg(test)]` test code. Per project conventions, `unwrap()` is acceptable in tests.

## `reverse()` no-op implementations — `const fn` pattern (updated)

After the full-verification-fixes branch, `explode.rs`, `shockwave.rs`, `gravity_well.rs`, `piercing_beam/effect.rs`, and `life_lost.rs` implement no-op reversals as `pub(crate) const fn reverse(_entity: Entity, _source_chip: &str, _world: &mut World) {}`. The `const fn` form is now the established pattern — it avoids the need for `let _ = world` by using underscore-prefixed parameters. Do NOT flag as an idiom violation.

## `WallSize {}` in second_wind.rs

`WallSize {}` is spawned as part of the second wind wall bundle. The struct has no fields — it is a unit-struct marker. The `{}` syntax is required because Bevy's `Bundle` derive needs a complete struct literal, even for empty structs without Default. Do not flag as unnecessary brace syntax.
