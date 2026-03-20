---
name: known_unsafe_blocks
description: Inventory of unsafe code in the codebase — confirmed none as of 2026-03-19
type: project
---

Audited 2026-03-19 (develop, commit 7986274).

## Result: No unsafe blocks found

All three crates have `unsafe_code = "deny"` in `[workspace.lints.rust]` (root Cargo.toml).
A grep for the literal string `unsafe` across all source files returns zero results.

No unsafe blocks, no FFI boundaries, no raw pointer manipulation in any first-party code.

**How to apply:** On future audits, re-grep for `unsafe` — any new result is a new finding that needs review.
