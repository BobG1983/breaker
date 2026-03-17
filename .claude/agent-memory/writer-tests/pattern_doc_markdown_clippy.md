---
name: doc_markdown clippy — backticks required
description: Workspace enforces clippy::pedantic which denies doc_markdown. All CamelCase type names and field names in doc comments must be wrapped in backticks.
type: feedback
---

The workspace has `clippy::pedantic = "deny"` which includes `clippy::doc_markdown`. This means:

- CamelCase type names in doc comments MUST have backticks: `ChaosMonkey`, `ScriptedInput`, `Vec`, etc.
- Struct field names in doc comments MUST have backticks: `action_prob`, `is_active`, etc.
- Variant names listed inline in text MUST have backticks: `[MoveLeft, MoveRight]` → `` `[MoveLeft, MoveRight]` ``
- This applies to ALL doc comments: `//!` module docs, `///` item docs, and test doc comments

**Common mistakes to avoid:**
```rust
// WRONG — will fail clippy::doc_markdown:
/// ChaosMonkey fires on action_prob frames.
/// Returns empty Vec when is_active is false.
/// Actions are one of [MoveLeft, MoveRight, Bump].

// RIGHT:
/// `ChaosMonkey` fires on `action_prob` frames.
/// Returns empty `Vec` when `is_active` is false.
/// Actions are one of `[MoveLeft, MoveRight, Bump]`.
```

Also applies to `//!` module-level docs at the top of files:
```rust
// WRONG:
//! Input strategies — ChaosMonkey and ScriptedInput.

// RIGHT:
//! Input strategies — `ChaosMonkey` and `ScriptedInput`.
```

**Also**: `clippy::nursery` warns `derive_partial_eq_without_eq` — if you derive `PartialEq` on a type where all fields implement `Eq`, also derive `Eq`.
