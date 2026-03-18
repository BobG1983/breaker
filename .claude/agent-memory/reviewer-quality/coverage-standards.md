---
name: Coverage Standards & Doc Conventions
description: Test coverage standards by domain and documentation conventions
type: reference
---

## Test Coverage Standards by Domain

- **Bump system**: Very high. Grade functions, update_bump, grade_bump, combined pipeline, BoltServing guard, input-loss regression, perfect_bump_dash_cancel.
- **CCD physics**: Comprehensive. All collision surfaces, multi-bolt, cascade prevention, MAX_BOUNCES cap, wall vs cell, overlap resolution.
- **Breaker state machine**: All 4 transitions, easing correctness, frame-rate independence, timer init.
- **Node completion**: All branches (required, non-required, zero, remaining).
- **Run end paths**: All three outcomes (node-transition, win, no-op).
- **Scenario invariants (as of 2026-03-18, feature/scenario-runner-dedup-summary)**: Very high. All checkers have happy-path, sad-path, and edge-case tests including physics_frozen_during_pause. Known gaps: (1) `check_bolt_in_bounds` does not test that `ScenarioStats::invariant_checks` is incremented by the number of bolts checked, even though the doc promises it. (2) `Settling → Dashing` legal transition in `check_valid_breaker_state` has no test (only `Idle → Dashing` is tested). (3) Several test doc comments contain stale TDD-phase language ("This test MUST FAIL until the fix lands") — the fixes have landed; these should be removed. (4) Two pairs of duplicate tests: `valid_breaker_state_fires_on_idle_to_braking` / `check_valid_breaker_state_illegal_idle_to_braking_produces_violation`, and `valid_breaker_state_does_not_fire_on_idle_to_dashing` / `check_valid_breaker_state_legal_idle_to_dashing_produces_no_violation`.
- **ScenarioVerdict (verdict.rs)**: Very high as of 2026-03-18. All 6 health checks tested (no_actions_injected, never_entered_playing, no_bolts_tagged, no_breakers_tagged, early_exit, no_invariant_checks). All 3 expected_violations branches tested (None/no-violations → Pass, Some([]) → Pass, expected matches → Pass, expected not fired → Fail, unexpected violation → Fail). Logs cause failure. add_fail_reason semantics (appends, keeps Fail, reverts Pass to Fail). Default verdict is Fail with sentinel reason. Known gap: `collect_and_evaluate` in runner.rs has no unit test for the missing-resource path (the 4-way match arm that adds individual resource-missing reasons).
- **runner.rs grouping helpers (as of 2026-03-18, feature/scenario-runner-dedup-summary)**: group_violations fully covered (same-kind grouping, different-kind separation, single entry). group_logs now covered including same-message-different-levels separation. is_invariant_fail_reason now tested exhaustively against InvariantKind::ALL and health-check strings. is_health_check_reason has no direct unit test (tested indirectly via is_invariant_fail_reason). collect_and_evaluate missing-resource path (4-way match arm) still untested. scenario_log_plugin() helper is private and not directly tested (acceptable — it's simple config construction).
- **game.rs (as of 2026-03-18, feature/scenario-runner-dedup-summary)**: `game_plugin_group_builds` tests the non-headless path. `headless_game_spawns_no_camera` tests the headless path (no Camera2d spawned). `HeadlessAssetsPlugin` is private — it is implicitly exercised by `headless_game_spawns_no_camera` (the app.update() would fail asset init if the plugin were broken). No standalone `headless_assets_plugin_builds` test exists; acceptable for a private plugin covered transitively.
- **bolt/queries.rs**: Module doc misleading (says "clippy type_complexity lint" when real reason is convention). Flag if seen.
- **update_timer_display.rs**: `total == 0.0` divide-by-zero path untested.
- **read_input.rs**: `repeat: true` key event filter path untested.

## Lifecycle Tests (2026-03-17, updated 2026-03-18)
Cover all major public systems: tick_scenario_frame, check_frame_limit, apply_debug_setup, enforce_frozen_positions, tag_game_entities, inject_scenario_input, init_scenario_input, ScenarioStats increments.
Known gap (as of 2026-03-18): `allow_early_end: false` path (`restart_run_on_end` system) has no unit test. The branch is covered only by the 13 stress scenario RON files running in integration. A unit test should exercise the `OnEnter(GameState::RunEnd)` → `GameState::MainMenu` transition directly.

## Documentation Conventions
- Module-level `//!` doc comments on all `.rs` files.
- Public types/functions have `///` doc comments with field-level docs.
- Private helpers often lack doc comments (accepted when name is self-describing).
- Units documented inline on component fields (e.g., "world units per second").
- `#[must_use]` applied to pure query methods and value-returning helpers.
