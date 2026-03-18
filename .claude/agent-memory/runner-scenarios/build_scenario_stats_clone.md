---
name: ScenarioStats missing Clone derive (resolved)
description: Build failure — ScenarioStats was missing #[derive(Clone)]; fixed as of 2026-03-17. Kept as reference for the pattern.
type: project
---

`breaker-scenario-runner/src/invariants.rs` defines `ScenarioStats`. When the health-warning summary feature was added, `.cloned()` was called on the resource in `runner.rs` but `Clone` was not added to the derive list, causing compiler error `E0277`.

**Status:** Fixed as of 2026-03-17. All 27 scenarios pass cleanly.

**Why recorded:** The pattern is worth remembering — when scenario runner structs gain new usages (cloning for summary reports), check derives. Future resource structs used in summary/cloning contexts need `Clone`.

**How to apply:** If a future scenario build fails with `E0277` on a resource struct, check whether a new summary/reporting feature added a `.cloned()` or `Clone` bound without updating the derive list.
