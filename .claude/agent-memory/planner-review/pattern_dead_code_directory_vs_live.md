---
name: Dead code directory vs live directory confusion
description: When a module rename creates a new directory but lib.rs still declares the old one, specs targeting the new directory will fail because Rust doesn't compile undeclared modules
type: feedback
---

When a multi-wave refactor renames a module directory (e.g., `behaviors/` -> `effect/`), the new directory may exist on disk but remain dead code if `lib.rs` (or the parent `mod.rs`) still declares the old module name.

**Why:** Specs that reference the new directory's file paths will cause writer-tests to produce code that doesn't compile — Rust can't find `crate::effect::*` if `lib.rs` only has `pub mod behaviors;`.

**How to apply:** When reviewing specs that target a recently-renamed directory:
1. Check `lib.rs` for which module is actually declared
2. Check `game.rs` for which plugin is actually wired in
3. Verify the new directory is not just a prepared copy waiting for the rename wave to complete
4. If both directories exist, the spec MUST state which is the target and list B12a completion as a prerequisite
