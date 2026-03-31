---
name: Test Directory Split Pattern
description: How to convert a flat tests.rs into a tests/ directory module when adding new test files to an existing domain
type: feedback
---

## Test Directory Split Pattern

When adding a new test file to an existing domain that has a flat `tests.rs`, convert to a directory module:

1. Create `tests/mod.rs` with the exact content of old `tests.rs` plus `mod new_feature;`
2. Create `tests/new_feature.rs` with the new tests
3. Remove the old `tests.rs` (can't coexist with `tests/mod.rs`)

**Import paths in the new sub-file:**
- `super::super::system::*` to reach the system module (up through tests/, up through parent/)
- `crate::...` for anything else

**Parent mod.rs (`#[cfg(test)] mod tests;`) does NOT change** -- Rust resolves it to the directory module automatically.

**Why:** The orchestrator may ask for tests in a `tests/` directory when the existing tests.rs would exceed 400 lines with new tests added. This follows Strategy C from file-splitting rules.

**How to apply:** When instructed to create tests in a `tests/` subdirectory of an existing flat-file test module, follow this exact conversion pattern. Preserve all existing test content unchanged in mod.rs.
