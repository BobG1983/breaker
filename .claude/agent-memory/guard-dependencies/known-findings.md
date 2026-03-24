---
name: known-findings
description: Accepted/wontfix findings with rationale from past audits
type: project
---

## Accepted Findings (updated 2026-03-23)

### proc-macro2 in breaker-derive flagged by machete
- **Finding:** cargo machete reports proc-macro2 as unused in breaker-derive
- **Status:** FALSE POSITIVE — proc-macro crate wontfix
- **Why:** The `quote!` macro emits `proc_macro2::TokenStream` types internally. Even though
  `proc_macro2` is not referenced directly in source, it is required for the crate to compile
  correctly when used as a proc-macro. This is a known machete limitation.
- **Fix applied:** `[package.metadata.cargo-machete] ignored = ["proc-macro2"]` — machete no longer flags it.

### bevy/serialize feature
- **Finding:** `bevy = { features = ["2d", "serialize"] }` — serialize may not be directly needed
- **Status:** DEFER — needs deeper investigation before removal
- **Why:** The project uses `serde::Deserialize` on structs with primitive fields (`f32`, `[f32; 3]`),
  not on Bevy math types. However, the `serialize` feature may be needed for internal Bevy
  scene/asset system behavior. Removing requires a full build + test cycle to verify no regressions.

### default = ["dev"] activates dev features in scenario runner — RESOLVED
- **Finding (historical):** `breaker-game/Cargo.toml` had `default = ["dev"]`, activating bevy_egui
  in the scenario runner.
- **Status:** FIXED — `breaker-scenario-runner/Cargo.toml` specifies
  `breaker = { path = "../breaker-game", default-features = false }`.
- **Residual:** The game crate still declares `default = ["dev"]`, which means bare `cargo build -p breaker`
  still activates dev features. This is intentional for the dev workflow.

### self_cell GPL-2.0-only OR Apache-2.0
- **Finding:** self_cell 1.2.2 is licensed `Apache-2.0 OR GPL-2.0-only` (transitive via bevy_text)
- **Status:** NOT a compliance issue — OR license allows Apache-2.0. Exception added in deny.toml.
- **Why:** Using the crate under Apache-2.0 is explicitly permitted.

### BSL-1.0 (clipboard-win) — RESOLVED
- **Status:** RESOLVED 2026-03-22 — "BSL-1.0" added to deny.toml allow list. No longer fails.

### MIT-0 (encase) — RESOLVED
- **Status:** RESOLVED 2026-03-22 — "MIT-0" added to deny.toml allow list. No longer fails.

### Workspace crates missing license field — RESOLVED
- **Status:** RESOLVED 2026-03-22 — `private.ignore = true` added to deny.toml [licenses] section.

### bevy_common_assets 0.16.0 upgrade — DEFERRED
- **Finding:** `cargo outdated` flags bevy_common_assets 0.15.0 → 0.16.0
- **Status:** DEFERRED — do not upgrade without verifying Bevy version compatibility
- **Why:** 0.16.0 targets `bevy ^0.18.0` (same as workspace — compatible). However, the `ron`
  feature in 0.16.0 still depends on `ron ^0.11`, the same as 0.15.0. Upgrading provides no
  reduction in the ron 0.11/0.12 duplicate. No changelog benefit identified for this project.
  The 0.15.0 → 0.16.0 diff should be reviewed before upgrading.
- **When to re-evaluate:** When Bevy upgrades, or when bevy_common_assets releases a version that
  uses `ron ^0.12` directly (which would eliminate the duplicate transitive ron).

### ron v0.11 transitive duplicate
- **Finding:** `cargo tree -d` shows `ron v0.11.0` alongside `ron v0.12.0` in the dep tree.
- **Status:** WONTFIX (upstream) — caused by `bevy_common_assets` requiring `ron ^0.11`. Both
  0.15.0 and 0.16.0 have this same dependency spec.
- **Impact:** Moderate — two ron versions compiled into the binary. Not a runtime conflict (different
  features from different crates), but does increase compile time and binary size.
- **Fix if needed:** Replace `bevy_common_assets` with a direct implementation using ron 0.12
  directly, or wait for a future bevy_common_assets release that bumps to `ron ^0.12`.

### OFL-1.1 / Ubuntu-font-1.0 (epaint_default_fonts) — RESOLVED
- **Status:** RESOLVED 2026-03-22 — "OFL-1.1" and "Ubuntu-font-1.0" added to deny.toml allow list.

### CC0-1.0 (hexf-parse) — RESOLVED
- **Status:** RESOLVED 2026-03-22 — "CC0-1.0" added to deny.toml allow list.

### Unicode-3.0 (unicode-ident) — RESOLVED
- **Status:** RESOLVED 2026-03-22 — "Unicode-3.0" added to deny.toml allow list.

### Workspace crates unlicensed — RESOLVED (2026-03-24)
- **Finding (historical):** `cargo deny check licenses` errors on workspace crates missing `publish = false`.
- **Status:** RESOLVED — all six workspace crates have `publish = false`. `cargo deny check licenses`
  passes cleanly with `licenses ok`.
- **Note:** The `breaker-derive` crate referenced in 2026-03-23 audit no longer exists. It was
  replaced by `rantzsoft_defaults_derive`, which carries `publish = false`.

### macOS platform dep tree objc2 version split — ACCEPTED (2026-03-24)
- **Finding:** `cargo tree -d` shows objc2 0.5.2 / 0.6.4, block2 0.5.1 / 0.6.2,
  objc2-foundation 0.2.2 / 0.3.2, objc2-app-kit 0.2.2 / 0.3.2, core-foundation 0.9.4 / 0.10.1.
- **Status:** WONTFIX (upstream) — driven by bevy_egui and winit pulling different objc2 generations.
- **Impact:** Compile time increase on macOS only. No runtime conflict.
- **Fix:** Resolves when bevy_egui or winit unify their objc2 version pins.
  Re-evaluate at next Bevy upgrade.
