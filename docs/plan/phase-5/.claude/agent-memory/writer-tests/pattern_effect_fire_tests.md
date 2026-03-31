---
name: Effect fire() test pattern
description: How to write tests for effect fire() functions — bare World, direct call, query for components
type: feedback
---

Effect fire() tests use a **bare World pattern** (no App, no MinimalPlugins):
1. `let mut world = World::new();`
2. Spawn owner entity with `Position2D`
3. Call `fire(entity, ...)` directly on `&mut World`
4. Query for spawned components and assert

Import pattern for tests in `effects/<effect>/tests/<file>.rs`:
```rust
use super::super::effect::*;
```
This brings in all pub items from the effect module.

**Why:** fire() operates on a raw World pointer, not through systems. No scheduling or App needed.

**How to apply:** When writing tests for any effect's fire() function, follow this pattern. No test_app() needed — that's only for system-level tests (tick, pull, etc.).

Reference files:
- `breaker-game/src/effect/effects/shockwave/tests/fire_tests.rs`
- `breaker-game/src/effect/effects/gravity_well/tests/fire_tests.rs`
