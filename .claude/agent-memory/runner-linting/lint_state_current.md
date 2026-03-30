---
name: Current lint state
description: Full workspace lint result as of 2026-03-30 on develop — ALL PASS after error-fix wave
type: project
---

Last run: 2026-03-30 (develop branch)
Branch HEAD: 408fa19 (docs(design): refine graphics design docs) + fx/transition/tests.rs import fix

## Format: PASS (no changes)

## Clippy (all-dclippy): PASS
- rantzsoft_spatial2d: PASS
- rantzsoft_physics2d: PASS
- rantzsoft_defaults: PASS
- breaker-game: PASS
- breaker-scenario-runner: PASS

## Previous errors (now resolved)
- `rantzsoft_physics2d/src/quadtree/tests/basic_ops_tests.rs:549` — `clippy::similar_names` — FIXED
- `breaker-game/src/chips/definition/tests.rs:515` — `clippy::manual_string_new` — FIXED
- `breaker-game/src/effect/effects/fire_helpers.rs:148` — `clippy::cast_lossless` — FIXED

## Previous warnings (now resolved)
- `breaker-game/src/bolt/resources.rs:133` — `clippy::redundant_clone` — FIXED
- `breaker-game/src/chips/resources/tests/chip_catalog.rs:406` — `clippy::redundant_clone` — FIXED
- `breaker-game/src/effect/commands.rs:1083` — `clippy::redundant_clone` — FIXED
