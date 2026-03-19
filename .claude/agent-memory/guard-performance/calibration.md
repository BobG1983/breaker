---
name: Performance Calibration
description: Entity scale and scenario count calibration for severity ratings (as of 2026-03-19)
type: project
---

## Calibration

- Entity scale: fixed grid (cells per layout), 1 bolt, 1 breaker. Max ~50-200 entities total.
- Scenario count: 29 RON files (as of 2026-03-19); 2 are stress scenarios.
- Phase 2 is active. Phase 3 (upgrades/full content) will increase entity counts.

**Why:** Scale determines whether concerns are real. With 1 bolt and ~50 cells per node, most hot-path issues are academic until Phase 3+ with upgrades multiplying bolt count.

**How to apply:** Flag Critical only if it causes hitches at current scale. Flag Moderate if it would hurt at 200+ entities or full upgrade system. Flag Minor for everything else.
