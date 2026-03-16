# System Dependency Mapper Memory

## Bevy Version
**0.18.1** — uses `#[derive(Message)]`, `MessageWriter<T>`, `MessageReader<T>` for inter-system comms. Messages persist across frames (not drained per-tick), so cross-system message delays of one tick are safe and normal.

## Current Files
- [system-map.md](system-map.md) — Full system inventory: every system, plugin, schedule, ordering, data access
- [message-flow.md](message-flow.md) — Message flow map: who sends what, who receives what, cross-plugin boundaries
- [known-conflicts.md](known-conflicts.md) — Identified conflicts, ordering issues, missing constraints

## Key Architectural Facts
- All gameplay systems run in FixedUpdate (physics, breaker, bolt, cells, run) gated by `run_if(in_state(PlayingState::Active))`
- Visual-only systems run in Update (animate_bump_visual, animate_tilt_visual, update_timer_display, debug overlays, update_lives_display, animate_fade_out)
- Physics chain: `prepare_bolt_velocity` → `bolt_cell_collision` → `bolt_breaker_collision` (BreakerCollision set) → `apply_bump_velocity` → `bolt_lost` (BoltLost set)
- apply_bump_velocity: `.after(BreakerCollision).before(BoltLost)` — correctly ordered, conflict resolved
- Breaker state chain: `update_bump` → `move_breaker` (BreakerSystems::Move) → `update_breaker_state` → `grade_bump` → post-grade systems
- BumpPerformed multiplier embedded in message — grade_bump/update_bump use BumpGradingQuery/BumpTimingQuery (Optional multipliers on Breaker); apply_bump_velocity reads only bolt components
- RunPlugin ordering: `track_node_completion` (NodeSystems::TrackCompletion) → `handle_node_cleared`; `tick_node_timer` (NodeSystems::TickTimer) → `handle_timer_expired` (also .after(handle_node_cleared)); same-tick propagation
- RunLost message: sent by `handle_life_lost` observer (BehaviorPlugin) via bridge_bolt_lost → received by `handle_run_lost` (RunPlugin FixedUpdate)
- FxPlugin: owns `animate_fade_out` — moved from BoltPlugin. Now a first-class plugin in game.rs
- Input: `read_input_actions` in PreUpdate (after InputSystems) writes InputActions resource consumed by FixedUpdate gameplay systems
- NodePlugin OnEnter chain: `set_active_layout` → `spawn_cells_from_layout` (NodeSystems::Spawn) → `init_clear_remaining` → `init_node_timer` (all chained)

## Active Conflicts (Action Required)
1. `handle_run_lost` (RunPlugin, FixedUpdate) is UNORDERED vs `handle_node_cleared` and `handle_timer_expired`. All three write ResMut<RunState> + ResMut<NextState<GameState>>. Race: if RunLost and NodeCleared arrive same tick, order is non-deterministic. Fix: add `.after(handle_node_cleared).after(handle_timer_expired)` to `handle_run_lost` in `run/plugin.rs`.

## Low-Severity Issues (Optional Fixes)
2. `animate_bump_visual` and `animate_tilt_visual` both write `Transform` on Breaker in Update. Different fields (translation vs rotation), no logical conflict.
3. `handle_cell_hit` and `track_node_completion` are unordered across plugins. 1-tick delay on cell-to-completion counting is imperceptible.

## Still Stub (No Systems)
AudioPlugin, UpgradesPlugin — registered but contain no game systems

## Orphan Messages
- `UpgradeSelected` (UiPlugin) — no sender or receiver. Expected for future phases.
