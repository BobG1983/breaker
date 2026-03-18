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
