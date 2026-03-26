---
name: message-flow-for-run-stats
description: Flow map for the six gameplay messages RunStats (4i) must observe, plus RunState, NodeTimer, GameRng/RunSeed, and run-end screen
type: project
---

## Messages the Run Stats system must observe

| Message | Defined in | Sender system | Schedule | Consumers |
|---------|-----------|---------------|----------|-----------|
| CellDestroyed | cells/messages.rs | handle_cell_hit (cells) | FixedUpdate, PlayingState::Active | track_node_completion, bridge_cell_destroyed |
| BumpPerformed | breaker/messages.rs | update_bump, grade_bump (breaker) | FixedUpdate, PlayingState::Active | spawn_bump_grade_text, bridge_bump, perfect_bump_dash_cancel |
| BoltLost | bolt/messages.rs | bolt_lost (bolt) | FixedUpdate, PlayingState::Active | spawn_bolt_lost_text, bridge_bolt_lost |
| ChipSelected | ui/messages.rs | handle_chip_input (screen/chip_select) | Update, during ChipSelect | apply_chip_effect (chips) |
| NodeCleared | run/node/messages.rs | track_node_completion (run/node) | FixedUpdate, PlayingState::Active | handle_node_cleared (run) |
| TimerExpired | run/node/messages.rs | tick_node_timer, apply_time_penalty (run/node) | FixedUpdate, PlayingState::Active | handle_timer_expired (run) |

## Key resources

- RunState: { node_index: u32, outcome: RunOutcome, transition_queued: bool } -- reset in reset_run_state on OnExit(MainMenu)
- NodeTimer: { remaining: f32, total: f32 } -- init per node in init_node_timer on OnEnter(Playing)
- GameRng: wraps ChaCha8Rng -- reseeded in reset_run_state from RunSeed
- RunSeed: Option<u64> -- set by handle_run_setup_input from SeedEntry text field

## Run-end screen

- RunEndPlugin registered at OnEnter(GameState::RunEnd)
- spawn_run_end_screen reads RunState.outcome, shows title+subtitle only (no stats currently)
- handle_run_end_input transitions to MainMenu on MenuConfirm
- Cleanup on OnExit(RunEnd)
