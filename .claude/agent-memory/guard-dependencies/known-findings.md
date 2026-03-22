---
name: known-findings
description: Accepted/wontfix findings with rationale from past audits
type: project
---

## Accepted Findings (updated 2026-03-22)

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

### OFL-1.1 / Ubuntu-font-1.0 (epaint_default_fonts) — OPEN
- **Finding:** `cargo deny check licenses` fails on OFL-1.1 and Ubuntu-font-1.0 from
  `epaint_default_fonts 0.33.3`, transitively via `bevy_egui → epaint`.
- **Status:** OPEN — deny.toml does not allow these licenses; cargo deny check licenses fails.
- **Compliance note:** OFL-1.1 (SIL Open Font License) is a font-specific copyleft license. It
  applies only to the font software itself, not to programs that use the fonts. Embedding OFL-1.1
  fonts in a proprietary binary is permitted under OFL-1.1 terms.
  Ubuntu-font-1.0 is the Ubuntu Font Licence — similar font-specific terms, embedding permitted.
- **Fix:** Add `"OFL-1.1"` and `"Ubuntu-font-1.0"` to the `allow` array in `deny.toml`.

### CC0-1.0 (hexf-parse) — OPEN
- **Finding:** `cargo deny check licenses` fails on CC0-1.0 from `hexf-parse 0.2.1`,
  transitively via `naga → wgpu → bevy_render`.
- **Status:** OPEN — deny.toml does not allow CC0-1.0; cargo deny fails.
- **Compliance note:** CC0-1.0 is a public domain dedication — the most permissive license possible.
  No attribution required, no copyleft, no restrictions. Safe for proprietary use.
- **Fix:** Add `"CC0-1.0"` to the `allow` array in `deny.toml`.

### Unicode-3.0 (unicode-ident) — OPEN
- **Finding:** `cargo deny check licenses` fails on Unicode-3.0 from `unicode-ident`,
  transitively via `naga → proc-macro2`.
- **Status:** OPEN — deny.toml does not allow Unicode-3.0; cargo deny fails.
- **Compliance note:** Unicode-3.0 (Unicode License v3) is OSI-approved. It is a permissive license
  that allows use in proprietary software. No copyleft.
- **Fix:** Add `"Unicode-3.0"` to the `allow` array in `deny.toml`.
