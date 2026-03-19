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
- `bevy::mesh::MeshPlugin` path confirmed valid. Does `init_asset::<Mesh>()` which requires live `AssetServer` (AssetPlugin added first — correct ordering).
- `Mesh2d` has `#[require(Transform)]` — Transform has a default impl, no plugin required. MeshMaterial2d has no required components.
- `SpriteRenderPlugin` (which calls `register_required_components::<Sprite, SyncToRenderWorld>()`) is NOT added in headless — correct, avoids render world sync dependency.
- Game domain UiPlugin works without Bevy engine UiPlugin — UI layout systems don't run in headless but component types are available. Game tests confirm this.
- Simplified log filter `"warn,bevy_egui=error"` is correct — no render-related warnings fire because RenderPlugin is not loaded at all.
- `LogBuffer` sharing: first run extracts buffer from app world (inserted by scenario_log_layer_factory via LogPlugin); subsequent runs receive it via `insert_resource(buf.clone())`. Same Arc<Mutex<...>> writes to the global tracing subscriber. Correct.

## HeadlessAssetsPlugin refactor (feature/scenario-runner-dedup-summary continued)

- `HeadlessAssetsPlugin` added to `Game::headless()` PluginGroupBuilder after all domain plugins.
- `HeadlessAssetsPlugin::build()` calls `app.init_asset::<ColorMaterial>()` and `app.add_plugins(bevy::text::TextPlugin)`.
- Plugin build order: when runner.rs adds `AssetPlugin` before `Game::headless()`, AssetPlugin is fully built before HeadlessAssetsPlugin executes — correct. init_asset::<ColorMaterial>() requires a live AssetServer which is present.
- `bevy::text::TextPlugin` path confirmed valid. No RenderApp dependency. Registers Font asset + loader, CPU resources only.
- `HeadlessAssetsPlugin` added last in PluginGroupBuilder — domain plugins only schedule systems at build time; asset types registered by HeadlessAssetsPlugin are available before any system runs. No ordering hazard.
- `PluginGroupBuilder::disable::<T>()` panics if T is NOT in the plugins map (verified from Bevy 0.18.1 source, plugin_group.rs:502-508). However, it does NOT panic if T is already disabled (just sets enabled=false again). Double-disable in game.rs test_app(Game::headless()) is benign.
- `scenario_log_plugin()` returns `LogPlugin`. Used as `app.add_plugins(LogPlugin)` in headless mode (correct — Plugin implements Plugins). Used as `defaults.set(LogPlugin)` in visual mode (correct — LogPlugin is in DefaultPlugins so set() won't panic).
- `PluginGroupBuilder::set()` panics if the plugin type is absent from the group. DefaultPlugins always includes LogPlugin, so `defaults.set(scenario_log_plugin())` is safe.

## feature/scenario-runner-dedup-summary — invariant checker bug fixes (2026-03-18)

- `check_valid_breaker_state` `retain` at end of loop: runs after insert, correctly removes stale entries after live entities are updated. Bevy's generational IDs ensure a recycled entity has a different `Entity` value so old entries are pruned. Confirmed correct.
- `check_bolt_speed_in_range` `SPEED_TOLERANCE = 1.0`: widens both bounds symmetrically. Zero-speed guard still skips serving bolts. Edge case where `min_speed < 1.0` would make lower bound negative — acceptable for an invariant checker. Confirmed correct.
- `check_timer_monotonically_decreasing` `(remaining, total)` tracking: `f32::EPSILON` threshold on total is safe because `total` is set once and never mutated during a node (only `remaining` changes). Same-total consecutive nodes are handled by the resource removal/re-insertion between nodes (which resets `previous` to `None`). Confirmed correct.
- `check_bolt_in_bounds` margin `r.0 + 1.0`: applied correctly in all four directions. Fallback to 0.0 when `BoltRadius` absent is safe — no false positives. Violation message shows `bottom_bound=...` without the margin offset — not a logic bug, just slightly less informative for debugging. Confirmed correct.

## ScenarioVerdict Refactor (refactor/scenario-verdict)
- `evaluate()` clears `reasons` before building from scratch — correct, not a bug.
- `None | Some([])` slice pattern on `as_deref()` result is valid Rust — correctly matches both absent and empty expected_violations.
- `init_resource::<ScenarioVerdict>()` in lifecycle.rs registers a resource that `collect_and_evaluate` does not read from the world — `collect_and_evaluate` constructs its own local `ScenarioVerdict::default()`. This is intentional: the resource exists for the default-fail safety net pattern (any run that never calls evaluate() is still a safe fail), even though collect_and_evaluate doesn't read the world resource.
- `add_fail_reason` on a default verdict accumulates on top of the default reason — not a bug, just noisy output in the unreachable missing-resource path.
- `is_empty_scripted` macro pattern `if actions.is_empty()` guard works correctly because `actions` binds as `&Vec<ScriptedFrame>` and `.is_empty()` auto-derefs. Correct.

## feature/scenario-runner-dedup-summary — run_all_parallel (2026-03-18)

- **CONFIRMED BUG (fixed in later commit)**: `run_all_parallel` called `child.wait()` BEFORE reading stdout/stderr → pipe-buffer deadlock. Fixed with `child.wait_with_output()`.
- `batch_size = jobs.unwrap_or(names.len()).max(1)` — `jobs=Some(0)` becomes 1 (sequential). Correct edge-case handling.
- Spawn failure path: adds `ChildResult { passed: false }` immediately, does not push to `children` vec, no wait attempted. Correct.
- `print_summary` called from both `run_with_args` and `run_all_parallel`. Correct.
- Each child's own `print_summary` output ("--- scenario result:") is captured in stdout and re-printed indented in parent. Slightly noisy but not a logic error.
- `run_with_args` `all: bool` parameter is now dead (main always passes false). Not a logic bug.
- `drop(out.read_to_string(&mut stdout))` silently discards I/O errors after child exit. Acceptable.

## feature/scenario-runner-dedup-summary — clap refactor + --loop + --serial (2026-03-18)

- Fast path `args.scenario.is_some() && !args.all && loop_count == 1 && !args.execution.serial` — correctly skips for `--loop N` and `--serial`. `--visual -s foo` still goes through fast path (in-process, visual=true, headless=false). Correct.
- `Parallelism::resolve(Count(n))` ignores `total` — returns `n.max(1)` unconditionally. `Count(100)` with 3 scenarios → chunks(100) gives one batch of 3. Correct.
- `run_all_parallel` redundant `.max(1)` (batch_size = parallelism.max(1)): `resolve()` already guarantees ≥1. Redundant but harmless.
- `--visual --serial` guard (runs.len() > 1) correctly blocks multi-scenario visual serial. Does NOT block single-scenario visual serial — intentional (app.run() works once).
- **CONFIRMED BUG**: `--visual --serial --loop N` (N > 1) with any single scenario: the guard at line 60-65 only checks `runs.len() > 1`, not the loop count. On the second iteration, `run_all_serial` calls `run_scenario(headless=false)` which calls `build_app(headless=false, first_run=true)` (always true because `shared_log_buffer` is reset to `None` on each `run_all_serial` call) and then `app.run()`. Winit event loop cannot be started a second time in the same process → crash.
- `--visual --loop N` without `--serial` is safe: uses `run_all_parallel` which spawns subprocesses; each subprocess sees `--visual` without `--loop`, runs Winit exactly once. Correct.
- `parse_loop_count` correctly rejects 0. `parse_parallelism` correctly rejects 0 and non-numeric strings.
- clap `conflicts_with` is bidirectional: `parallel` has `conflicts_with = "serial"` and `serial` has `conflicts_with = "parallel"`. Clap handles both directions. Correct.
- `loop_count = args.loops.unwrap_or(1)`: default of 1 means no-`--loop` behaves as exactly one iteration. Correct.

## feature/scenario-runner-dedup-summary — SpawnChecklist + EntityLeakBaseline (2026-03-18)

- `check_spawn_complete` `Local<SpawnChecklist>` bitfield: 4 bits (BOLT|BREAKER|CELLS|WALLS). `is_complete()` uses `& Self::ALL == Self::ALL`. Correct.
- `check_spawn_complete` resets checklist after firing SpawnNodeComplete — `*checklist = SpawnChecklist::default()`. MessageReader cursor advances on `read()`, so messages from the same OnEnter don't re-trigger on frame N+1. No double-fire.
- `check_spawn_complete` runs in `FixedUpdate` with NO `run_if` guard — messages only arrive from `OnEnter(Playing)` so it's a no-op outside Playing. Correct.
- `spawn_breaker` idempotency: existing breaker → sends `BreakerSpawned` then returns. Freshly spawned → also sends `BreakerSpawned`. Both paths covered. Correct.
- Tuple render assets `(ResMut<Assets<Mesh>>, ResMut<Assets<ColorMaterial>>)` in `spawn_bolt` and `spawn_cells_from_layout`: destructured as `.0` and `.1`. Correct Bevy 0.18 system param pattern.
- `check_no_entity_leaks` samples `count` BEFORE reading SpawnNodeComplete, then sets baseline. On the same frame SpawnComplete arrives, `count > base * 2` → `count > count * 2` → false. No false positive.
- `check_no_entity_leaks` frame.0.is_multiple_of(120): frame 0 is a multiple of 120, but baseline is None at that point — guard returns early. First real check at next multiple of 120 after baseline is set. Correct.
- `add_message::<SpawnNodeComplete>()` called from both NodePlugin and ScenarioLifecycle. `init_resource` is idempotent in Bevy 0.18. No panic. Correct.
- NodePlugin::plugin_builds test does NOT register BoltSpawned/BreakerSpawned/WallsSpawned. FixedUpdate doesn't fire on first app.update() when Time<Fixed> accumulator starts at 0 with no ManualDuration. Test passes without panic. Correct.
- WallsSpawned is pub(crate): accessible from check_spawn_complete (same crate). Scenario runner only uses SpawnNodeComplete (pub). Correct.
- SpawnNodeComplete written by check_spawn_complete in frame N is readable by check_no_entity_leaks in frame N+1 (double-buffer semantics). Entity count at N+1 equals N (no spawning/despawning between). Correct 1-frame delay.
