---
name: Phase 14 findings -- new-cell-modifiers
description: Wave 18 scan (2026-04-16): 8 HIGH (cells/definition/tests 1371, cells/plugin 958, shape_d 1666, impact/bridges/tests 1219, death/bridges/tests 1184, basic_until 992, until_during 823, cells/resources/tests 813), 15 MEDIUM, 18 LOW. Detail at docs/todos/detail/2026-04-16-file-splits.md
type: project
---

## Wave 18 scan -- 2026-04-16

Scanned all `.rs` files in `breaker-game/` crate, focused on growth from `feature/new-cell-modifiers`.

### Key growth since Phase 12

- `cells/definition/tests.rs`: 510 -> 1371 (+861 lines, new survival + magnetic + volatile validation tests)
- `cells/plugin.rs`: was under threshold -> 958 (added sequence, armored, phantom, magnetic, survival cross-plugin tests)
- `cells/resources/tests.rs`: 723 -> 813 (+90 lines, toughness config tests)
- `effect_v3/triggers/impact/bridges/tests.rs`: was already split -> 1219 (salvo impact tests added)
- `effect_v3/triggers/death/bridges/tests.rs`: was already split -> 1184 (salvo death tests added)
- `effect_v3/conditions/evaluate_conditions/tests/shape_d.rs`: was already split -> 1666 (participant tracking tests)
- `effect_v3/walking/until/tests/basic_until.rs`: was already split -> 992 (grew organically)
- `effect_v3/walking/until/tests/until_during.rs`: was already split -> 823 (grew organically)

### Actionable (HIGH)

| File | Lines | Strategy | Status |
|------|-------|----------|--------|
| `cells/definition/tests.rs` | 1371 | C: sub-split into 8 test files + helpers | open |
| `cells/plugin.rs` | 958 | A: test extraction + C: sub-split tests | open |
| `conditions/evaluate_conditions/tests/shape_d.rs` | 1666 | C: sub-split into 5 test files | open |
| `triggers/impact/bridges/tests.rs` | 1219 | C: sub-split into 4 test files + helpers | open |
| `triggers/death/bridges/tests.rs` | 1184 | C: sub-split into 4 test files + helpers | open |
| `walking/until/tests/basic_until.rs` | 992 | C: sub-split into 4 test files | open |
| `walking/until/tests/until_during.rs` | 823 | C: sub-split into 3 test files | open |
| `cells/resources/tests.rs` | 813 | C: sub-split into 4 test files | open |

### Monitor (MEDIUM, 15 files)

All 500-799 lines. Will need splitting if they grow past 800.

### Detail
`docs/todos/detail/2026-04-16-file-splits.md`
