# Shield

## Config
```rust
struct ShieldConfig {
    /// Duration in seconds before the wall despawns
    duration: f32,
    /// Time subtracted from ShieldWallTimer per bolt reflection
    reflection_cost: f32,
}
```
**RON**: `Shield(duration: 3.0, reflection_cost: 0.5)`

## Reversible: YES (despawns the wall)

## Target: Any (entity-independent — spawns global floor wall)

## Components (on spawned wall entity)
```rust
#[derive(Component)]
struct ShieldWall;  // marker

#[derive(Component)]
struct ShieldWallTimer(Timer);  // countdown timer

#[derive(Component)]
struct ShieldReflectionCost(f32);  // time subtracted per reflection
```

## Fire
1. If `ShieldWall` entity already exists → reset its timer to `duration` and return
2. Read `PlayfieldConfig` for floor position
3. Spawn wall entity using `Wall::builder().floor().with_half_thickness().timed()` 
4. Add `ShieldWall`, `ShieldWallTimer`, `ShieldReflectionCost`, visual mesh

## Reverse
1. Query all `ShieldWall` entities
2. Despawn them all

## Runtime System: `tick_shield_wall_timer`
**Schedule**: FixedUpdate (always — not gated by NodeState)

1. Tick each `ShieldWallTimer` by delta time
2. When timer finishes → despawn the wall entity

## Runtime System: `deduct_shield_on_reflection`
**Schedule**: FixedUpdate, after BoltSystems::WallCollision

1. Read `BoltImpactWall` messages
2. If impacted wall has `ShieldWall` → subtract `ShieldReflectionCost` from `ShieldWallTimer`
3. If remaining time <= 0 → set timer elapsed to full (triggers despawn next tick)

## Notes
- Only one shield wall can exist at a time (fire resets timer if existing)
- Shield condition monitor watches for `Added<ShieldWall>` and `RemovedComponents<ShieldWall>` to drive `During(ShieldActive, ...)` effects
- Reflection cost creates "shield burns out faster under heavy use" gameplay
