---
name: Production logic extracted from effect/effects/mod.rs
description: RESOLVED — effective_range, entity_position, spawn_extra_bolt are in fire_helpers.rs; mod.rs is routing-only; banned name issue fixed
type: project
---

**Fully resolved as of full-verification-fixes branch (2026-03-30).**

The production functions `effective_range`, `entity_position`, and `spawn_extra_bolt` were extracted from `effect/effects/mod.rs` into `effect/effects/fire_helpers.rs`. The mod.rs routing-only violation is fixed, and `fire_helpers.rs` is a descriptive name (not the banned `helpers.rs`).

`effect/effects/mod.rs` now only re-exports via `pub(crate) use fire_helpers::{...}`. No production logic in mod.rs.

No open gaps remaining for this item. When reviewing new effects, flag any production logic added directly to `effects/mod.rs`.
