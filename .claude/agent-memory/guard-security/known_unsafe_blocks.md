---
name: known_unsafe_blocks
description: Inventory of unsafe blocks in the workspace and their justification status
type: project
---

## Unsafe block inventory (as of 2026-03-28)

**Result: None found.**

The workspace lint configuration in `Cargo.toml` sets:
```
unsafe_code = "deny"
unsafe_op_in_unsafe_fn = "deny"
undocumented_unsafe_blocks = "deny"
```

No unsafe blocks exist anywhere in `breaker-game/src/`. Verified by grep across all
changed files in the Phase 1 collision cleanup diff and full-source scan.

No FFI boundaries, no raw pointer manipulation, no proc macros with untrusted input.
No `build.rs` files in any crate.
