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
- `check_no_entity_leaks` samples at frame 60, checks at every multiple of 120 after that (threshold: base * 2). First check at frame 120.
- `tag_game_entities` called both OnEnter(Playing) and every FixedUpdate tick — handles mid-game Prism extra bolt spawns. Without<Tag> filter prevents re-tagging.
- `enforce_frozen_positions` resets entity to ScenarioPhysicsFrozen.target every FixedUpdate tick — runs after physics. Correct.
- `bypass_menu_to_playing` goes MainMenu → Playing directly — not forbidden by `check_valid_state_transitions`.
- New mechanic scenarios (aegis_dash_wall, aegis_pause_stress, aegis_state_machine, aegis_speed_bounce, aegis_lives_exhaustion) and stress scenarios (aegis_multinode, prism_bolt_stabilization, prism_concurrent_hits, chrono_clear_race, chrono_penalty_stress) use standard invariant sets.
- `check_physics_frozen_during_pause` stores position every tick (active and paused), violations fire only when paused and bolt moved since last tick.
