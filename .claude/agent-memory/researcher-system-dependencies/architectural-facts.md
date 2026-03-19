---
name: Architectural Facts
description: Key architectural facts, hot-reload pipeline, scenario runner systems, stubs, orphan messages
type: reference
---

## Key Architectural Facts
- All gameplay systems run in FixedUpdate gated by `run_if(in_state(PlayingState::Active))`
- EXCEPTION: `check_spawn_complete` (NodePlugin, FixedUpdate) has NO run_if guard — it must fire in the first tick of Playing before Active is set
- Visual-only systems run in Update (animate_bump_visual, animate_tilt_visual, update_timer_display, debug overlays, update_lives_display, animate_fade_out)
- InterpolatePlugin registered BEFORE PhysicsPlugin in game.rs
- BehaviorsPlugin is STANDALONE, registered between BreakerPlugin and BoltPlugin
- Interpolation pipeline: `restore_authoritative` (FixedFirst) → [FixedUpdate physics] → `store_authoritative` (FixedPostUpdate) → `interpolate_transform` (PostUpdate)
- Bolt entities carry InterpolateTransform + PhysicsTranslation; bolt_lost inserts PhysicsTranslation on respawn
- Physics chain: `prepare_bolt_velocity` → `bolt_cell_collision` → `bolt_breaker_collision` → `clamp_bolt_to_playfield` → `bolt_lost`
- clamp_bolt_to_playfield: safety clamp for bolts escaping through CCD corner overlaps; no bottom clamp (bolt_lost handles that)
- apply_bump_velocity: `.after(BreakerCollision).before(BoltLost)`
- spawn_additional_bolt: `.after(BehaviorSystems::Bridge)`
- ExtraBolt: despawned permanently when lost (not respawned); still sends BoltLost message
- Behavior observer chain: bridge systems fire commands.trigger(ConsequenceFired) → observers run immediately
- apply_time_penalty: `NodeSystems::ApplyTimePenalty` set, `.after(NodeSystems::TickTimer)` — can also send TimerExpired
- handle_timer_expired: now `.after(NodeSystems::ApplyTimePenalty)` (was `.after(NodeSystems::TickTimer)`) — same-tick penalty-induced expiry guaranteed
- handle_run_lost: `.after(handle_node_cleared).after(handle_timer_expired)` — win takes priority
- Breaker state chain: `update_bump` → `move_breaker` → `update_breaker_state` → `grade_bump`
- Input: `read_input_actions` in PreUpdate writes InputActions consumed by FixedUpdate
- Scenario runner inject_scenario_input: moved to FixedPreUpdate (was in FixedUpdate chain)
- NodePlugin OnEnter chain: `set_active_layout` → `spawn_cells_from_layout` → `init_clear_remaining` → `init_node_timer`
- UiPlugin OnEnter chain: `spawn_side_panels` → ApplyDeferred → `spawn_timer_hud` (in_set(UiSystems::SpawnTimerHud))
- Spawn coordinator: `check_spawn_complete` waits for BoltSpawned+BreakerSpawned+CellsSpawned+WallsSpawned → sends SpawnNodeComplete; consumed by check_no_entity_leaks for baseline sampling
- Scenario runner invariant checkers: `.after(tag_game_entities).after(update_breaker_state).before(PhysicsSystems::BoltLost)`
- toggle_pause reads InputActions/GameAction::TogglePause
- bolt_breaker_collision: upward-bolt guard at top of bolt loop
- BoltHitCell message no longer carries bolt Entity field
- DebugOverlays: bool fields replaced by enum-indexed array
- bolt/queries.rs: BoltLostQuery type alias
- ChaosMonkey includes TogglePause in GAMEPLAY_ACTIONS
- RecordingPlugin (debug, cfg(feature="dev")): `capture_frame` (FixedUpdate, reads InputActions), `write_recording_on_exit` (Last, triggers on AppExit message)

## Hot-Reload Pipeline (HotReloadPlugin, Update, GameState::Playing)
- Set 1 `PropagateDefaults` (11 systems): asset Modified event → Config resource or registry
- Set 2 `PropagateConfig` (2 systems): `.after(PropagateDefaults)`, gated by `resource_changed::<T>`
- Breaker path: direct `ResMut<BreakerConfig>` write → same-frame propagation
- Bolt/cell/etc.: `commands.insert_resource` → next-frame propagation
- `propagate_archetype_changes` also writes `ResMut<BreakerConfig>` and `ResMut<ActiveBehaviors>`
- `propagate_breaker_defaults` and `propagate_archetype_changes` both hold `ResMut<BreakerConfig>` — Bevy serializes, no race

## Scenario Runner (breaker-scenario-runner)
- 15 systems in FixedUpdate (lifecycle chain + 12 invariant checkers + enforce_frozen_positions + tag_game_entities), 1 OnEnter group
- Lifecycle chain: `tick_scenario_frame → inject_scenario_input → check_frame_limit` .chain() .before(BreakerSystems::Move)
- 12 invariant checkers (unordered, all read-only on game world): check_bolt_in_bounds, check_bolt_speed_in_range, check_bolt_count_reasonable, check_breaker_in_bounds, check_no_nan, check_timer_non_negative, check_valid_state_transitions, check_valid_breaker_state, check_timer_monotonically_decreasing, check_breaker_position_clamped, check_physics_frozen_during_pause, check_no_entity_leaks
- 2 mutators: enforce_frozen_positions (writes &mut Transform on ScenarioPhysicsFrozen entities), tag_game_entities (Commands insert marker components)
- OnEnter chain: `init_scenario_input → tag_game_entities → apply_debug_setup` .chain() .after(init_bolt_params)
- InputStrategy: Chaos, Scripted, Hybrid
- ScenarioStats: tracks actions_injected, invariant_checks, max_frame, entered_playing, bolts_tagged, breakers_tagged
- ScenarioPhysicsFrozen: component holding frozen Vec3 target — entity Transform pinned each tick by enforce_frozen_positions
- DebugSetup: RON field, supports bolt_position, breaker_position, disable_physics
- InvariantParams: RON field, supports max_bolt_count (default 8)
- check_valid_state_transitions uses ResMut<PreviousGameState> (not Local) — stored in world, survives ticks
- check_valid_breaker_state uses Local<Option<BreakerState>> — not in world, per-system state
- All new invariant checkers imported from breaker:: — uses pub bolt, breaker, run modules (lib.rs visibility change)
- 12 InvariantKind variants: BoltInBounds, BoltSpeedInRange, BoltCountReasonable, BreakerInBounds, NoEntityLeaks, NoNaN, TimerNonNegative, ValidStateTransitions, ValidBreakerState, TimerMonotonicallyDecreasing, BreakerPositionClamped, PhysicsFrozenDuringPause
- New scenario categories: mechanic/ (11 scenarios), stress/ (13 scenarios), self_tests/ (3 scenarios)

## Still Stub (No Systems)
AudioPlugin

## Orphan Messages
- None at current phase. `ChipSelected` (UiPlugin) is now received by `chips/apply_chip_effect`.

## New in 2026-03-19 (post-spawn-coordinator work)
- `BoltSpawned` — BoltPlugin, sent by `spawn_bolt`
- `BreakerSpawned` — BreakerPlugin, sent by `spawn_breaker` (even when no-op, i.e. breaker already exists)
- `CellsSpawned` — NodePlugin, sent by `spawn_cells_from_layout`
- `WallsSpawned` — WallPlugin, sent by `spawn_walls`
- `SpawnNodeComplete` — NodePlugin, sent by `check_spawn_complete` coordinator; consumed by scenario runner only
- `NodeSystems::ApplyTimePenalty` — new system set variant in NodeSystems enum
- `clamp_bolt_to_playfield` — new PhysicsPlugin system, safety clamp after bolt_breaker_collision
- `seed_chip_registry` — new LoadingPlugin system (seeds ChipRegistry from chip definition assets)
- `RecordingPlugin` with `capture_frame` + `write_recording_on_exit` — debug-only input recorder
