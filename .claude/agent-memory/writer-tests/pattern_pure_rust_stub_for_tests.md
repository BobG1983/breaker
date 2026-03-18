---
name: Pure Rust stub pattern for test-first modules
description: How to write compilable stubs with todo!() so tests fail correctly for pure-Rust (non-Bevy) modules
type: feedback
---

For pure-Rust modules (no Bevy), use `todo!()` in method bodies to create stubs that compile but make all tests fail.

**How to apply:**

1. Write the full public API surface with doc comments (required for `missing_docs = "deny"`).
2. Mark return-value methods with `#[must_use]`.
3. Use `todo!()` as the body — this panics at runtime, so tests hitting it fail with "not yet implemented".
4. Import only the types needed by the struct fields; unused rand traits (`Rng`, `SliceRandom`) cause warnings from dead-code paths in stubs. Either suppress with `_` prefixed parameter names or accept the warnings.
5. `const` arrays used only inside `todo!()` bodies will trigger `dead_code` warnings in stub phase — that's expected and acceptable.

**Why:** `unimplemented!()` and `todo!()` both panic, but `todo!()` is the idiomatic "I haven't written this yet" marker. The stub needs to compile fully (including the workspace `missing_docs` and `clippy::pedantic` lint requirements) before the writer-code implements it.
