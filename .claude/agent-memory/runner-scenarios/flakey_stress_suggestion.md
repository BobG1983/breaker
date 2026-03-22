---
name: Stress Scenario Usage
description: When and how to use stress scenarios to exercise game mechanics under parallel load
type: pattern
---

# Stress Scenarios

Stress scenarios exercise game mechanics under parallel contention to find real game bugs
that only manifest under CPU/IO load (race conditions, timing-sensitive physics, etc.).

## When to suggest creating a stress scenario

When a game mechanic is sensitive to timing or parallelism — NOT to work around scenario
runner bugs. If a scenario fails in parallel but passes individually, the scenario runner
has a bug that needs fixing (ordering, gating, timing assumption).

Good reasons for stress scenarios:
- A physics system that behaves differently under frame-time variance
- A state machine with tight timing windows
- Entity spawn/despawn patterns that could leak under load

## Stress RON format

```ron
stress: Some(()),              // 32 runs, 32 parallelism (defaults)
stress: Some((runs: 64)),      // 64 runs, 32 parallelism
stress: Some((runs: 64, parallelism: 16)),  // explicit both
```

When `stress` is `Some(...)`, `cargo scenario -- --all` spawns multiple copies and
aggregates results. A stress scenario passes only if ALL copies pass.
