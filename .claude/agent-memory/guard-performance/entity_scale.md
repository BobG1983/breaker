---
name: entity_scale
description: Entity count expectations per phase, for calibrating severity ratings
type: project
---

Derived from CLAUDE.md, docs/architecture, and code review (as of 2026-03-19).

- Bolt entities: 1 baseline + small number of ExtraBolts via upgrades; design maximum is ~8-16 active at once
- Cell entities: fixed grid per node; current layouts are small (estimated ~50 cells per node)
- Breaker: always exactly 1
- Physics domain: FixedUpdate, bolt count bounded at all phases

**Why:** Scale determines whether "theoretical" concerns are real. With 1 bolt and ~50 cells per node, most hot-path issues are academic until Phase 3+ with upgrades multiplying bolt count.

**How to apply:** Flag Critical only if it causes hitches at current scale (single bolt, ~50 cells). Flag Moderate if it would hurt at 200+ entities or full upgrade system. Flag Minor for everything else.
