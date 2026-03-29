---
name: RON format conventions
description: Scenario RON field order, syntax quirks, and conventions observed from existing scenarios
type: reference
---

## Standard field order (follow this exactly)

```ron
(
    breaker: "Aegis",
    layout: "Dense",
    input: Chaos((action_prob: 0.6)),
    max_frames: 8000,
    invariants: [NoEntityLeaks, NoNaN, BoltInBounds],
    expected_violations: None,
    debug_setup: None,
    invariant_params: (max_bolt_count: 12),   // only include if overriding default of 8
    allow_early_end: false,
    stress: (runs: 16, parallelism: 16),       // only for stress scenarios
    seed: 1234,
    initial_effects: [...],
)
```

## What to omit

- Omit `invariant_params` unless overriding `max_bolt_count` from its default of 8.
- Omit `stress` for non-stress mechanic scenarios.
- Omit fields with `#[serde(default)]` when using their defaults (seed, initial_chips, frame_mutations, chip_selections, expected_offerings).
- `allow_early_end: false` is the standard for almost all effect scenarios — enables restart so max_frames is the actual test duration.

## Input strategy syntax

```ron
input: Chaos((action_prob: 0.6))     // random, moderate chaos
input: Perfect(AlwaysPerfect)        // guaranteed perfect bumps, good for PerfectBumped trigger scenarios
input: Chaos((action_prob: 0.9))     // extreme chaos for breaker edge tests
```

## Stress config syntax

```ron
stress: Some(())                     // 32 runs, 32 parallelism (both default to 32)
stress: (runs: 16, parallelism: 16)  // explicit — use this form
```

Note: stress scenarios use the non-`Some()` form in practice (see existing scenarios). The `Some()` form is the typed RON; in practice the runner accepts `(runs: 16, parallelism: 16)` directly because the field is `Option<StressConfig>` with RON's implicit unwrapping for structs.

## Available layouts (confirmed from existing scenarios)

- `"Dense"` — maximum cell density, maximises simultaneous impacts
- `"Scatter"` — irregular spacing, exercises varied gap widths
- `"Corridor"` — narrow vertical channel, maximises wall bounces
- `"Fortress"` — heavy cell armor, long run duration
- `"Gauntlet"` — linear progression layout

## Available breakers

- `"Aegis"` — standard breaker, most scenarios use this
- `"Chrono"` — timer-focused, required for NodeTimerThreshold scenarios
- `"Prism"` — multi-bolt breaker

## Scenario file locations

- `breaker-scenario-runner/scenarios/stress/` — stress scenarios (multiple runs)
- `breaker-scenario-runner/scenarios/mechanic/` — single-run mechanic scenarios
- `breaker-scenario-runner/scenarios/self_tests/` — invariant self-tests with expected_violations
- `breaker-scenario-runner/scenarios/regressions/` — regression scenarios

## Key gotcha: dscheck may show pre-existing errors

On the `feature/runtime-effects` branch, `cargo dscheck` may show compile errors in `breaker-scenario-runner` from in-progress `MutationKind` variants. These are NOT caused by scenario RON files. The release binary (`target/release/breaker_scenario_runner`) can still be used directly to validate RON parsing when dscheck fails to compile due to pre-existing errors.
