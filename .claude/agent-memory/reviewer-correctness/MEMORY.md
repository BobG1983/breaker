# reviewer-correctness Memory

## Known Correct Patterns (Do Not Flag)
- `init_breaker_params` calls `insert_if_new((BumpPerfectMultiplier(1.0), BumpWeakMultiplier(1.0)))` BEFORE `init_archetype` runs (ordering: `init_breaker_params` → `init_archetype`). Archetype's plain `insert` then overwrites the 1.0 default — correct last-write-wins on node 1. On node re-entry both systems skip via `Without<BreakerMaxSpeed>` / `Without<LivesCount>` guards — multipliers preserved.
- `reset_breaker` uses `f32::midpoint(playfield.left(), playfield.right())` — equivalent to `(left + right) / 2.0`, always 0.0 for symmetric playfield. Correct.
- `handle_cell_hit` replaces HashSet with `Vec + peek()` early exit — correct; `despawned.contains()` O(n) is safe at MAX_BOUNCES=4 bound.
- `animate_fade_out` moved from bolt domain to UI domain — `Update` schedule, `run_if(in_state(PlayingState::Active))` guard unchanged. FadeOut entities have `CleanupOnNodeExit` so no accumulation across nodes.
- `spawn_bolt_lost_text` test imports `animate_fade_out` from `crate::ui::systems` after the move — correct, no stale import.
- `bolt_lost` respawns the bolt with randomized angle within `BoltRespawnAngleSpread` (±30° default) using `Vec2::new(speed*sin(angle), speed*cos(angle))` — angle-from-vertical convention, speed preserved via sin²+cos²=1. Correct.
- `set_active_layout` wraps `node_index % registry.layouts.len()` — deliberate, not a bug
- `handle_main_menu_input` reads `ButtonInput<KeyCode>` directly rather than `InputActions` — intentional; InputActions is cleared in FixedPostUpdate which is between PreUpdate and Update
- `spawn_bolt` adds `BoltServing` only on first node; subsequent nodes launch immediately — correct and tested
- `animate_bump_visual` subtracts the previous frame's offset before applying the new one — correct differential approach
- `track_node_completion` uses `remaining.is_changed()` — correct guard to avoid spurious `NodeCleared` on frames with no destroyed cells
- `handle_cell_hit` despawns via commands while iterating `reader.read()` — safe; despawn only takes effect when commands flush, not mid-iteration
- `spawn_side_panels` has an `existing.is_empty()` guard — it does NOT re-spawn on `Playing → NodeTransition → Playing` re-entry. The `StatusPanel` entity therefore persists across nodes (correct, it's a `CleanupOnRunEnd` child).
- `spawn_timer_hud` has no idempotency guard but relies on `CleanupOnNodeExit` to remove the old wrapper before `OnEnter(Playing)` fires again. The OnExit(Playing) cleanup runs before OnEnter(Playing) re-fires, so there is no double-spawn.
- `spawn_lives_display` uses `existing.iter().next().is_some()` guard — prevents re-spawn on node re-entry.
- Lives wrapper has no `CleanupOnNodeExit` or `CleanupOnRunEnd` — it is cleaned up via parent cascade when `StatusPanel` (CleanupOnRunEnd) is despawned on `OnExit(RunEnd)`. This is intentional: lives persist across nodes.
- Timer wrapper has `CleanupOnNodeExit` as a child of `StatusPanel` — double-despawn is NOT possible because `cleanup_entities::<CleanupOnNodeExit>` calls `commands.entity(e).despawn()` (not `despawn_recursive`), and the timer wrapper has `CleanupOnNodeExit` but `StatusPanel` does not. The parent despawn of `StatusPanel` (on RunEnd) would cascade to orphaned timers only if they weren't already cleaned up at node exit. They are cleaned at node exit, so by RunEnd they are already gone.
- `handle_run_setup_input` and `handle_pause_input` use `ButtonInput<KeyCode>` directly in `Update` — same pattern as main menu, correct for the same reason (InputActions cleared in FixedPostUpdate).
- `toggle_pause` is guarded by `run_if(in_state(GameState::Playing))` — it cannot fire in RunSetup, UpgradeSelect, or any other top-level state.
- `RunSetupSelection` is inserted as a resource in `spawn_run_setup` (OnEnter) and cleaned up when `cleanup_entities::<RunSetupScreen>` runs on `OnExit(RunSetup)`. The resource is NOT explicitly removed — it persists in the world as a stale resource. However, `spawn_run_setup` calls `insert_resource` (not `init_resource`) so it will be overwritten on the next `OnEnter(RunSetup)`. This is correct for repeated run-setup visits.
- `PauseMenuSelection` follows the same stale-resource pattern — re-inserted fresh on each `OnEnter(Paused)`. Correct.
- `ChipSelectTimer`, `ChipSelectSelection`, and `ChipOffers` follow the same stale-resource pattern — all re-inserted fresh by `spawn_chip_select` on each `OnEnter(ChipSelect)`. Correct.
- `transition_queued` in `RunState`: fixed — `advance_node` now resets `transition_queued = false` on each node transition. The flag correctly tracks per-node clear vs timer-expired tie-breaking.
- Bevy 0.18 sub-state `OnExit` fires when the parent state exits. `OnExit(PlayingState::Paused)` fires on `GameState` leaving `Playing`. No redundant cleanup needed for the pause menu quit path — the sub-state exit handler covers it automatically.
- `GameRng::default()` seeds from 0 (deterministic). `reset_run_state` reseeds via `ChaCha8Rng::from_os_rng()` (rand_chacha 0.9 correct API). No `GameRng` reseeding test exists — this is a coverage gap, not a bug.
- `MenuLeft`/`MenuRight` share ArrowLeft/ArrowRight with `MoveLeft`/`MoveRight` in the input bindings. Both actions are emitted simultaneously when those keys are pressed. Harmless: gameplay systems (Active state) only consume MoveLeft/DashLeft, menu screens read ButtonInput directly and ignore InputActions entirely.
- `update_run_setup_colors` sorts cards alphabetically before applying selection index — matches `handle_run_setup_input` which sorts registry keys alphabetically when resolving `selection.index` to an archetype name. Consistent. Correct.

## Known Bug Patterns
- **Double-tap consume uses 0.0 not NEG_INFINITY**: In `read_input_actions` (src/input/systems/read_input.rs:57, 86), after a dash fires, `last_left_tap` / `last_right_tap` is set to `0.0` instead of `f64::NEG_INFINITY`. During the first ~250ms of app lifetime (`elapsed_secs_f64() < double_tap_window`), any subsequent left/right key press would re-trigger a dash because `now - 0.0 < window`. In practice harmless (player won't have started yet), but technically wrong. Fix: use `f64::NEG_INFINITY` as the consume sentinel, same as the `Default`.

## Recurring Bug Categories
- **Stale screen resources**: `RunSetupSelection`, `PauseMenuSelection`, `ChipSelectTimer`, `ChipSelectSelection`, `ChipOffers` are inserted by spawn systems on `OnEnter` but never explicitly removed. They persist as stale resources between visits. This is safe because `insert_resource` overwrites on re-entry. Do not flag as a bug unless a system reads them outside the guarded state.
- **Stale selection index with variable card count**: Selection index resources persist across visits. All reset to 0 by spawn systems on `OnEnter`. Safe because always reset before input handler can run.
- **seed_upgrade_registry Local<bool> not reset across runs**: The `Local<bool>` seeded flag persists for the app lifetime. This is correct — Loading only runs once per app launch. Would be a bug if Loading could be re-entered, but it cannot (no transition back to Loading).

## State Machine Rules
- Valid transitions: Loading→MainMenu, MainMenu→RunSetup, RunSetup→Playing, Playing→NodeTransition→Playing (node advance), Playing→ChipSelect→NodeTransition→Playing (after non-final node), Playing→RunEnd (win/timer expire), RunEnd→MainMenu
- Pause sub-machine: Playing(Active)↔Playing(Paused), Paused+Quit→MainMenu (sets GameState directly, leaving PlayingState in Paused until Playing exits)
- `advance_node` runs `OnEnter(NodeTransition)` and immediately sets `NextState(Playing)` — 1-frame intermediate pattern
- `reset_run_state` runs `OnExit(MainMenu)` — resets node_index and outcome
- `handle_timer_expired` guards on `RunOutcome::InProgress` — prevents timer from overriding a Won run
- `handle_node_cleared` routes non-final nodes to `GameState::ChipSelect` instead of directly to `NodeTransition`
- `CleanupOnNodeExit` fires on `OnExit(GameState::Playing)` — fires on Playing→ChipSelect as well as Playing→RunEnd and Playing→NodeTransition. The ChipSelect screen has its own `cleanup_entities::<ChipSelectScreen>` on `OnExit(ChipSelect)`. Node entities (bolt, cells) are correctly despawned before ChipSelect.

## ECS Pitfalls Found
- `apply_bump_velocity` collects messages into a Vec before querying — correct pattern to avoid borrow conflicts between MessageReader and mutable Query
- `ChipSelected` message has no consumer yet (chips plugin is a stub). Messages are sent by `handle_chip_input` but silently dropped. No ECS error; Bevy messages are fire-and-forget. Will need a consumer in a later phase.
- `spawn_chip_select` takes `Res<ChipRegistry>` (not `Option<Res>`). If the registry is somehow absent at OnEnter(ChipSelect), Bevy will panic. Guaranteed safe in practice because Loading always completes before ChipSelect is reachable — but worth noting for future test harnesses.

## Math/Physics Notes
- `bolt_lost` angle: `Vec2::new(speed*angle.sin(), speed*angle.cos())` — angle-from-vertical convention. Speed preserved (sin²+cos²=1). Range `[-spread, +spread]` inclusive. Correct.
- `enforce_min_angle` uses `atan2(|y|, |x|)` — result is always [0, π/2], correct for angle-from-horizontal
- `reflect_top_hit`: `hit_fraction * max_angle + tilt_angle` clamped to `[-max_angle, max_angle]` — tilt can be fully cancelled by clamp when it pushes past the window; this is a design choice
- CCD `remaining -= advance` (not `advance + CCD_EPSILON`) — intentional; prevents sticking at contact surfaces

## Session History
See [ephemeral/](ephemeral/) — not committed.
