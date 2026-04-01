---
name: Direct World test pattern — spawn_phantom and fire_helpers
description: Correct pattern for World-based tests calling fire() directly in this codebase
type: feedback
---

In this codebase, effect module tests use a direct World pattern (no App, no system schedule):

1. Create a bare `World::new()`
2. Insert `BoltConfig::default()` and `GameRng::from_seed(42)` as resources
3. Spawn an owner entity with `Position2D(Vec2::new(x, y))`
4. Call `fire(owner, duration, max_active, "", &mut world)` directly
5. Query with `world.query::<(ComponentA, ComponentB)>()` and `.iter(&world)` or `.iter_mut(&mut world)`

Helper functions `sorted_spawn_orders()` and `phantom_count_for_owner()` are the canonical pattern for asserting FIFO ordering in these tests (confirmed in wave1c fifo_tests.rs).

Note: `world.query()` returns a `QueryState` that requires `&mut world` for iteration, so helper functions take `&mut World` even for read-only queries.
