---
name: Phase 12 findings -- effect-system-refactor
description: Wave 16 scan (2026-04-14 effect-system-refactor): 10 HIGH, 13 MEDIUM, 10 LOW in effect_v3 domain. Detail at docs/todos/detail/2026-04-14-file-splits.md
type: project
---

## Scope

Feature scan: effect-system-refactor branch, focused on `breaker-game/src/effect_v3/`.

## HIGH (10 files, all Strategy A: test extraction + sub-split)

- `conditions/evaluate_conditions.rs` (3509 lines, 206 prod, 64 tests) -- 3 test groups: during_basic, shape_c, shape_d
- `triggers/bump/bridges.rs` (2718 lines, 311 prod, 52 tests) -- 5 test groups: no_bump, on_bumped, grade_filters, occurred, staged
- `walking/until.rs` (2206 lines, 227 prod, 27 tests) -- 3 test groups: basic_until, until_tracking, until_during
- `stacking/effect_stack.rs` (1049 lines, 81 prod, 57 tests) -- 3 test groups: push_pop, aggregate, multi_source
- `effects/circuit_breaker/systems.rs` (1031 lines, 64 prod, 25 tests)
- `effects/piercing_beam/config.rs` (1017 lines, 72 prod, 29 tests)
- `effects/pulse/systems.rs` (1015 lines, 137 prod, 25 tests)
- `effects/entropy_engine/systems.rs` (976 lines, 93 prod, 26 tests)
- `effects/tether_beam/systems.rs` (973 lines, 76 prod, 30 tests)
- `triggers/death/bridges.rs` (971 lines, 180 prod, 18 tests)

## MEDIUM (13 files)

- `triggers/impact/bridges.rs` (961)
- `watchers/tests/bolt.rs` (810, Strategy C)
- `effects/pulse/config.rs` (792)
- `effects/shield/config.rs` (773)
- `walking/when.rs` (733)
- `walking/on.rs` (713)
- `effects/tether_beam/config.rs` (683)
- `effects/shockwave/systems.rs` (629)
- `walking/during.rs` (624)
- `effects/chain_lightning/config.rs` (568)
- `triggers/bolt_lost/bridges.rs` (544)
- `walking/once.rs` (517)
- `triggers/node/bridges.rs` (509)

## LOW (10 files, 400-500 lines)

- second_wind/config.rs (495), time/bridges.rs (492), reverse_dispatch.rs (470),
  anchor/systems.rs (459), second_wind/systems.rs (449), chain_lightning/systems.rs (445),
  ramping_damage/config.rs (431), walk_effects.rs (427), commands/ext.rs (425),
  node/scan_thresholds.rs (410)

## Monitors (from previous scans, unchanged)

- cells/resources/tests.rs (723), cells/definition/tests.rs (510),
  spawn_cells_from_layout/system.rs (521), behaviors.rs (590), helpers.rs (447)

## Batching

5 parallel batches: walking, conditions+stacking+commands, trigger bridges, effect configs+systems, storage.
