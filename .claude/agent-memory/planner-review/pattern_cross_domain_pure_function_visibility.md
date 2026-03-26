---
name: Pure functions used cross-domain need explicit visibility and export path
description: When a pure function in domain A is called by domain B's system, the spec must specify pub visibility and the mod.rs export chain, or it will be invisible at compile time.
type: feedback
---

Pure functions (not Bevy systems) that are called from another domain's systems need explicit visibility planning. Example: `select_highlights` in `run/systems/select_highlights.rs` called by `screen/run_end/systems/spawn_run_end_screen.rs`.

**Why:** Domain systems files default to `pub(crate)` visibility. If a function is only `pub(crate)`, it's accessible within `breaker-game` crate but the module path must be exported through `mod.rs` → `systems/mod.rs`. If the function is in a private module, the caller can't reach it.

**How to apply:** When a spec introduces a pure function consumed by another domain, verify:
1. The function is at least `pub(crate)`
2. The module chain (`systems/mod.rs`, domain `mod.rs`) re-exports it
3. The implementation spec explicitly states the export path
