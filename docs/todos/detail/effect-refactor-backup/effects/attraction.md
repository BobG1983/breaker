# Attraction

## Config
```rust
struct AttractionConfig {
    /// Which entity type to attract toward
    attraction_type: AttractionType,  // Cell, Wall, Breaker
    /// Attraction strength
    force: f32,
    /// Optional maximum force magnitude per tick (clamps velocity delta)
    max_force: Option<f32>,
}
```
**RON**: `Attraction(attraction_type: Cell, force: 100.0, max_force: Some(50.0))`

## Reversible: YES

## Target: Bolt

## Components
```rust
#[derive(Component, Default)]
struct ActiveAttractions(Vec<AttractionEntry>);

struct AttractionEntry {
    attraction_type: AttractionType,
    force: f32,
    max_force: Option<f32>,
    active: bool,  // deactivates on hit with attracted type
}
```

## Fire
1. Create entry with `active: true`
2. If `ActiveAttractions` exists, push entry
3. If absent, insert new `ActiveAttractions` with single entry

## Reverse
1. Find first matching entry (attraction_type + force + max_force all match)
2. Remove it from Vec

## Runtime System: `apply_attraction`
**Schedule**: FixedUpdate, after PhysicsSystems::MaintainQuadtree, run_if NodeState::Playing

1. For each entity with `ActiveAttractions`, skip if no active entries
2. For each active entry, query quadtree for nearest entity of the attracted type (search radius 500.0)
3. Track overall nearest candidate across all attraction types
4. Steer entity toward nearest: `velocity += direction * effective_force * dt`, then normalize and apply velocity formula

## Runtime System: `manage_attraction_types`
**Schedule**: FixedUpdate, run_if NodeState::Playing

Reads `BoltImpactCell`, `BoltImpactWall`, `BoltImpactBreaker` messages:
- If bolt has an attraction entry for the impact type → **deactivate** all entries of that type
- If bolt does NOT have an entry for the impact type → **reactivate** all deactivated entries

This creates the "lock on, hit, re-target" gameplay loop.

## Notes
- Attraction deactivation/reactivation on impact is the key mechanic — bolt steers toward target, hits it, then steers toward next target
- Uses quadtree spatial query for nearest-entity lookup
- max_force caps the per-tick steering delta to prevent instant turns
