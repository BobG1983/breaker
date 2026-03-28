---
name: bolt module directory structure
description: Established pattern for bolt system modules with tests — directory with mod.rs + system.rs + tests (file or dir)
type: project
---

The `bolt/systems/` modules that have tests follow this directory pattern:

```
bolt/systems/
  some_system/
    mod.rs      // pub(crate) use system::*; mod system; #[cfg(test)] mod tests;
    system.rs   // production code
    tests.rs    // tests if under ~800 lines
    tests/      // or a tests/ directory if 800+ test lines
      mod.rs
      helpers.rs
      concern_a.rs
      ...
```

`mod.rs` always has exactly: `pub(crate) use system::*;`, `mod system;`, and `#[cfg(test)] mod tests;` — nothing else.

Examples confirmed:
- `bolt_cell_collision/` — tests/ directory (large test suite, 8 files)
- `bolt_breaker_collision/` — tests/ directory (838 lines across 5 files)

When tests are under ~800 lines a single `tests.rs` is appropriate.
