# ChainBolt

## Config
```rust
struct ChainBoltConfig {
    /// Maximum distance between the two chained bolts
    tether_distance: f32,
}
```
**RON**: `ChainBolt(tether_distance: 120.0)`

## Reversible: YES (despawns chain bolts tied to source)

## Target: Bolt (spawns a tethered pair from bolt's position)

## Components
```rust
#[derive(Component)]
struct ChainBoltMarker(Entity);  // on spawned bolt — points to source entity

#[derive(Component)]
struct ChainBoltAnchor;  // on source entity — marks it as a chain origin
```

## Fire
1. Read source entity's position and bolt definition
2. Insert `ChainBoltAnchor` on source entity
3. Spawn new bolt at source position with random velocity
4. Insert `ChainBoltMarker(source_entity)` on spawned bolt
5. Apply `DistanceConstraint` between the two bolts (from rantzsoft_physics2d)
6. The constraint keeps them within `tether_distance` of each other

## Reverse
1. Query all entities with `ChainBoltMarker` pointing to this source entity
2. Despawn them
3. Remove `ChainBoltAnchor` from source entity

## Notes
- Creates a tethered bolt pair — two bolts connected by a distance constraint
- The tether beam (TetherBeam effect) can add damaging beam between tethered bolts
- Spawned bolt is `ExtraBolt` tagged
- Distance constraint is a physics2d primitive — enforced each physics tick
- Reverse is meaningful — it cleans up the tethered bolt
