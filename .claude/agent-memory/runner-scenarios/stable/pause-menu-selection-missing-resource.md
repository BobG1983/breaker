---
name: PauseMenuSelection missing resource on FadeIn transition
description: handle_pause_input crashes when RunState::Setup→Node FadeIn pauses Time<Virtual> before PauseMenuSelection is inserted
type: project
---

## Root cause (confirmed, branch: feature/wall-builder-pattern)

**System:** `handle_pause_input` in `breaker-game/src/state/pause/systems/handle_pause_input.rs`

**Crash condition:** `handle_pause_input` runs in `Update` gated only by `run_if(is_time_paused)`. It requires `ResMut<PauseMenuSelection>` as a non-optional parameter. `PauseMenuSelection` is only inserted (via deferred `commands.insert_resource`) when `spawn_pause_menu` runs. Both systems share the `is_time_paused` condition. When both become eligible on the same frame, `PauseMenuSelection` does not yet exist (deferred commands haven't flushed), so param validation panics.

**Trigger path:** The `feature/wall-builder-pattern` branch added FadeIn transitions to declarative routes:
- `GameState::Loading → GameState::Menu` with `TransitionType::In(FadeIn)` 
- `RunState::Setup → RunState::Node` with `TransitionType::In(FadeIn)`

Both call `begin_transition` which pauses `Time<Virtual>`. When time is paused in a frame's `Update` schedule (by `dispatch_condition_routes` exclusive system), `tick_executor` is called from within the exclusive system's context. At that point, `handle_pause_input`'s run condition `is_time_paused` evaluates to TRUE, and `should_run` validates its params — `PauseMenuSelection` is absent → `SystemParamValidationError::invalid("Resource does not exist")` with `skipped: false` → error handler panics.

**Why this was previously safe:** On `develop`, these state transitions were plain direct state changes (no FadeIn), so `Time<Virtual>` was never paused by routing, `is_time_paused` was always false during setup, and `handle_pause_input` never ran before `PauseMenuSelection` was inserted.

**Fix options (do NOT apply — describe only):**
1. Change `ResMut<PauseMenuSelection>` to `Option<ResMut<PauseMenuSelection>>` in `handle_pause_input` and return early if `None`
2. Initialize `PauseMenuSelection` as a default resource in `PauseMenuPlugin` so it always exists

**Affected file:** `breaker-game/src/state/pause/systems/handle_pause_input.rs`
**Secondary affected:** `breaker-game/src/state/pause/plugin.rs` (could add `init_resource::<PauseMenuSelection>()`)

**Why:** `PauseMenuSelection` starts uninitialized.
**How to apply:** Any time ALL scenarios fail with "Resource does not exist" at frame=0 and FadeIn transitions are in use, check `handle_pause_input`'s resource requirements.
