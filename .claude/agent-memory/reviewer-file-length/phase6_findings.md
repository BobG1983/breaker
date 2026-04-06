---
name: Phase 6 findings — full workspace scan
description: Wave 10 scan (2026-04-06 feature/effect-placeholder-visuals): 2 HIGH, 11 MEDIUM actionable, 1 mod.rs violation, 21 monitor, 6 LOW prod-only. rantzsoft_lifecycle dominates. Spec at .claude/specs/file-splits.md
type: project
---

Updated 2026-04-06 on feature/effect-placeholder-visuals branch (Wave 10, full workspace scan).

## Scope

Full workspace scan: breaker-game/src (872 files), rantzsoft_lifecycle/src (22 files), rantzsoft_spatial2d/src (49 files), rantzsoft_physics2d/src (22 files), rantzsoft_defaults/src (19 files), breaker-scenario-runner/src (147 files).

## Status vs Phase 5

Phase 5 flagged the same rantzsoft_lifecycle files. They have shrunk slightly (orchestration 1518->1302, fade 1142->1032, etc.) but remain over threshold. The effects/mod.rs violation shrunk from 441 to 375 (under 400 now, but still a violation).

New actionable files since phase 5:
- `breaker-game/src/state/plugin.rs` (505 lines, grew with resolve_node_next_state + tests)
- `breaker-game/src/effect/core/types/definitions/enums.rs` (506 lines, 86% production)

## HIGH priority (2 files)

| File | Total | Prod | Tests | Test Fns | Strategy |
|------|-------|------|-------|----------|----------|
| `rantzsoft_lifecycle/src/transition/orchestration.rs` | ~1302 | 297 | 1005 | 38 | A + sub-split |
| `rantzsoft_lifecycle/src/transition/effects/fade.rs` | ~1032 | 242 | 790 | 38 | A + sub-split |

## MEDIUM actionable (11 files)

dispatch.rs (735), snapshot_node_highlights.rs (692), wipe.rs (652), iris.rs (637), pixelate.rs (604), dissolve.rs (601), enums.rs (506), plugin.rs (505 state), slide.rs (480), lib.rs (444), plugin.rs (440 lifecycle)

## mod.rs violation (1 file, under threshold)

`rantzsoft_lifecycle/src/transition/effects/mod.rs` — 375 lines, production logic in mod.rs

## Batching

- Batch 1 (parallel): lib.rs, lifecycle/plugin.rs, dispatch.rs, orchestration.rs
- Batch 2 (serialize): effects/mod.rs fix first, then fade/wipe/iris/pixelate/dissolve/slide
- Batch 3 (parallel, concurrent with 1+2): state/plugin.rs, snapshot_node_highlights.rs, enums.rs
