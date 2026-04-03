---
name: color_from_rgb canonical location
description: color_from_rgb lives in shared/color.rs; chip_select/mod.rs has a duplicate
type: project
---

`color_from_rgb` is canonically defined in `shared/color.rs` and re-exported from `shared/mod.rs` as `pub use color::color_from_rgb`.

`chip_select/mod.rs` contains a duplicate `pub(crate) const fn color_from_rgb` at line 15. This should be removed and replaced with a re-export of `crate::shared::color_from_rgb`.

**Why:** The duplicate was left over when code was moved into the state/ folder structure. The chip_select systems import from `chip_select::*` which resolves to the local copy instead of shared.

**How to apply:** When reviewing chip_select code, check that `color_from_rgb` is imported from `crate::shared`, not from `crate::state::run::chip_select`.
