---
name: Phase 13 findings — scenario-runner-tiling-streaming
description: Wave 17 scan (2026-04-15): 1 HIGH (app_tests.rs 1110 lines), 2 MEDIUM (tiling_tests.rs 562, app.rs 546). Detail at docs/todos/detail/2026-04-15-file-splits.md
type: project
---

## Wave 17 scan — 2026-04-15

Scanned changed files in `breaker-scenario-runner/src/runner/`.

### Actionable

| File | Lines | Priority | Strategy | Status |
|------|-------|----------|----------|--------|
| `breaker-scenario-runner/src/runner/tests/app_tests.rs` | 1110 | HIGH | C: sub-split into 8 test files + helpers | open |

### Monitor

| File | Lines | Priority | Notes |
|------|-------|----------|-------|
| `breaker-scenario-runner/src/runner/tests/tiling_tests.rs` | 562 | MEDIUM | Split if grows past 800 |
| `breaker-scenario-runner/src/runner/app.rs` | 546 | MEDIUM | Split if grows past 700 |

### Detail
`docs/todos/detail/2026-04-15-file-splits.md`
