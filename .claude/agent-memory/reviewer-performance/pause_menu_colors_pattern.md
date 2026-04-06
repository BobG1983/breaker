---
name: update_pause_menu_colors pattern
description: 2-entity Update query gated on is_time_paused + PauseMenuScreen; unconditional write on every frame while paused
type: project
---

`update_pause_menu_colors` in `breaker-game/src/state/pause/systems/update_pause_menu_colors.rs`:

- Queries `(&PauseMenuItem, &mut TextColor)` — exactly 2 entities
- Runs in `Update` gated by `is_time_paused.and(any_with_component::<PauseMenuScreen>).and(not_in_transition)`
- Writes `TextColor` unconditionally every frame while paused regardless of whether the selection changed
- `PauseMenuSelection` is a `Res<>` (immutable borrow), but change detection on it is not consulted
- No allocations; no loops over large sets

**Findings (at review time):**
- Unconditional write on 2 entities is negligible at current scale — not a performance issue
- Adding `selection.is_changed()` guard would be a correctness-style improvement for dirty-marking accuracy,
  but with 2 entities and a tight pause menu lifecycle, this is Minor at most
- `&mut TextColor` is correct — write is always needed on first frame post-spawn and on navigation changes;
  a change-detection guard would need careful handling of the spawn-frame case

**How to apply:** Pattern is acceptable. Only flag if entity count grows or system moves to an ungated hot path.
