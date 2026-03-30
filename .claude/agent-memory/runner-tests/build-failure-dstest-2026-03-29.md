---
name: Build failure: dstest — private import + missing rand::Rng trait
description: Two compiler errors in breaker-scenario-runner preventing dstest from building (2026-03-29)
type: project
---

Two compiler errors in `breaker-scenario-runner` (lib):

**Error 1 — E0603 (private import)**
- File: `breaker-scenario-runner/src/lifecycle/systems/debug_setup.rs:14`
- `ScenarioTagBolt` is imported from `types::` module directly, but that re-export is private.
- Fix: change import path to use the public re-export: `crate::invariants::ScenarioTagBolt` (already re-exported via `pub use types::*` in `invariants/mod.rs`)
- Compiler hint: `import ScenarioTagBolt through the re-export` at `invariants::ScenarioTagBolt`

**Error 2 — E0599 (missing trait in scope)**
- File: `breaker-scenario-runner/src/lifecycle/systems/perfect_tracking.rs:60`
- `SmallRng::random_range` called but `rand::Rng` trait is not in scope.
- Fix: add `use rand::Rng;` at the top of `perfect_tracking.rs`

**Why:** These appear to have been introduced by a file-split refactor that moved types between modules, breaking import paths and trait visibility.

**How to apply:** Route to `researcher-rust-errors` then `writer-code` with the two fix spec hints above.
