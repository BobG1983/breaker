---
name: PauseMenuSelection missing resource on FadeIn transition — RESOLVED
description: handle_pause_input crash when FadeIn pauses Time<Virtual> before PauseMenuSelection exists — fixed by not_in_transition guard + PauseMenuScreen presence check
type: project
---

## RESOLVED — feature/wall-builder-pattern branch

**Original bug:** `handle_pause_input` crashed when `RunState::Setup→Node` FadeIn transition
paused `Time<Virtual>` before `PauseMenuSelection` was inserted. The system had `run_if(is_time_paused)`
but no guard against transitions or against the pause menu not yet existing.

**Fix (confirmed in `breaker-game/src/state/pause/plugin.rs`):**

`handle_pause_input` now has compound run condition:
```rust
is_time_paused
    .and(any_with_component::<PauseMenuScreen>)
    .and(not_in_transition.clone())
```

Where `not_in_transition = not(resource_exists::<ActiveTransition>)`.

This means:
1. `not_in_transition` — system does NOT run during FadeIn/FadeOut transitions (which pause `Time<Virtual>`)
2. `any_with_component::<PauseMenuScreen>` — system only runs AFTER `spawn_pause_menu` has created the screen entity
3. By the time `PauseMenuScreen` exists, `PauseMenuSelection` has also been inserted by `spawn_pause_menu`

**Why:** `PauseMenuSelection` starts uninitialized. The two-part guard ensures the system only runs when the pause menu is fully set up and no transition is active.

**How to apply:** If ALL scenarios fail with "Resource does not exist" at frame=0 and FadeIn transitions are in use, verify the `not_in_transition` guard is present on any system that requires resources only available after pause menu spawn.
