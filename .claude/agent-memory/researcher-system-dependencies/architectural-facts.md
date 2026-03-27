---
name: Architectural Facts
description: Key architectural facts, hot-reload pipeline, scenario runner systems, stubs, orphan messages
type: reference
---

## Key Architectural Facts
- All gameplay systems run in FixedUpdate gated by `run_if(in_state(PlayingState::Active))`
- EXCEPTION: `check_spawn_complete` (NodePlugin, FixedUpdate) has NO run_if guard — it must fire in the first tick of Playing before Active is set
- Visual-only systems run in Update (animate_bump_visual, animate_tilt_visual, update_timer_display, debug overlays, update_lives_display, animate_fade_out)
- EffectPlugin (renamed from BehaviorsPlugin in C7-R, 2026-03-25) is STANDALONE, registered between BreakerPlugin and BoltPlugin
- Spatial2d pipeline: `save_previous` (FixedFirst, SpatialSystems::SavePrevious) → [FixedUpdate gameplay/physics] → `compute_globals` (SpatialSystems::ComputeGlobals) → `derive_transform` (SpatialSystems::DeriveTransform) (AfterFixedMainLoop, chained in RantzSpatial2dPlugin). NOTE: propagate_position/rotation/scale are NOT registered by the plugin.
- SpatialSystems set: `SavePrevious`, `ApplyVelocity`, `ComputeGlobals`, `DeriveTransform` — exported from `rantzsoft_spatial2d::plugin` and prelude; use for cross-system ordering
- InterpolatePlugin and PhysicsPlugin DELETED (2026-03-24 spatial/physics extraction). Replaced by RantzSpatial2dPlugin and RantzPhysics2dPlugin.
- Bolt entities carry Position2D (canonical) + InterpolateTransform2D (for visual smoothing); bolt_lost no longer inserts PhysicsTranslation (that type is gone)
- Physics collision chain (bolt domain): `prepare_bolt_velocity` → `bolt_cell_collision` → `bolt_breaker_collision` → `clamp_bolt_to_playfield` → `bolt_lost`
- clamp_bolt_to_playfield: safety clamp for bolts escaping through CCD corner overlaps; no bottom clamp (bolt_lost handles that)
- apply_bump_velocity: DELETED (2026-03-21) — velocity scaling now via Effect::SpeedBoost { multiplier } leaf → handle_speed_boost observer in effect/effects/speed_boost.rs
- spawn_additional_bolt: `.after(EffectSystems::Bridge)`
- NOTE (2026-03-21): bolt/behaviors/ sub-domain DELETED. BoltBehaviorsPlugin REMOVED. behaviors/consequences/→behaviors/effects/. All bridge/effect logic unified in BehaviorsPlugin (since renamed EffectPlugin in C7-R, 2026-03-25).
- NOTE (2026-03-25, C7-R): behaviors/ domain → effect/ domain. BehaviorsPlugin→EffectPlugin. BehaviorSystems→EffectSystems. ActiveChains→ActiveEffects. ArmedTriggers→ArmedEffects. EffectFired (unified trigger) DELETED → replaced by per-effect typed events (ShockwaveFired, LoseLifeFired, etc.) in effect/typed_events.rs.
- ExtraBolt: despawned permanently when lost (not respawned); still sends BoltLost message
- Effect dispatch chain: bridge systems fire typed events (e.g., ShockwaveFired, LoseLifeFired) via fire_typed_event() → per-effect observers run immediately
- apply_time_penalty: `NodeSystems::ApplyTimePenalty` set, `.after(NodeSystems::TickTimer)` — can also send TimerExpired
- handle_timer_expired: now `.after(NodeSystems::ApplyTimePenalty)` (was `.after(NodeSystems::TickTimer)`) — same-tick penalty-induced expiry guaranteed
- handle_run_lost: `.after(handle_node_cleared).after(handle_timer_expired)` — win takes priority
- Breaker state chain: `update_bump` → `move_breaker` → `update_breaker_state` → `grade_bump`
- Input: `read_input_actions` in PreUpdate writes InputActions consumed by FixedUpdate
- Scenario runner inject_scenario_input: moved to FixedPreUpdate (was in FixedUpdate chain)
- NodePlugin OnEnter chain: `set_active_layout` → `spawn_cells_from_layout` → `init_clear_remaining` → `init_node_timer`
- UiPlugin OnEnter chain: `spawn_side_panels` → ApplyDeferred → `spawn_timer_hud` (in_set(UiSystems::SpawnTimerHud))
- Spawn coordinator: `check_spawn_complete` waits for BoltSpawned+BreakerSpawned+CellsSpawned+WallsSpawned → sends SpawnNodeComplete; consumed by check_no_entity_leaks for baseline sampling
- Scenario runner invariant checkers: `.after(tag_game_entities).after(update_breaker_state).before(BoltSystems::BoltLost)`
- toggle_pause reads InputActions/GameAction::TogglePause
- bolt_breaker_collision: upward-bolt guard at top of bolt loop
- BoltHitCell carries `{ cell: Entity, bolt: Entity }` — both fields present
- DebugOverlays: bool fields replaced by enum-indexed array
- bolt/queries.rs: BoltLostQuery type alias
- ChaosDriver includes TogglePause in GAMEPLAY_ACTIONS
- RecordingPlugin (debug, cfg(feature="dev")): `capture_frame` (FixedUpdate, reads InputActions), `write_recording_on_exit` (Last, triggers on AppExit message)

## Hot-Reload Pipeline (HotReloadPlugin, Update, GameState::Playing)
- Set 1 `PropagateDefaults` (3 systems): registry rebuilds on asset Modified event — `propagate_cell_type_changes`, `propagate_node_layout_changes`, `propagate_breaker_changes`. Simple config propagation (bolt, breaker, input, etc.) is now handled by `rantzsoft_defaults::systems::propagate_defaults` registered via `RantzDefaultsPlugin` — no longer in HotReloadPlugin.
- Set 2 `PropagateConfig` (2 systems): `.after(PropagateDefaults)`, gated by `resource_changed::<T>` — `propagate_bolt_config`, `propagate_breaker_config`
- Breaker path: direct `ResMut<BreakerConfig>` write → same-frame propagation
- `propagate_breaker_changes` writes `ResMut<BreakerConfig>` AND `Query<&mut EffectChains, With<Breaker>>`
- NOTE: `DefaultsCollection` DELETED (SeedableRegistry feature). The old 11-system PropagateDefaults count was before rantzsoft_defaults absorbed simple config propagation.

## Scenario Runner (breaker-scenario-runner)
- 17 systems in FixedUpdate (lifecycle chain + 14 invariant checkers + enforce_frozen_positions + tag_game_entities), 1 OnEnter group
- Lifecycle chain: `tick_scenario_frame → inject_scenario_input → check_frame_limit` .chain() .before(BreakerSystems::Move)
- 14 invariant checkers (unordered, all read-only on game world): check_bolt_in_bounds, check_bolt_speed_in_range, check_bolt_count_reasonable, check_breaker_in_bounds, check_no_nan, check_timer_non_negative, check_valid_state_transitions, check_valid_breaker_state, check_timer_monotonically_decreasing, check_breaker_position_clamped, check_physics_frozen_during_pause, check_no_entity_leaks, check_offering_no_duplicates, check_maxed_chip_never_offered
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
- 14 InvariantKind variants: BoltInBounds, BoltSpeedInRange, BoltCountReasonable, BreakerInBounds, NoEntityLeaks, NoNaN, TimerNonNegative, ValidStateTransitions, ValidBreakerState, TimerMonotonicallyDecreasing, BreakerPositionClamped, PhysicsFrozenDuringPause, OfferingNoDuplicates, MaxedChipNeverOffered
- Scenario categories (as of 2026-03-22): mechanic/ (12 scenarios), stress/ (15 scenarios), self_tests/ (12 scenarios)

## Still Stub (No Systems)
AudioPlugin

## RunPlugin Summary (as of 2026-03-23, memorable moments wave)
- FixedUpdate (PlayingState::Active): track_cells_destroyed, track_bumps, track_bolts_lost, track_time_elapsed, track_node_cleared_stats (.after TrackCompletion), detect_mass_destruction, detect_close_save (.after BreakerCollision), detect_combo_and_pinball, detect_nail_biter (.after TrackCompletion)
- Update (ChipSelect state): track_chips_collected, detect_first_evolution
- OnEnter(Playing): reset_highlight_tracker, capture_run_seed (unordered)
- OnEnter(TransitionIn): advance_node
- OnExit(MainMenu): reset_run_state → generate_node_sequence_system
- Registered: HighlightTriggered message; HighlightConfig, HighlightTracker, RunStats init_resource'd
- spawn_highlight_text IS registered in RunPlugin (Update, PlayingState::Active) — wiring gap resolved in C7-R (2026-03-25)

## Orphan Messages
- None at current phase. `ChipSelected` (UiPlugin) is now received by `chips/apply_chip_effect`.

## Spawn Coordination Messages
- `BoltSpawned` — BoltPlugin, sent by `spawn_bolt`
- `BreakerSpawned` — BreakerPlugin, sent by `spawn_breaker` (even when no-op, i.e. breaker already exists)
- `CellsSpawned` — NodePlugin, sent by `spawn_cells_from_layout`
- `WallsSpawned` — WallPlugin, sent by `spawn_walls`
- `SpawnNodeComplete` — NodePlugin, sent by `check_spawn_complete` coordinator; consumed by scenario runner only
- `NodeSystems::ApplyTimePenalty` — system set variant in NodeSystems enum
- `clamp_bolt_to_playfield` — BoltPlugin system, safety clamp after bolt_breaker_collision
- `build_chip_catalog` — LoadingPlugin system (builds ChipCatalog from ChipTemplateRegistry + EvolutionRegistry after both are seeded by RantzDefaultsPlugin)
- `RecordingPlugin` with `capture_frame` + `write_recording_on_exit` — debug-only input recorder
