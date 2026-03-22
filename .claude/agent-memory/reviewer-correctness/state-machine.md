---
name: State Machine & Physics Notes
description: Valid state transitions, math/physics correctness notes
type: reference
---

## State Machine Rules
- Valid transitions: Loading→MainMenu, MainMenu→RunSetup, RunSetup→Playing, Playing→TransitionOut→ChipSelect→TransitionIn→Playing, Playing→RunEnd, RunEnd→MainMenu
- Pause sub-machine: Playing(Active)↔Playing(Paused), Paused+Quit→MainMenu
- `advance_node` runs OnEnter(GameState::TransitionIn) and immediately sets NextState(Playing) — 1-frame intermediate (was OnEnter(NodeTransition); NodeTransition state removed in Wave 3)
- `reset_run_state` runs OnExit(MainMenu) — resets node_index and outcome
- `handle_timer_expired` guards on RunOutcome::InProgress
- `handle_node_cleared` routes non-final nodes to TransitionOut (not directly to ChipSelect; FxPlugin drives TransitionOut→ChipSelect animation)
- `CleanupOnNodeExit` fires on OnExit(GameState::Playing) — fires on Playing→TransitionOut too

## Math/Physics Notes
- `bolt_lost` angle: angle-from-vertical convention. Speed preserved (sin²+cos²=1).
- `enforce_min_angle` uses `atan2(|y|, |x|)` — always [0, π/2]
- `reflect_top_hit`: clamped to [-max_angle, max_angle] — tilt can be fully cancelled
- CCD `remaining -= advance` (not advance + epsilon) — prevents sticking
- `bolt_breaker_collision` upward guard before face-type check — upward side hits not reflected
- `inject_scenario_input` passes `is_active: true` always — intentional for pause-toggle testing
- `toggle_pause` changed from ButtonInput to InputActions::TogglePause — correct
- `apply_time_penalty` only subtracts — TimerMonotonicallyDecreasing invariant valid
