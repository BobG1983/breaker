---
name: Scenario Runner Patterns
description: Confirmed correct patterns in the scenario runner — do not re-flag
type: reference
---

## Scenario Runner Patterns (feature/scenario-coverage-expansion)
- `check_bolt_in_bounds` only checks `y < bottom` — top/left/right have walls.
- `check_timer_monotonically_decreasing` resets Local to None when NodeTimer absent — correct.
- `check_physics_frozen_during_pause` clears HashMap on OnExit(Playing) — entity ID recycling not a concern.
- `init_scenario_input` resets InputDriver from scratch on each OnEnter(Playing) — intentional fresh chaos per node.
- `inject_scenario_input` hardcodes `is_active: true` — enables TogglePause injection while paused.
- `ScriptedInput::actions_for_frame` uses linear find() — O(n) fine for ≤200 entries.
- `HybridInput` scripted phase returns empty vec without advancing chaos RNG — correct.
- `evaluate_pass` gates on `logs.is_empty()` even with `expected_violations` — logs are unexpected warnings/errors.
- New mechanic and stress scenarios use standard invariant sets. Bolt count limits: 8 (stabilization), 12 (concurrent hits).
- `ScenarioDefinition.invariants` field is documentation-only — all invariant systems run unconditionally.
- `check_valid_breaker_state` legal transitions include `Dashing → Settling` — overly permissive but not a game bug.
- `check_no_entity_leaks` samples at frame 60, checks at every multiple of 120 after that (threshold: base * 2). First check at frame 120.
- `tag_game_entities` called both OnEnter(Playing) and every FixedUpdate tick — handles mid-game Prism extra bolt spawns. Without<Tag> filter prevents re-tagging.
- `enforce_frozen_positions` resets entity to ScenarioPhysicsFrozen.target every FixedUpdate tick — runs after physics. Correct.
- `bypass_menu_to_playing` goes MainMenu → Playing directly — not forbidden by `check_valid_state_transitions`.
- New mechanic scenarios (aegis_dash_wall, aegis_pause_stress, aegis_state_machine, aegis_speed_bounce, aegis_lives_exhaustion) and stress scenarios (aegis_multinode, prism_bolt_stabilization, prism_concurrent_hits, chrono_clear_race, chrono_penalty_stress) use standard invariant sets.
- `check_physics_frozen_during_pause` stores position every tick (active and paused), violations fire only when paused and bolt moved since last tick.

## feature/fix-scenario-log-sharing (lifecycle fixes)
- `inject_scenario_input` in `FixedPreUpdate` is correct: reads frame N, injects for frame N; `tick_scenario_frame` then increments to N+1 in `FixedUpdate`. Consistent with old behavior.
- `clear_input_actions` is registered in `FixedPostUpdate` (not `FixedPreUpdate` as its function docstring incorrectly states). The lifecycle.rs comment correctly says "FixedPostUpdate of previous tick."
- `Plugin::build()` calling `app.world().resource::<ScenarioConfig>()` is safe — `ScenarioConfig` is inserted immediately before `add_plugins(ScenarioLifecycle)` in runner.rs.
- `exit_on_run_end` in `Update` with `run_if(in_state(GameState::RunEnd))` is correct; writing `AppExit::Success` every frame is harmless, first write exits the headless loop.
- `restart_run_on_end` sets `NextState(MainMenu)` on `OnEnter(RunEnd)`: `RunEnd → MainMenu` is NOT in the forbidden set of `check_valid_state_transitions`. Correct.
- `bypass_menu_to_playing` re-sets `ScenarioLayoutOverride` on every `OnEnter(MainMenu)` — so repeated restarts correctly pin to the scenario's layout. Correct.
- `TimeUpdateStrategy::ManualDuration(10.0/64.0)` in visual mode: Winit respects this, advances virtual time 10 fixed steps per rendered frame. Achieves ~10x speedup. Correct.
- `allow_early_end` defaults to `true` via `#[serde(default = "ScenarioDefinition::default_allow_early_end")]` — existing RON files without the field get the old behavior (exit on RunEnd). Correct.

## Compact/Verbose Output + Dedup Refactor
- `Hash` derive added to `InvariantKind` — all unit variants; derived Hash is consistent with Eq. Correct.
- `group_violations` or_insert_with initializes `(0, v.frame, v.frame)` then increments entry.0 immediately — count starts at 0 and becomes 1 on first insert. Correct.
- `group_logs` key is `(format!("{:?}", l.level), l.message.clone())` — same level+message pairs correctly group; insertion_order uses moved original key; filter_map lookup uses insertion_order key. Correct.
- `is_health_check_reason` classifies "expected violation X never fired" as a health-check reason (not a violation or log reason) — prints it in compact mode under health checks. Intentional and complete.
- `is_invariant_fail_reason` uses exact string equality against all `fail_reason()` static strings — no overlap with "captured" prefix or "expected violation" prefix. Correct.
- verbose=false path calls `print_compact_failures`, verbose=true calls `print_verbose_failures` — wiring through run_with_args → run_scenario → collect_and_evaluate is complete. Correct.
- Cross-scenario summary: `i32::from(failed_count > 0)` returns 0 for all-pass, 1 for any-fail. Correct.

## feature/scenario-runner-dedup-summary — MinimalPlugins headless refactor

- `build_app(headless=true)` uses `MinimalPlugins + StatesPlugin + AssetPlugin + InputPlugin + MeshPlugin` — correct. MinimalPlugins does NOT include LogPlugin, StatesPlugin, AssetPlugin, InputPlugin, or MeshPlugin.
- `MinimalPlugins` content verified from Bevy 0.18.1 source: only `TaskPoolPlugin + FrameCountPlugin + TimePlugin + ScheduleRunnerPlugin`.
- Headless `first_run=false`: LogPlugin is simply not added (nothing to disable — MinimalPlugins has no LogPlugin). Asymmetry with visual branch's `else { disable::<LogPlugin>() }` is intentional and correct.
- `ScheduleRunnerPlugin` in MinimalPlugins is harmless in headless mode — its runner is only invoked by `app.run()`, which is never called; manual loop uses `app.update()` directly.
- `TimeUpdateStrategy::ManualDuration` overwrites `Automatic` (initialized by TimePlugin in MinimalPlugins) — supported pattern, verified from Bevy source.
- `app.init_asset::<ColorMaterial>()` correctly registers `Assets<ColorMaterial>` without render deps. Requires `AssetPlugin` to be present first (which it is).
- `bevy::text::TextPlugin` path confirmed valid (`bevy_internal` re-exports `bevy_text as text`). No RenderApp dependency. Registers Font asset + loader, CPU resources only.
- `bevy::mesh::MeshPlugin` path confirmed valid. Does `init_asset::<Mesh>()` which requires live `AssetServer` (AssetPlugin added first — correct ordering).
- `Mesh2d` has `#[require(Transform)]` — Transform has a default impl, no plugin required. MeshMaterial2d has no required components.
- `SpriteRenderPlugin` (which calls `register_required_components::<Sprite, SyncToRenderWorld>()`) is NOT added in headless — correct, avoids render world sync dependency.
- Game domain UiPlugin works without Bevy engine UiPlugin — UI layout systems don't run in headless but component types are available. Game tests confirm this.
- Simplified log filter `"warn,bevy_egui=error"` is correct — no render-related warnings fire because RenderPlugin is not loaded at all.
- `LogBuffer` sharing: first run extracts buffer from app world (inserted by scenario_log_layer_factory via LogPlugin); subsequent runs receive it via `insert_resource(buf.clone())`. Same Arc<Mutex<...>> writes to the global tracing subscriber. Correct.

## ScenarioVerdict Refactor (refactor/scenario-verdict)
- `evaluate()` clears `reasons` before building from scratch — correct, not a bug.
- `None | Some([])` slice pattern on `as_deref()` result is valid Rust — correctly matches both absent and empty expected_violations.
- `init_resource::<ScenarioVerdict>()` in lifecycle.rs registers a resource that `collect_and_evaluate` does not read from the world — `collect_and_evaluate` constructs its own local `ScenarioVerdict::default()`. This is intentional: the resource exists for the default-fail safety net pattern (any run that never calls evaluate() is still a safe fail), even though collect_and_evaluate doesn't read the world resource.
- `add_fail_reason` on a default verdict accumulates on top of the default reason — not a bug, just noisy output in the unreachable missing-resource path.
- `is_empty_scripted` macro pattern `if actions.is_empty()` guard works correctly because `actions` binds as `&Vec<ScriptedFrame>` and `.is_empty()` auto-derefs. Correct.
