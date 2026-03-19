---
name: doc_markdown clippy — backticks required
description: Workspace enforces clippy::pedantic which denies doc_markdown. All CamelCase type names and field names in doc comments must be wrapped in backticks.
type: feedback
---

The workspace has `clippy::pedantic = "deny"` which includes `clippy::doc_markdown`. This means:

- CamelCase type names in doc comments MUST have backticks: `ChaosDriver`, `ScriptedInput`, `Vec`, `AppExit`, etc.
- Struct field names in doc comments MUST have backticks: `action_prob`, `max_frames`, `is_active`, etc.
- Variant names listed inline in text MUST use backticks: `[MoveLeft, MoveRight]` → `` `[MoveLeft, MoveRight]` ``
- This applies to ALL doc comments: `//!` module docs, `///` item docs, and test doc comments

**Common mistakes to avoid:**
```rust
// WRONG — will fail clippy::doc_markdown:
/// When frame equals max_frames, AppExit is sent.
/// ScenarioFrame increments each tick.
/// Returns empty Vec when is_active is false.

// RIGHT:
/// When frame equals `max_frames`, `AppExit` is sent.
/// [`ScenarioFrame`] increments each tick.
/// Returns empty `Vec` when `is_active` is false.
```

**Also**: `clippy::nursery` warns `derive_partial_eq_without_eq` — if you derive `PartialEq` on a type where all fields implement `Eq`, also derive `Eq`.
