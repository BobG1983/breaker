---
name: Fireable::fire quadtree broad-phase + world.get narrow-phase pattern
description: Canonical shape for spatial Fireable configs — rantzsoft query_circle_filtered + world.get::<Component> narrow filtering is NOT a rantzsoft-crates.md violation
type: project
---

The canonical pattern for `Fireable::fire` implementations that need to target
spatially-filtered entities:

```rust
impl Fireable for FooConfig {
    fn fire(&self, entity: Entity, source: &str, world: &mut World) {
        let pos = world.get::<Position2D>(entity).map_or(Vec2::ZERO, |p| p.0);

        // Broad phase: rantzsoft_physics2d spatial query with layer mask
        let query_layers = CollisionLayers::new(0, CELL_LAYER);
        let candidates = world
            .resource::<CollisionQuadtree>()
            .quadtree
            .query_circle_filtered(pos, radius, query_layers);

        // Narrow phase: entity-component reads to enforce semantics the
        // broad phase cannot express (distance-to-center, Without<Dead>,
        // other component predicates).
        let targets: Vec<Entity> = candidates
            .into_iter()
            .filter(|&e| {
                let Some(cell_pos) = world.get::<Position2D>(e) else { return false; };
                if world.get::<Dead>(e).is_some() { return false; }
                pos.distance(cell_pos.0) <= radius
            })
            .collect();

        for target in targets { /* write messages */ }
    }
}
```

**Reference sites:**
- `breaker-game/src/effect_v3/effects/explode/config.rs:20-62` (explode —
  quadtree broad phase + Position2D/Dead narrow phase)
- `breaker-game/src/effect_v3/effects/shockwave/config.rs:34-70` (shockwave —
  `world.get::<Position2D>`, `world.get::<BoltBaseDamage>`, `world.get::<EffectStack<...>>`)

**Why this does NOT violate `rantzsoft-crates.md`:**

The "always use rantzsoft_* APIs for spatial queries" rule governs spatial
*indexing and querying*, not entity-component access. `world.get::<Component>(e)`
is an ordinary ECS read, not a spatial query. `Vec2::distance` on a pair of
already-fetched positions is scalar math, not a spatial operation.

**Why the narrow phase is usually necessary:**

1. `query_circle_filtered` uses `circle_overlaps_aabb` internally
   (`rantzsoft_physics2d/src/quadtree/tree.rs:368`) — it returns any entity
   whose **AABB overlaps the circle**. Games that want center-point distance
   semantics (`pos.distance(target) <= radius`) must add a narrow-phase filter
   or the behavior silently expands.
2. The quadtree has no hooks for ECS filter predicates like `Without<Dead>`,
   `With<SomeState>`, or `Option<&Component>` evaluation. Narrow-phase
   `world.get::<T>(e)` is the only way to enforce those predicates on
   broad-phase candidates.
3. CollisionLayers only gives you a u32 bitmask — it cannot represent
   per-entity component predicates. Anything beyond "is on layer X" must be
   narrow-phase filtering.

**When to add a new capability to rantzsoft_physics2d vs narrow-phase filter:**

- If the capability is **spatial-semantic** (different shape query, different
  distance metric, sweep, continuous collision): add it to the crate.
- If the capability is **ECS-predicate** (filter by component presence/value,
  per-entity state checks): do it in narrow phase via `world.get::<T>(e)`.
  rantzsoft_physics2d must not learn game component types.

**Related rule:** `.claude/rules/rantzsoft-crates.md` "Zero Game Knowledge" —
rantzsoft_physics2d must never reference game types. So filtering by
`With<Cell>` or `Without<Dead>` literally cannot be pushed into the quadtree
API; it has to be narrow-phase game-side code.
