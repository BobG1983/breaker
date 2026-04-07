---
name: Phase 5 file length findings — rantzsoft_stateflow + state transition
description: Wave 9 scan (2026-04-03 feature/wall-builder-pattern): 3 HIGH, 7 MEDIUM, 1 LOW in rantzsoft_stateflow + breaker-game/src/state. Spec at .claude/specs/file-splits.md
type: project
---

Updated 2026-04-03 on feature/wall-builder-pattern branch (Wave 9, rantzsoft_stateflow + state/ targeted scan).

## Scope

Scanned `rantzsoft_stateflow/src/` (22 files) and `breaker-game/src/state/` (235 files).

## HIGH priority (3 files)

| File | Total | Prod | Tests | Test Fns | Strategy |
|------|-------|------|-------|----------|----------|
| `rantzsoft_stateflow/src/transition/orchestration.rs` | 1518 | 297 | 1221 | 38 | A: test extraction + sub-split |
| `rantzsoft_stateflow/src/transition/effects/fade.rs` | 1142 | 236 | 906 | 38 | A: test extraction + sub-split |
| `rantzsoft_stateflow/src/transition/effects/mod.rs` | 441 | 148 | 293 | 7 | B: mod.rs violation |

## MEDIUM priority (7 files)

| File | Total | Prod | Tests | Test Fns | Strategy |
|------|-------|------|-------|----------|----------|
| `rantzsoft_stateflow/src/transition/effects/slide.rs` | 783 | 216 | 567 | 25 | A |
| `rantzsoft_stateflow/src/transition/effects/wipe.rs` | 730 | 283 | 447 | 18 | A |
| `rantzsoft_stateflow/src/transition/effects/iris.rs` | 712 | 239 | 473 | 17 | A |
| `rantzsoft_stateflow/src/dispatch.rs` | 695 | 179 | 516 | 16 | A |
| `rantzsoft_stateflow/src/transition/effects/pixelate.rs` | 678 | 252 | 426 | 14 | A |
| `rantzsoft_stateflow/src/transition/effects/dissolve.rs` | 675 | 251 | 424 | 14 | A |
| `breaker-game/src/state/run/chip_select/systems/snapshot_node_highlights.rs` | 692 | 48 | 644 | 17 | A |

## LOW priority (1 file)

| File | Total | Prod | Tests | Test Fns |
|------|-------|------|-------|----------|
| `rantzsoft_stateflow/src/plugin.rs` | 493 | 160 | 333 | 11 |

## Batching

- Batch 1 (parallel): orchestration.rs, effects/mod.rs, dispatch.rs, snapshot_node_highlights.rs
- Batch 2 (after mod.rs fix): fade.rs, slide.rs, wipe.rs, iris.rs, pixelate.rs, dissolve.rs

## Note on breaker-game/src/state/

All previously tracked state/ files from phase4 are unchanged. The only new file over threshold is snapshot_node_highlights.rs (692 lines, newly written as part of transition infrastructure).
