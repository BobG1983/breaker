---
name: bolt-domain-inventory
description: Bolt domain types, systems, query aliases, collision patterns, and test structure
type: project
---

## Bolt Domain — `breaker-game/src/bolt/`

### Key Types (components/definitions.rs)
- `Bolt` — marker component, requires Spatial2D + InterpolateTransform2D + Velocity2D
- `BoltServing` — marker for serving state
- `BoltBaseSpeed(f32)`, `BoltMinSpeed(f32)`, `BoltMaxSpeed(f32)` — speed params
- `BoltRadius(f32)` — collision radius
- `PiercingRemaining(u32)` — decremented per pierce, reset on wall/breaker contact
- `ExtraBolt` — marker for non-baseline bolts (despawned on loss, not respawned)
- `SpawnedByEvolution(String)` — damage attribution
- `BoltLifespan(Timer)` — auto-despawn countdown
- `LastImpact { position: Vec2, side: ImpactSide }` — stamped on rebound
- `ImpactSide { Top, Bottom, Left, Right }` — which surface the bolt hit
- `enforce_min_angle(velocity: &mut Vec2, min_angle: f32)` — free function

### Query Aliases (queries.rs)
- `CollisionQueryBolt` — 10-element tuple used by all 3 collision systems. Entity + mut Position2D + mut Velocity2D + BoltBaseSpeed + BoltRadius + Option<mut PiercingRemaining> + Option<EffectivePiercing> + Option<EffectiveDamageMultiplier> + Option<EntityScale> + Option<SpawnedByEvolution>
- `ResetBoltQuery` — used by reset_bolt
- `LostQuery` — used by bolt_lost

### Collision Systems
All in `systems/` as directory modules (mod.rs + system.rs + tests/ or tests.rs):
- `bolt_breaker_collision` — CCD sweep + overlap resolution, reflects via BreakerSurface::reflect_top_hit, sends BoltImpactBreaker, resets PiercingRemaining
- `bolt_cell_collision` — CCD loop (up to MAX_BOUNCES=4), sends BoltImpactCell + DamageCell, handles piercing pass-through
- `bolt_wall_collision` — overlap detection via quadtree, reflects off nearest face, sends BoltImpactWall, resets PiercingRemaining

### System Sets (sets.rs)
BoltSystems: InitParams, PrepareVelocity, Reset, CellCollision, WallCollision, BreakerCollision, BoltLost

### Scheduling (plugin.rs)
- FixedUpdate under PlayingState::Active
- Order: PrepareVelocity → CellCollision → WallCollision (after Cell), BreakerCollision (after Cell) → clamp → BoltLost

### Test Structure
- bolt_breaker_collision/tests/ — mod.rs with helpers + collision + message_content + reflection submodules
- bolt_wall_collision/tests.rs — single test file with inline helpers
- bolt_cell_collision/tests/ — mod.rs with helpers + aabb_collision + attribution + basic_collision + damage_messages + piercing + reflection

### Test Patterns
- Each test file has a `test_app()` helper that builds MinimalPlugins + RantzPhysics2dPlugin + message registration + system under test
- `tick()` helper accumulates one fixed timestep of overstep then calls app.update()
- spawn_bolt / spawn_cell / spawn_wall helpers create entities with required components
- Message capture via Resource + collector system pattern (e.g., WallHitMessages + collect_wall_hits)
- Component assertions via `app.world().get::<T>(entity)`
