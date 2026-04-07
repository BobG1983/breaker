---
name: Drop-to-discard pattern
description: `drop(expr)` is used intentionally in scenario runner to silence #[must_use] on Result/io::Result without ? or unwrap
type: feedback
---

The scenario runner codebase intentionally uses `drop(result)` to explicitly discard results from fallible operations where failure is a deliberate silent no-op (e.g., log writes on disconnected channel, fs cleanup in tests). This is not a code smell in this context — it is preferred over `let _ =` for `Result` types with `#[must_use]`.

**Why:** The project uses `#[must_use]` aggressively. `drop()` is the explicit signal that "I know this returns a Result and I am intentionally not handling it." `let _ =` achieves the same but is less visually prominent.

**How to apply:** Do not flag `drop(expr)` where the surrounding comment or function name makes the intent clear (e.g., cleanup in test teardown, log sends after shutdown). Only flag if no rationale is visible.
