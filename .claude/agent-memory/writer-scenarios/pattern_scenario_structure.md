---
name: Scenario RON structure and field conventions
description: Required and optional fields in ScenarioDefinition, stress config, and naming conventions
type: reference
---

## Scenario file location

All scenarios live under `breaker-scenario-runner/scenarios/`:
- `mechanic/` — single-run scenarios testing specific mechanical behavior
- `stress/` — multi-run scenarios with `stress: Some(...)` config
- `self_tests/` — scenarios with `expected_violations` that intentionally trigger invariants
- `regressions/` — regression scenarios for previously-fixed bugs

## Required fields

```ron
(
    breaker: "Aegis",     // "Aegis", "Prism", "Chrono" — must be a valid breaker name
    layout: "Corridor",   // "Corridor", "Dense", "Scatter", "Fortress", "BossArena", "Gauntlet"
    input: Chaos((action_prob: 0.3)),
    max_frames: 5000,
    disallowed_failures: [BoltInBounds, NoEntityLeaks, NoNaN],
    allowed_failures: None,
    debug_setup: None,
)
```

## Common optional fields

```ron
allow_early_end: false,          // Default true; false = run continues past RunEnd for full max_frames
seed: 42,                        // RNG seed; 0 if omitted
stress: (runs: 32, parallelism: 32),  // Stress config; omit for single-run
initial_effects: [...],          // Effect chains injected before play starts
invariant_params: (max_bolt_count: 16),  // Defaults: max_bolt_count=8
```

Note: `disallowed_failures` replaces the old `invariants:` field; `allowed_failures` replaces `expected_violations:`. RON field names changed in the invariant rename refactor (2026-04-02).

## Stress conventions

- Stress scenarios: `allow_early_end: false` so the node doesn't end early
- Stress scenarios using `BoltCountReasonable`: raise `max_bolt_count` to match expected spawn count
- `stress: (runs: 32, parallelism: 32)` is the standard 32-run stress config
- `stress: (runs: 16, parallelism: 16)` for lighter stress (lower bolt/entity overhead)

## Seed conventions

- Mechanic scenarios: small primes or memorable numbers (1111, 3141, 500)
- Stress scenarios: larger values with varied patterns (2222, 7777, 8080, 9999)
- Different scenarios should use different seeds to explore diverse paths

## Adversarial comment header

Every scenario should have a comment header explaining:
1. What adversarial condition it sets up
2. Why that condition stresses the system
3. Which invariant fires and why
4. Stress run rationale (if stress)

The comment precedes the `(` opener with no blank line between.
