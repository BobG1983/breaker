---
name: Pause Quit Fix Coverage Map
description: Coverage gaps for the quit-from-pause state machine fix (NodeResult::Quit → RunState::Teardown via NodeState::Teardown)
type: project
---

# Pause Quit Fix Coverage Map

## What the Fix Does

`handle_pause_input` now sets `NodeOutcome.result = NodeResult::Quit` and writes `ChangeState<NodeState>`.
The node state machine routes: `Playing → AnimateOut → NodeState::Teardown`.
`resolve_node_next_state` maps `NodeResult::Quit → RunState::Teardown` (not ChipSelect or RunEnd).
`register_cleanup` adds a safety net: `OnEnter(RunState::Teardown)` also runs `cleanup_on_exit::<NodeState>`.

Previously the quit bypassed teardown; now it goes through it.

## Key Architectural Finding: Pause Input is NOT Injectable

`toggle_pause` reads `ButtonInput<KeyCode>` directly (not `InputActions`).
`handle_pause_input` also reads `ButtonInput<KeyCode>` directly.
The scenario runner injects into `InputActions` only.

Result: `TogglePause`, `MenuDown`, and `MenuConfirm` in `GameAction` do NOT drive the pause systems.
The `aegis_pause_stress` scenario injects `TogglePause` into `InputActions` but this never reaches
`toggle_pause` or `handle_pause_input`. The scenario exercises `InputActions` injection, not actual pausing.

## Unit Test Coverage (good)

- `handle_pause_input.rs` tests: confirm_quit_writes_change_state_message, confirm_quit_sets_node_outcome_to_quit, confirm_quit_unpauses_time, confirm_quit_overrides_prior_timer_expired_result
- `plugin.rs` tests: resolve_node_next_state_quit_returns_teardown, resolve_node_next_state_quit_ignores_node_index_and_transition_queued, cleanup_on_node_exit_runs_on_enter_run_state_teardown

## Scenario Coverage: NONE for the actual quit-from-pause path

No scenario exercises the end-to-end path:
1. Pause the game (Time<Virtual>::pause)
2. Navigate pause menu to Quit
3. NodeState transitions through AnimateOut → Teardown
4. RunState routes to Teardown (not ChipSelect, not RunEnd)
5. cleanup_on_exit:<NodeState> fires on RunState::Teardown entry

The `aegis_pause_stress` scenario only verifies that rapid TogglePause injection doesn't crash
(via BoltInBounds, BreakerInBounds, NoNaN). It does NOT test actual pause-then-quit.

## Runner Capability Gap

The scenario runner cannot drive `toggle_pause` or `handle_pause_input` because these systems
read `ButtonInput<KeyCode>` directly, not `InputActions`. To test quit-from-pause end-to-end
in a scenario, the runner would need either:
(a) A `QuitToMenu` scripted action that directly sets NodeOutcome.result = Quit and writes ChangeState, OR
(b) The pause systems to consume InputActions instead of ButtonInput<KeyCode>

## What IS Covered

- Unit tests verify each piece of the quit routing in isolation
- `aegis_pause_stress` verifies no crash under rapid TogglePause injection (not actual pausing)
- `entity_leak` self-test verifies NoEntityLeaks fires; would catch entity leaks after quit IF the quit path were exercised

## Why: recorded 2026-04-06 after reviewing feature/effect-placeholder-visuals branch audit request
