---
name: Known Correct Patterns
description: Code patterns confirmed correct that should not be re-flagged in reviews
type: reference
---

## Known Correct Patterns (Do Not Flag)
- `init_breaker_params` calls `insert_if_new` BEFORE `init_archetype` runs. Archetype's plain `insert` overwrites defaults — correct last-write-wins.
- `reset_breaker` uses `f32::midpoint(playfield.left(), playfield.right())` — correct.
- `handle_cell_hit` replaces HashSet with `Vec + peek()` early exit — correct at MAX_BOUNCES=4 bound.
- `animate_fade_out` in UI domain (Update, PlayingState::Active guard). FadeOut entities have `CleanupOnNodeExit`.
- `bolt_lost` respawn angle: `Vec2::new(speed*sin(angle), speed*cos(angle))` — angle-from-vertical convention, speed preserved.
- `set_active_layout` wraps `node_index % registry.layouts.len()` — deliberate.
- `handle_main_menu_input` reads `ButtonInput<KeyCode>` directly (InputActions cleared in FixedPostUpdate) — intentional.
- `spawn_bolt` adds `BoltServing` only on first node; subsequent nodes launch immediately.
- `animate_bump_visual` subtracts previous frame's offset before applying new one — correct differential.
- `track_node_completion` uses `remaining.is_changed()` — correct guard.
- `handle_cell_hit` despawns via commands while iterating `reader.read()` — safe; commands flush later.
- `spawn_side_panels` has `existing.is_empty()` guard — does NOT re-spawn on node re-entry. StatusPanel persists (CleanupOnRunEnd).
- `spawn_timer_hud` has explicit `if !existing.is_empty() { return; }` guard.
- `spawn_lives_display` uses `existing.iter().next().is_some()` guard.
- Lives wrapper has no cleanup marker — cleaned via parent cascade when StatusPanel despawned.
- Timer wrapper has `CleanupOnNodeExit` — cleaned at node exit, gone by RunEnd.
- `handle_run_setup_input` and `handle_pause_input` use `ButtonInput<KeyCode>` directly — same pattern as main menu.
- `toggle_pause` guarded by `run_if(in_state(GameState::Playing))`.
- `RunSetupSelection`, `PauseMenuSelection`, `ChipSelectTimer`, `ChipSelectSelection`, `ChipOffers` — stale-resource pattern. All re-inserted fresh on OnEnter. Correct.
- `transition_queued` in RunState: `advance_node` resets to false on each node transition.
- Bevy 0.18 sub-state `OnExit` fires when parent state exits. No redundant cleanup needed for pause quit.
- `GameRng::default()` seeds from 0. `reset_run_state` reseeds via `ChaCha8Rng::from_os_rng()`.
- `MenuLeft`/`MenuRight` share keys with `MoveLeft`/`MoveRight` — harmless, different state contexts.
- `update_run_setup_colors` sorts cards alphabetically, matching `handle_run_setup_input`.
- `apply_debug_setup` uses post-teleport `transform.translation` for `ScenarioPhysicsFrozen.target` — correct because mutation happens before the insert call.
- `check_timer_monotonically_decreasing` resets Local to None when NodeTimer absent (node transition) — correct; no false positive on new node start.
- `check_bolt_count_reasonable` queries `With<ScenarioTagBolt>` — one-frame tolerance on newly spawned extra bolts is acceptable.
- `evaluate_pass` with `expected_violations: Some([])` → requires `violations.is_empty() && logs.is_empty()` — correct (vacuously all-fired AND all-in-list).
- `HybridInput::actions_for_frame` returns empty during scripted phase without advancing chaos RNG — correct, RNG state only advances when chaos phase is active.
- `#![cfg_attr(test, allow(...))]` in lib.rs is the approved conditional form, not a bare `#[allow(...)]` — do not re-flag.
- `check_valid_breaker_state` legal set includes `Settling → Dashing` — correct; `handle_idle_or_settling` allows dash from Settling state.
- `RenderSetupPlugin` inserts `ClearColor(PlayfieldConfig::default().background_color())` at plugin-build time using compile-time defaults — intentional; RON default matches Rust default `[0.02, 0.01, 0.04]`.
- `Game` uses `#[derive(Default)]` for `headless: false` — correct; bool defaults to false, identical to removed manual impl.
- `app.rs` `headless_app()` test helper uses `Game::default()` (headless=false, includes RenderSetupPlugin) — intentional; `camera_spawns` test verifies camera spawning.
- `runner.rs` `build_app(headless=true)` correctly uses `Game::headless()`, `build_app(headless=false)` correctly uses `Game::default()`.
- `RenderSetupPlugin` is added last in the PluginGroupBuilder chain — correct; Startup ordering does not matter here.
- `build_app(headless=true)` in runner.rs uses `MinimalPlugins + StatesPlugin + AssetPlugin + InputPlugin + MeshPlugin + init_asset::<ColorMaterial>() + TextPlugin + TimeUpdateStrategy::ManualDuration + Game::headless()` — reviewed 2026-03-18, confirmed correct. No missing plugins, no spurious render hacks needed.
