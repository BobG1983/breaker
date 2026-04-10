# SecondWind

## Config
No config fields.

**RON**: `SecondWind`

## Reversible: YES (despawns the wall)

## Target: Any (entity-independent — spawns global floor wall)

## Components (on spawned wall entity)
```rust
#[derive(Component)]
struct SecondWindWall;  // marker
```

## Fire
1. If `SecondWindWall` entity already exists → skip (no double-spawn)
2. Spawn invisible floor wall using `Wall::builder().floor().invisible().one_shot()`
3. Add `SecondWindWall` marker

## Reverse
1. Query all `SecondWindWall` entities
2. Despawn them all

## Runtime System: `despawn_second_wind_on_contact`
**Schedule**: FixedUpdate, run_if NodeState::Playing, after BoltSystems::WallCollision

1. Read `BoltImpactWall` messages
2. If impacted wall has `SecondWindWall` → despawn it
3. Uses `Local<HashSet<Entity>>` to prevent double-despawn when two bolts hit same wall same frame

## Notes
- Single-use: bounces bolt once, then despawns
- Invisible — no visual mesh, no draw layer
- Prevents bolt loss by providing a safety net floor wall
- Only one can exist at a time (fire guards against double-spawn)
