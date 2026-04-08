---
name: Phase 9 findings -- bolt-birthing-animation
description: Wave 13 scan (2026-04-08 feature/bolt-birthing-animation): 1 MEDIUM (tick_birthing.rs 605 lines), 2 LOW (terminal.rs 489, birthing_tests.rs 438). Spec at .claude/specs/file-splits.md
type: project
---

## Scope

Branch scan: all .rs files changed or created on feature/bolt-birthing-animation vs develop.

## MEDIUM (1 file)

- `breaker-game/src/bolt/systems/tick_birthing.rs` (605 lines, 33 prod, 572 test, 10 fns) -- Strategy A: test extraction

## LOW (2 files, monitor only)

- `breaker-game/src/bolt/builder/core/terminal.rs` (489 lines, all prod, 0 tests) -- 8 typestate terminal impls, could split headless vs rendered
- `breaker-game/src/bolt/builder/tests/birthing_tests.rs` (438 lines, all test, 10 fns) -- already extracted, under 800

## Near-threshold watch

- `rantzsoft_stateflow/src/transition/effects/post_process.rs` at 387 lines
- `breaker-game/src/state/plugin/system.rs` at 363 lines

## Batching

Single writer-code agent for tick_birthing.rs split.
