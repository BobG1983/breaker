---
name: Refactor atomicity — multi-unit type moves break compilation
description: When a refactor moves/renames types used across multiple files, parallel units cannot compile independently because removing/renaming in one file breaks imports in another
type: feedback
---

When a spec decomposes a refactor into N parallel units and any unit removes/renames types imported by other units, all N units fail to compile independently.

**Why:** Rust compilation requires all imports to resolve. Moving `Trigger` from `definition.rs` (Unit 5) breaks `active.rs` (Unit 2), `bridges.rs` (Unit 3), etc. Each unit's tests fail to compile before the other units are applied.

**How to apply:** For any refactor that moves or renames types:
1. List ALL import sites (grep for the type name across the crate + scenario runner)
2. If any import site is outside the unit that creates the new type, the units cannot be parallel
3. Fix options: (a) single atomic unit, (b) two-phase with re-export bridge, (c) re-order units sequentially with each compiling on top of the previous
4. Always check `breaker-scenario-runner/` for cross-crate imports of `pub` types
