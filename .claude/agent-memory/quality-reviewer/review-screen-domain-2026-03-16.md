---
name: screen-domain-review-2026-03-16
description: Quality review of run_setup, pause_menu, upgrade_select, loading/seed_upgrade_select_config, and screen plugin additions
type: project
---

## Review: Screen Domain — run_setup, pause_menu, upgrade_select

**Why:** Phase implementation adding three new screen sub-domains
**How to apply:** These patterns are now established in the codebase; flag deviations in future work

### Intentional Patterns Confirmed
- `RunSetupSelection` lives in `spawn_run_setup.rs` (same file as `PauseMenuSelection` in `spawn_pause_menu.rs`). This is the established pattern for co-locating the selection resource with the spawn system that inserts it. Other systems in the same domain import it via `use super::spawn_*.rs::*`. Acceptable pattern for intra-domain use — no public re-export needed.
- `UpgradeTimerText` marker also lives in `spawn_upgrade_select.rs` for the same reason.
- `description_for()` in `spawn_run_setup.rs` — hardcoded archetype descriptions intentional placeholder for Phase 7 content.
- `unwrap_or(0)` in `current_index()` helpers in `handle_main_menu_input.rs` and `handle_pause_input.rs` — pre-existing pattern; the fallback is a safe default (selection is always in the array). Not a new violation.

### Issues Found
- `update_run_setup_colors.rs` — no tests. Only system in the new screens with zero test coverage.
- `update_upgrade_display.rs` — no tests. Timer display and border color updates go untested.
- `spawn_upgrade_select.rs` — user-facing strings "CHOOSE AN UPGRADE", "UPGRADE A/B/C", "Placeholder upgrade effect" use the banned term "upgrade" instead of "Amp". Placeholder content — document with a TODO referencing Phase 7.
- `handle_upgrade_input.rs` — uses `config.move_left`/`config.move_right` for card navigation (gameplay movement keys). Main menu and pause menu use `menu_up`/`menu_down`. This is a divergence — left/right navigation is intentional for horizontal card layout, but no comment explains the decision.
