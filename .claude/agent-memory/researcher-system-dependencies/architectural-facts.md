---
name: Architectural Facts
description: Key architectural facts, hot-reload pipeline, scenario runner systems, stubs, orphan messages
type: reference
---

## Key Architectural Facts
- All gameplay systems run in FixedUpdate gated by `run_if(in_state(PlayingState::Active))`
- Visual-only systems run in Update (animate_bump_visual, animate_tilt_visual, update_timer_display, debug overlays, update_lives_display, animate_fade_out)
- InterpolatePlugin registered BEFORE PhysicsPlugin in game.rs
- BehaviorsPlugin is STANDALONE, registered between BreakerPlugin and BoltPlugin
- Interpolation pipeline: `restore_authoritative` (FixedFirst) → [FixedUpdate physics] → `store_authoritative` (FixedPostUpdate) → `interpolate_transform` (PostUpdate)
- Bolt entities carry InterpolateTransform + PhysicsTranslation; bolt_lost inserts PhysicsTranslation on respawn
- Physics chain: `prepare_bolt_velocity` → `bolt_cell_collision` → `bolt_breaker_collision` → `apply_bump_velocity` + `spawn_additional_bolt` → `bolt_lost`
- apply_bump_velocity: `.after(BreakerCollision).before(BoltLost)`
- spawn_additional_bolt: `.after(BehaviorSystems::Bridge)`
- ExtraBolt: despawned permanently when lost (not respawned); still sends BoltLost message
- Behavior observer chain: bridge systems fire commands.trigger(ConsequenceFired) → observers run immediately
- apply_time_penalty: `.after(NodeSystems::TickTimer)` — can also send TimerExpired
- handle_run_lost: `.after(handle_node_cleared).after(handle_timer_expired)` — win takes priority
- Breaker state chain: `update_bump` → `move_breaker` → `update_breaker_state` → `grade_bump`
- Input: `read_input_actions` in PreUpdate writes InputActions consumed by FixedUpdate
- NodePlugin OnEnter chain: `set_active_layout` → `spawn_cells_from_layout` → `init_clear_remaining` → `init_node_timer`
- toggle_pause reads InputActions/GameAction::TogglePause
- bolt_breaker_collision: upward-bolt guard at top of bolt loop
- BoltHitCell message no longer carries bolt Entity field
- DebugOverlays: bool fields replaced by enum-indexed array
- bolt/queries.rs: BoltLostQuery type alias
- ChaosMonkey includes TogglePause in GAMEPLAY_ACTIONS

## Hot-Reload Pipeline (HotReloadPlugin, Update, GameState::Playing)
- Set 1 `PropagateDefaults` (11 systems): asset Modified event → Config resource or registry
- Set 2 `PropagateConfig` (2 systems): `.after(PropagateDefaults)`, gated by `resource_changed::<T>`
- Breaker path: direct `ResMut<BreakerConfig>` write → same-frame propagation
- Bolt/cell/etc.: `commands.insert_resource` → next-frame propagation
- `propagate_archetype_changes` also writes `ResMut<BreakerConfig>` and `ResMut<ActiveBehaviors>`
- `propagate_breaker_defaults` and `propagate_archetype_changes` both hold `ResMut<BreakerConfig>` — Bevy serializes, no race

## Scenario Runner (breaker-scenario-runner)
- 14 systems in FixedUpdate, 1 group in OnEnter(GameState::Playing)
- Lifecycle chain: `tick_scenario_frame → inject_scenario_input → check_frame_limit` before BreakerSystems::Move
- 12 invariant checkers (unordered, read-only)
- 2 mutators: enforce_frozen_positions, tag_game_entities
- OnEnter chain: `init_scenario_input → tag_game_entities → apply_debug_setup` after init_bolt_params
- InputStrategy: Chaos, Scripted, Hybrid

## Still Stub (No Systems)
AudioPlugin, UpgradesPlugin

## Orphan Messages
- `UpgradeSelected` (UiPlugin) — no sender or receiver. Expected for future phases.
