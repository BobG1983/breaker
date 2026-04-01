---
name: Effect Module Test Structure
description: How to structure test directories for new effect modules in breaker-game, including stubs, dispatch wiring, and serde tests
type: feedback
---

## Effect Module Test Structure

New effect modules follow the `spawn_bolts` pattern:

```
src/effect/effects/<name>/
  mod.rs          -- pub(crate) use effect::*; mod effect; #[cfg(test)] mod tests;
  effect.rs       -- fire(), reverse(), register() stubs
  tests/
    mod.rs        -- mod per test file
    fire_*.rs     -- behavioral test files
    reverse.rs    -- reverse noop tests
```

**Key wiring steps (stubs only, no production logic):**
1. Add variant to `EffectKind` enum in `src/effect/core/types/definitions/enums.rs`
2. Add dispatch arms in `fire.rs` and `reverse.rs` in the same definitions directory
3. Add `pub mod <name>;` in `src/effect/effects/mod.rs` and `<name>::register(app);` in register()
4. Serde and dispatch tests go in `src/effect/core/types/tests.rs` (not in the module's test directory)

**Test imports pattern:**
```rust
use super::super::effect::*;  // imports fire(), reverse() from the module's effect.rs
```

**World setup for bolt-spawning effects:**
```rust
fn world_with_bolt_config() -> World {
    let mut world = World::new();
    world.insert_resource(BoltConfig::default());
    world.insert_resource(GameRng::default());
    world
}
```

**Why:** This structure matches existing patterns (spawn_bolts, chain_bolt) and ensures the test module hierarchy is consistent.

**How to apply:** When creating tests for any new effect module, follow this exact structure. Check `spawn_bolts/tests/` for the reference pattern.
