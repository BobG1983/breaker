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
- InterpolatePlugin registered BEFORE PhysicsPlugin in game.rs
- Interpolation pipeline: `restore_authoritative` (FixedFirst) → [FixedUpdate physics] → `store_authoritative` (FixedPostUpdate) → `interpolate_transform` (PostUpdate)
- Bolt entities (baseline + ExtraBolt) carry InterpolateTransform + PhysicsTranslation; bolt_lost inserts PhysicsTranslation on respawn to snap interpolation
- Physics chain: `prepare_bolt_velocity` → `bolt_cell_collision` → `bolt_breaker_collision` (BreakerCollision set) → `apply_bump_velocity` + `spawn_additional_bolt` → `bolt_lost` (BoltLost set)
- apply_bump_velocity: `.after(BreakerCollision).before(BoltLost)` — correctly ordered
- spawn_additional_bolt: `.after(BreakerCollision)` — Commands-only, no direct conflict with apply_bump_velocity
- ExtraBolt: despawned permanently when lost (not respawned); still sends BoltLost message
- New observer chain: bridge_bump/bridge_bolt_lost → SpawnBoltRequested observer → SpawnAdditionalBolt message; TimePenaltyRequested observer → ApplyTimePenalty message
- apply_time_penalty: `.after(NodeSystems::TickTimer)` — can also send TimerExpired when penalty drives timer to zero
- handle_run_lost: `.after(handle_node_cleared).after(handle_timer_expired)` — FIXED, win takes priority
- Breaker state chain: `update_bump` → `move_breaker` (BreakerSystems::Move) → `update_breaker_state` → `grade_bump` → post-grade systems
- Input: `read_input_actions` in PreUpdate (after InputSystems) writes InputActions resource consumed by FixedUpdate gameplay systems
- NodePlugin OnEnter chain: `set_active_layout` → `spawn_cells_from_layout` (NodeSystems::Spawn) → `init_clear_remaining` → `init_node_timer` (all chained)

## Active Conflicts (Action Required)
None. All previously-flagged ordering gaps are resolved.

## Low-Severity Issues (Optional Fixes)
1. `animate_bump_visual` and `animate_tilt_visual` both write `Transform` on Breaker in Update. Different fields (translation vs rotation), no logical conflict.
2. `handle_cell_hit` and `track_node_completion` are unordered across plugins. 1-tick delay on cell-to-completion counting is imperceptible.
3. `apply_time_penalty` and `handle_timer_expired` are unordered. 1-tick delay on penalty-induced timer expiry. Cross-plugin ordering would break encapsulation — acceptable trade-off.

## Still Stub (No Systems)
AudioPlugin, UpgradesPlugin — registered but contain no game systems

## Orphan Messages
- `UpgradeSelected` (UiPlugin) — no sender or receiver. Expected for future phases.
