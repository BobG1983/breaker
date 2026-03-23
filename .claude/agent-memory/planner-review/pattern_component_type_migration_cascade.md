---
name: Component type migration cascade
description: Migrating an entity from one position type to another (e.g., Transform to Position2D) cascades through query type aliases, test construction sites, scenario runner invariant checkers, and debug/effect systems that read the old type
type: feedback
---

When migrating an entity's canonical position from one component type to another (e.g., Transform.translation -> Position2D), the spec must address:

1. **Query type aliases**: every `type FooQuery = (...)` that included the old type must be updated
2. **Test construction sites**: every test that spawns the entity with the old type must update — this includes test helpers, spawn bundles, and assertion queries
3. **Scenario runner**: invariant checkers and frame mutations (e.g., MoveBolt) that write the old type directly will silently break — the new type's propagation system will overwrite the mutation
4. **Cross-crate breakage**: if the type alias is `pub(crate)` and used by the scenario runner via re-export, the runner crate must also update
5. **Required components**: marker types like `Spatial2D` auto-insert 8+ components — specs must verify non-obvious ones (PreviousPosition, propagation modes)
6. **Type dimension mismatch**: Vec2 vs Vec3 — every Z-coordinate handling pattern must be audited (old: preserve Z explicitly; new: DrawLayer handles Z)
7. **Timing**: propagation systems run in AfterFixedMainLoop, but if a system writes the new type in Update, the propagated value is one frame stale

**Why:** A Position2D migration for bolt touched 8+ system files, 8+ test files, 2 query alias files, the scenario runner's MoveBolt mutation, and 3 debug/effect read sites. The spec initially missed the query aliases, test helpers, and scenario runner regression.

**How to apply:** When reviewing any component type migration spec, demand an explicit enumeration of: (a) all query type aliases referencing the old type, (b) all test files that construct entities with the old type, (c) all scenario runner systems that read/write the old type, (d) all systems outside the migration scope that still read the old type (and confirm they work via propagation).
