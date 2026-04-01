---
name: Test/impl spec path mismatches
description: Parallel test and impl specs for the same feature commonly disagree on which file tests go in
type: feedback
---

When a test spec says "Tests go in: tests.rs" and the impl spec says "Failing Tests: tests/fire_tests.rs", these are structurally incompatible — a Rust module can resolve to either a flat file OR a directory module, not both.

**Why:** The module declaration `#[cfg(test)] mod tests;` resolves to `tests.rs` OR `tests/mod.rs`. If the intent is a directory module, `tests/mod.rs` must also be listed and both specs must agree on `tests/fire_tests.rs`.

**How to apply:** On every spec pair, compare the test file path in the test spec Constraints section against the Failing Tests section of the impl spec. Flag any disagreement as BLOCKING before writer-tests launches.

Observed in: wave1b (gravity_well) and wave1c (spawn_phantom) — both had the same split.
