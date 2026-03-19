---
name: Flakey Stress Suggestion
description: When a scenario fails intermittently, suggest creating a stress RON file to exercise it under parallel contention
type: pattern
---

# Flakey / Intermittent Scenario Failure Detection

When a scenario fails intermittently — passes in isolation (`-s name`) but fails under parallel load (`--all`) — this is a sign of a concurrency or timing issue that should be exercised under stress.

## What to suggest

When you see this pattern, suggest to the orchestrator:

> This looks like a flakey/intermittent failure. Consider creating a stress scenario RON file at `scenarios/stress/<name>_stress.scenario.ron` with `stress: Some(())` to exercise this under parallel contention. Base it on the failing scenario's config.

## Stress RON format

The `stress` field on `ScenarioDefinition` controls automatic stress testing:

```ron
stress: Some(()),              // 32 runs, 32 parallelism (defaults)
stress: Some((runs: 64)),      // 64 runs, 32 parallelism
stress: Some((runs: 64, parallelism: 16)),  // explicit both
```

When `stress` is `Some(...)`, `cargo scenario -- --all` automatically spawns multiple copies of that scenario and aggregates results. A stress scenario passes only if ALL copies pass.

## Detection heuristics

- Scenario FAIL under `--all` but PASS when re-run with `-s name`
- Non-deterministic failures (different frame numbers, different invariant violations)
- Race conditions in entity tagging, physics interactions, or state transitions
- Failures that only appear under CPU contention (parallel subprocess load)
