---
name: known-findings
description: Accepted/wontfix findings with rationale from past audits
type: project
---

## Accepted Findings (2026-03-19 audit)

### proc-macro2 in breaker-derive flagged by machete
- **Finding:** cargo machete reports proc-macro2 as unused in breaker-derive
- **Status:** FALSE POSITIVE — proc-macro crate wontfix
- **Why:** The `quote!` macro emits `proc_macro2::TokenStream` types internally. Even though
  `proc_macro2` is not referenced directly in source, it is required for the crate to compile
  correctly when used as a proc-macro. This is a known machete limitation.
- **Rationale:** Do not flag on future runs. Can add `[package.metadata.cargo-machete] ignored = ["proc-macro2"]` to suppress if desired.

### bevy/serialize feature
- **Finding:** `bevy = { features = ["2d", "serialize"] }` — serialize may not be directly needed
- **Status:** DEFER — needs deeper investigation before removal
- **Why:** The project uses `serde::Deserialize` on structs with primitive fields (`f32`, `[f32; 3]`),
  not on Bevy math types. However, the `serialize` feature may be needed for internal Bevy
  scene/asset system behavior. Removing requires a full build + test cycle to verify no regressions.

### default = ["dev"] activates dev features in scenario runner
- **Finding:** `breaker-game/Cargo.toml` has `default = ["dev"]`, which activates `bevy_egui` and
  `bevy/file_watcher` in the scenario runner's release build (since it depends on `breaker` with no
  explicit features override).
- **Status:** ACCEPTED TRADEOFF (as of 2026-03-19) — pending decision from project owner
- **Impact:** `bevy_egui` and `bevy/file_watcher` compiled into release scenario runner binary.
  Increases compile time and binary size for a headless tool.
- **Fix:** Remove `dev` from `default = ["dev"]` and explicitly pass `--features dev` in
  `.cargo/config.toml` dev aliases. OR add `default-features = false` to the breaker dep in
  scenario runner's Cargo.toml.

### self_cell GPL-2.0-only OR Apache-2.0
- **Finding:** self_cell 1.2.2 is licensed `Apache-2.0 OR GPL-2.0-only` (transitive via bevy_text)
- **Status:** NOT a compliance issue — OR license allows Apache-2.0
- **Why:** Using the crate under Apache-2.0 is explicitly permitted. This is a proprietary project
  using Apache-2.0 terms. No action needed.
