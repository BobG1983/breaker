---
name: color_from_rgb canonical location — duplicate REMOVED
description: color_from_rgb lives in shared/color.rs; the former chip_select/mod.rs duplicate was removed in the state folder restructure (2026-04-02)
type: project
---

`color_from_rgb` is canonically defined in `shared/color.rs` and re-exported from `shared/mod.rs` as `pub use color::color_from_rgb`.

The duplicate in `chip_select/mod.rs` that previously existed was REMOVED in the state folder restructure (refactor/state-folder-structure, 2026-04-02). The `state/run/chip_select/mod.rs` now contains only module declarations and re-exports — no duplicate function.

**Verified 2026-04-04:** `chip_select/mod.rs` has no `color_from_rgb` definition.

**How to apply:** Only one definition exists — `crate::shared::color_from_rgb`. Do NOT flag imports from shared as incorrect.
