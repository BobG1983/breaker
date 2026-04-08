---
name: iyes_progress stateflow integration pattern
description: How to use iyes_progress 0.16 without auto-transitioning state — poll ProgressTracker directly instead
type: project
---

`iyes_progress` 0.16 is in use at `breaker-game/Cargo.toml`. The project uses `ProgressPlugin::<AppState>::new().with_state_transition(AppState::Loading, AppState::Game)` which bypasses `rantzsoft_stateflow` and skips the FadeOut transition.

**Key API facts (confirmed from source):**
- `.with_state_transition()` is fully optional — omitting it means the plugin tracks progress but never calls `NextState::set`
- `Res<ProgressTracker<AppState>>` is freely readable from any system — call `.is_ready()` to check completion
- There are no events, callbacks, or hooks — polling only
- Removing `.with_state_transition()` also removes the `OnEnter(from)` auto-clear; add `OnEnter(AppState::Loading)` calling `clear_global_progress::<AppState>` if the loading state can be re-entered
- Zero-value guard needed: `is_ready()` returns `true` when `total == 0`; poll only when `total > 0`

**Why:** The loading transition is the only route in the game that skips the FadeOut animation. All other transitions go through `rantzsoft_stateflow`. The fix is to remove `.with_state_transition()` and replace with a polling system that dispatches via the stateflow route.

**How to apply:** When asked about loading screen transitions or AppState::Loading routing, reference this pattern.
