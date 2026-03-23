---
name: Performance Calibration
description: Entity scale and scenario count calibration for severity ratings (as of 2026-03-23)
type: project
---

## Calibration

- Entity scale: fixed grid (cells per layout), 1 bolt, 1 breaker. Max ~50-200 entities total.
- Scenario count: 39+ RON files (as of 2026-03-22, Wave 3 audit); 15+ are stress scenarios. Exact count grows each wave — grep scenarios/ directory to confirm.
- Phase 4 Wave 4 is active. Wave 5+ (evolution system, run-end enhancements) upcoming.

**Why:** Scale determines whether concerns are real. With 1 bolt and ~50 cells per node, most hot-path issues are academic until Phase 3+ with upgrades multiplying bolt count.

**How to apply:** Flag Critical only if it causes hitches at current scale. Flag Moderate if it would hurt at 200+ entities or full upgrade system. Flag Minor for everything else.
