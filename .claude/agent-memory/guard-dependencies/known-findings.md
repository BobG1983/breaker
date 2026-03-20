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

### default = ["dev"] activates dev features in scenario runner — RESOLVED
- **Finding (historical):** `breaker-game/Cargo.toml` had `default = ["dev"]`, activating bevy_egui
  in the scenario runner.
- **Status:** FIXED in the 2026-03-19 session — `breaker-scenario-runner/Cargo.toml` now specifies
  `breaker = { path = "../breaker-game", default-features = false }`.
- **Residual:** The game crate still declares `default = ["dev"]`, which means bare `cargo build -p breaker`
  still activates dev features. This is intentional for the dev workflow. The scenario runner is clean.

### self_cell GPL-2.0-only OR Apache-2.0
- **Finding:** self_cell 1.2.2 is licensed `Apache-2.0 OR GPL-2.0-only` (transitive via bevy_text)
- **Status:** NOT a compliance issue — OR license allows Apache-2.0. Exception added in deny.toml.
- **Why:** Using the crate under Apache-2.0 is explicitly permitted. This is a proprietary project
  using Apache-2.0 terms. No action needed.

### BSL-1.0 (clipboard-win) not in deny.toml allowlist
- **Finding:** clipboard-win 5.4.1 uses BSL-1.0 (Boost Software License 1.0); transitive via
  bevy_egui -> arboard -> clipboard-win (Windows clipboard backend).
- **Status:** OPEN — deny.toml does not allow BSL-1.0; cargo deny check licenses fails
- **Compliance note:** BSL-1.0 is OSI-approved and FSF Free/Libre. It imposes no copyleft, no
  attribution requirement. It is safe for proprietary use.
- **Fix:** Add `"BSL-1.0"` to the `allow` array in `deny.toml`. This is a cosmetic fix; the license
  is not a compliance risk.

### MIT-0 (encase) not in deny.toml allowlist
- **Finding:** encase 0.12.0 uses MIT-0 (MIT No Attribution); transitive via bevy (render pipeline).
- **Status:** OPEN — deny.toml does not allow MIT-0; cargo deny check licenses fails
- **Compliance note:** MIT-0 is a public-domain-equivalent license (no attribution required). It is
  strictly more permissive than MIT. Safe for proprietary use.
- **Fix:** Add `"MIT-0"` to the `allow` array in `deny.toml`.

### Workspace crates missing license field
- **Finding:** breaker, breaker_derive, breaker_scenario_runner have no `license` field in their
  Cargo.toml files; cargo deny reports them as unlicensed.
- **Status:** OPEN — this causes cargo deny check licenses to fail with errors
- **Fix:** Add `license = "LicenseRef-Proprietary"` (SPDX non-standard) or simply
  `license = "UNLICENSED"` to each workspace member's Cargo.toml, reflecting that this is a
  proprietary, not-for-distribution codebase. Alternatively, configure deny.toml to exclude
  workspace path deps from license checks using `private.ignore = true`.
- **Recommended fix:** Add `[licenses] private.ignore = true` to deny.toml — this is the canonical
  cargo-deny approach for workspace-internal crates that aren't published to crates.io.
