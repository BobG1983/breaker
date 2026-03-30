---
name: Scenario Coverage Branch — Intentional Patterns
description: Patterns from feature/scenario-coverage that look like violations but are intentional
type: project
---

## `unwrap()` on `get::<Velocity2D>(bolt)` in pull_tests.rs and tick_tests.rs

All `unwrap()` calls in gravity_well/tests/ are in `#[cfg(test)]` test code. Per project convention, `unwrap()` is acceptable in tests.

## Duplicate `tick()` / `test_app()` helpers across invariant checker modules

Each invariant checker file (check_chain_arc_count_reasonable.rs, check_pulse_ring_accumulation.rs, check_second_wind_wall_at_most_one.rs, etc.) defines its own local `tick()` and `test_app()` functions. These are short (5–10 line) scaffolding helpers, consistent with the co-located test helper pattern established in Phase 1. Do not flag as duplication.

## Glob re-export in `invariants/checkers/mod.rs` (`pub use check_xyz::*;`)

Every checker module is pub-used via `pub use module::*;`. This is intentional: the `invariants` module re-exports all checker functions flat for consumption by the lifecycle wiring. The number of modules (20+) exceeds the 4-item threshold for recommending `::*`, so the pattern is correct.

## `chip_a.clone()` in check_maxed_chip_never_offered tests

`chip_a.clone()` is used to insert the same chip into both `ChipOffers` and `ChipInventory`. `ChipDefinition` does not implement `Copy`, so `clone()` is required. Not an unnecessary clone.

## `bolt_count_test_app` name (not `test_app`) in bolt_count_reasonable.rs

The local test app factory is named `bolt_count_test_app` rather than `test_app`. This is intentional disambiguation — the function takes a `max_bolt_count` parameter, making the name more descriptive. Not a vague name issue.

## `owned.remove(0)` pattern in gravity_well/effect.rs fire()

The eviction loop in `fire()` uses `owned.remove(0)` — identical to the pattern in `spawn_phantom.rs` (documented in phase4-runtime-effects-patterns.md). Already established as intentional (bounded by `max_active`, not a hot path).
