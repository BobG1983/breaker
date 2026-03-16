# System Dependency Mapper Memory

## Bevy Version
**0.18.1** — uses `#[derive(Message)]`, `MessageWriter<T>`, `MessageReader<T>` for inter-system comms. Messages persist across frames (not drained per-tick), so cross-system message delays of one tick are safe and normal.

## Current Files
- [system-map.md](system-map.md) — Full system inventory: every system, plugin, schedule, ordering, data access
- [message-flow.md](message-flow.md) — Message flow map: who sends what, who receives what, cross-plugin boundaries
- [known-conflicts.md](known-conflicts.md) — Identified conflicts, ordering issues, missing constraints

## Key Architectural Facts
- All gameplay systems run in FixedUpdate (physics, breaker, bolt, cells, run) gated by `run_if(in_state(PlayingState::Active))`
- Visual-only systems run in Update (animate_bump_visual, animate_tilt_visual, update_timer_display, debug overlays)
- Physics chain: `prepare_bolt_velocity` → `bolt_cell_collision` → `bolt_breaker_collision` → `bolt_lost`
- Breaker state chain: `update_bump` → `move_breaker` (BreakerSystems::Move) → `update_breaker_state` → `grade_bump` → `perfect_bump_dash_cancel`
- Bolt systems after breaker move: `hover_bolt`, `prepare_bolt_velocity` both `.after(BreakerSystems::Move)`
- Physics after bolt: all three collision systems `.after(BoltSystems::PrepareVelocity)`
- Input: `read_input_actions` in PreUpdate (after InputSystems) writes InputActions resource consumed by FixedUpdate gameplay systems
- RunPlugin OnEnter chain: `set_active_layout` → `spawn_cells_from_layout` (NodeSystems::Spawn) → `init_clear_remaining` → `init_node_timer` (all chained)

## Active Conflicts (Action Required)
1. `apply_bump_velocity` (BoltPlugin, FixedUpdate) writes `BoltVelocity` but has NO ordering vs `bolt_lost`. Both are `.after(BreakerCollision)`. Fix: add `.after(bolt_lost)` to `apply_bump_velocity`.

## Low-Severity Issues (Optional Fixes)
2. RunPlugin FixedUpdate group (`track_node_completion`, `handle_node_cleared`, `tick_node_timer`, `handle_timer_expired`) has no ordering. Message persistence makes 1-tick delay safe. Optional: `.chain()` pairs for clarity.
3. `handle_cell_hit` and `track_node_completion` are unordered across plugins. 1-tick delay on cell-to-completion counting is imperceptible.
4. `animate_bump_visual` and `animate_tilt_visual` both write `Transform` on Breaker in Update. Different fields (translation vs rotation), no logical conflict.

## Previously Unimplemented, Now Active
- RunPlugin: fully implemented (track_node_completion, handle_node_cleared, tick_node_timer, handle_timer_expired, advance_node, reset_run_state, plus OnEnter setup chain)
- UiPlugin: now has systems (spawn_side_panels, spawn_timer_hud, update_timer_display)
- CellDestroyed: now consumed by RunPlugin::track_node_completion (was orphan previously)
- NodeCleared: now sent (track_node_completion) and consumed (handle_node_cleared)
- TimerExpired: now sent (tick_node_timer) and consumed (handle_timer_expired)

## Still Stub (No Systems)
AudioPlugin, UpgradesPlugin — registered but contain no game systems

## Remaining Orphan Messages
- `UpgradeSelected` (UiPlugin) — no sender or receiver. Expected for future phases.
