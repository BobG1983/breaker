---
name: known_unsafe_blocks
description: Inventory of unsafe code in the codebase — confirmed none as of 2026-03-19
type: project
---

Audited 2026-03-19 (develop, commit 7986274). Re-confirmed 2026-03-24 (spatial/physics extraction branch).

## Result: No unsafe blocks found

All six crates (including new `rantzsoft_spatial2d` and `rantzsoft_physics2d`) have `unsafe_code = "deny"` in `[workspace.lints.rust]` (root Cargo.toml, `workspace = true` in all crate Cargo.tomls).
A grep for the literal string `unsafe` across all source files returns zero results.

No unsafe blocks, no FFI boundaries, no raw pointer manipulation in any first-party code.

**How to apply:** On future audits, re-grep for `unsafe` — any new result is a new finding that needs review.
