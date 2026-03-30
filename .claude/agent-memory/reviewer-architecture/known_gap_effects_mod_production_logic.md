---
name: Production logic extracted from effect/effects/mod.rs
description: effective_range, entity_position, spawn_extra_bolt extracted to effects/helpers.rs — mod.rs violation resolved but helpers.rs name is banned
type: project
---

**Resolved (partially):** The production functions `effective_range`, `entity_position`, and `spawn_extra_bolt` were extracted from `effect/effects/mod.rs` into `effect/effects/helpers.rs`. The mod.rs routing-only violation is fixed.

**Remaining issue:** The file is named `helpers.rs`, which is one of four explicitly banned names in `docs/architecture/layout.md` line 31. Needs renaming.

**Why:** `docs/architecture/layout.md` states: "No `utils.rs`, `helpers.rs`, `common.rs`, or `types.rs`." The ban exists to prevent catch-all files that accumulate unrelated code over time.

**How to apply:** Rename to a descriptive name (e.g., `spawn.rs`, `effect_helpers.rs`, or `spawn_helpers.rs`). Update `effects/mod.rs` module declaration to match.
