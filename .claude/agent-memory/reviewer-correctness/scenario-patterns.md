---
name: Scenario Runner Patterns
description: Confirmed correct patterns in the scenario runner — do not re-flag
type: reference
---

## Scenario Runner Patterns (feature/scenario-coverage-expansion)
- `check_bolt_in_bounds` only checks `y < bottom` — top/left/right have walls.
- `check_timer_monotonically_decreasing` resets Local to None when NodeTimer absent — correct.
- `check_physics_frozen_during_pause` clears HashMap on OnExit(Playing) — entity ID recycling not a concern.
- `init_scenario_input` resets InputDriver from scratch on each OnEnter(Playing) — intentional fresh chaos per node.
- `inject_scenario_input` hardcodes `is_active: true` — enables TogglePause injection while paused.
- `ScriptedInput::actions_for_frame` uses linear find() — O(n) fine for ≤200 entries.
- `HybridInput` scripted phase returns empty vec without advancing chaos RNG — correct.
- `evaluate_pass` gates on `logs.is_empty()` even with `expected_violations` — logs are unexpected warnings/errors.
- New mechanic and stress scenarios use standard invariant sets. Bolt count limits: 8 (stabilization), 12 (concurrent hits).
- `ScenarioDefinition.invariants` field is documentation-only — all invariant systems run unconditionally.
- `check_valid_breaker_state` legal transitions include `Dashing → Settling` — overly permissive but not a game bug.
