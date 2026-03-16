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

### Issues Found (all resolved 2026-03-16)
- ~~`update_run_setup_colors.rs` — no tests~~ FIXED — 4 tests added (selected/unselected color, selection change, alphabetical sort)
- ~~`update_chip_display.rs` — no tests~~ FIXED — 4 tests added (timer ceiling, zero clamp, selected/unselected border)
- ~~`spawn_chip_select.rs` — user-facing strings used "upgrade"~~ FIXED — renamed to "CHOOSE A CHIP" + chip terminology throughout
- ~~`handle_chip_input.rs` — uses gameplay movement keys~~ FIXED — new `menu_left`/`menu_right` bindings added to InputConfig
