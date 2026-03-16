# correctness-reviewer Memory

## Known Correct Patterns (Do Not Flag)
- `init_breaker_params` calls `insert_if_new((BumpPerfectMultiplier(1.0), BumpWeakMultiplier(1.0)))` BEFORE `init_archetype` runs (ordering: `init_breaker_params` → `init_archetype`). Archetype's plain `insert` then overwrites the 1.0 default — correct last-write-wins on node 1. On node re-entry both systems skip via `Without<BreakerMaxSpeed>` / `Without<LivesCount>` guards — multipliers preserved.
- `reset_breaker` uses `f32::midpoint(playfield.left(), playfield.right())` — equivalent to `(left + right) / 2.0`, always 0.0 for symmetric playfield. Correct.
- `handle_cell_hit` replaces HashSet with `Vec + peek()` early exit — correct; `despawned.contains()` O(n) is safe at MAX_BOUNCES=4 bound.
- `animate_fade_out` moved from bolt domain to UI domain — `Update` schedule, `run_if(in_state(PlayingState::Active))` guard unchanged. FadeOut entities have `CleanupOnNodeExit` so no accumulation across nodes.
- `spawn_bolt_lost_text` test imports `animate_fade_out` from `crate::ui::systems` after the move — correct, no stale import.
- `bolt_lost` immediately respawns the bolt with straight-up velocity — intentional per design ("losing position is penalty enough")
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
- `UpgradeSelectTimer` and `UpgradeSelectSelection` follow the same stale-resource pattern. Correct.
- `transition_queued` in `RunState` is set to `true` by `handle_node_cleared` and is reset by `reset_run_state` (which runs `OnExit(MainMenu)`). It is NOT reset between nodes during a run. After the first UpgradeSelect transition, it stays `true` for all subsequent nodes. This means `handle_timer_expired` will always yield to `handle_node_cleared` on subsequent nodes. This is a known design choice — the timer-vs-clear tie-break always favors the clear on node 2+. The design intent (clear beats loss) is still upheld; only the first node needs the explicit flag to work correctly. Accepted behavior.

## Recurring Bug Categories
- **Partial message drain**: `bridge_bolt_lost` uses `reader.read().next().is_none()` which only checks the first message. Multiple simultaneous BoltLost messages (future Prism archetype with multiple bolts) will have extras silently consumed without firing consequences. Harmless with one bolt.
- **Stale screen resources**: `RunSetupSelection`, `PauseMenuSelection`, `UpgradeSelectTimer`, `UpgradeSelectSelection` are inserted by spawn systems on `OnEnter` but never explicitly removed. They persist as stale resources between visits. This is safe because `insert_resource` overwrites on re-entry. Do not flag as a bug unless a system reads them outside the guarded state.

## State Machine Rules
- Valid transitions: Loading→MainMenu, MainMenu→RunSetup, RunSetup→Playing, Playing→NodeTransition→Playing (node advance), Playing→UpgradeSelect→NodeTransition→Playing (after non-final node), Playing→RunEnd (win/timer expire), RunEnd→MainMenu
- Pause sub-machine: Playing(Active)↔Playing(Paused), Paused+Quit→MainMenu (sets GameState directly, leaving PlayingState in Paused until Playing exits)
- `advance_node` runs `OnEnter(NodeTransition)` and immediately sets `NextState(Playing)` — 1-frame intermediate pattern
- `reset_run_state` runs `OnExit(MainMenu)` — resets node_index and outcome
- `handle_timer_expired` guards on `RunOutcome::InProgress` — prevents timer from overriding a Won run
- `handle_node_cleared` now routes non-final nodes to `GameState::UpgradeSelect` instead of directly to `NodeTransition`
- `CleanupOnNodeExit` fires on `OnExit(GameState::Playing)` — this means it fires on Playing→UpgradeSelect as well as Playing→RunEnd and Playing→NodeTransition. The UpgradeSelect screen has its own `cleanup_entities::<UpgradeSelectScreen>` on `OnExit(UpgradeSelect)`. Node entities (bolt, cells) are correctly despawned before UpgradeSelect.

## ECS Pitfalls Found
- `bridge_bolt_lost` partial drain (see Recurring Bug Categories)
- `apply_bump_velocity` collects messages into a Vec before querying — correct pattern to avoid borrow conflicts between MessageReader and mutable Query

## Math/Physics Notes
- `enforce_min_angle` uses `atan2(|y|, |x|)` — result is always [0, π/2], correct for angle-from-horizontal
- `reflect_top_hit`: `hit_fraction * max_angle + tilt_angle` clamped to `[-max_angle, max_angle]` — tilt can be fully cancelled by clamp when it pushes past the window; this is a design choice
- CCD `remaining -= advance` (not `advance + CCD_EPSILON`) — intentional; prevents sticking at contact surfaces
