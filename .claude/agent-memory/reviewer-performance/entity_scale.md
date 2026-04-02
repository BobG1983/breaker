---
name: Entity scale expectations
description: Per-type entity count expectations for severity calibration
type: project
---

As of Phase 5 (chip evolution ecosystem):

- **Breaker**: always 1 (single player entity)
- **Bolt**: 1 primary + a few extras at most (ExtraBolt chips)
- **Cells**: ~50–200 fixed grid; never changes at runtime within a node
- **Walls**: 4 fixed walls
- **Effect entities** (gravity wells, etc.): a handful at most, short-lived

**Why:** Most "fragmentation" or "query efficiency" concerns are academic until entity counts grow significantly beyond these numbers.

**How to apply:** Never flag Critical for issues affecting single-entity queries (Breaker, Bolt). Moderate threshold is ~200+ entities.
