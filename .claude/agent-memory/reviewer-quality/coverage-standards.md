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
- **Scenario invariants**: Very high. All checkers have happy-path, sad-path, and edge-case tests including physics_frozen_during_pause.
- **Scenario runner**: `evaluate_pass` has 0 of 3 branches tested. `scenario_health_warnings` has 1 of 5. Known gap as of 2026-03-17.
- **bolt/queries.rs**: Module doc misleading (says "clippy type_complexity lint" when real reason is convention). Flag if seen.
- **update_timer_display.rs**: `total == 0.0` divide-by-zero path untested. Flag this gap.
- **read_input.rs**: `repeat: true` key event filter path untested.

## Lifecycle Tests (2026-03-17)
Cover all major public systems: tick_scenario_frame, check_frame_limit, apply_debug_setup, enforce_frozen_positions, tag_game_entities, inject_scenario_input, init_scenario_input, ScenarioStats increments.

## Documentation Conventions
- Module-level `//!` doc comments on all `.rs` files.
- Public types/functions have `///` doc comments with field-level docs.
- Private helpers often lack doc comments (accepted when name is self-describing).
- Units documented inline on component fields (e.g., "world units per second").
- `#[must_use]` applied to pure query methods and value-returning helpers.
