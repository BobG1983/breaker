---
name: bolt-domain-inventory
description: Bolt domain types, systems, query aliases, collision patterns, and test structure
type: project
---

## Bolt Domain — `breaker-game/src/bolt/`

### Key Types (components/definitions.rs)
- `Bolt` — marker component, requires Spatial2D + InterpolateTransform2D + Velocity2D
- `PrimaryBolt` — marker for the single primary bolt (set by builder `.primary()`)
- `BoltServing` — marker for serving state
- `BoltRadius(f32)` — collision radius (inserted by builder `.config(&config)`)
- `BoltSpawnOffsetY`, `BoltAngleSpread` — bolt spawn config, inserted by builder (NOTE: `BoltRespawnOffsetY`, `BoltRespawnAngleSpread`, `BoltInitialAngle` were DELETED in Wave 6 of feature/breaker-builder-pattern)
- Speed params are NOW `BaseSpeed`, `MinSpeed`, `MaxSpeed`, `MinAngleH`, `MinAngleV` from `rantzsoft_spatial2d` — NOT `BoltBaseSpeed`/`BoltMinSpeed`/`BoltMaxSpeed`
- `PiercingRemaining(u32)` — decremented per pierce, reset on wall/breaker contact
- `ExtraBolt` — marker for non-baseline bolts (despawned on loss, not respawned)
- `SpawnedByEvolution(String)` — damage attribution
- `BoltLifespan(Timer)` — auto-despawn countdown
- `LastImpact { position: Vec2, side: ImpactSide }` — stamped on rebound
- `ImpactSide { Top, Bottom, Left, Right }` — which surface the bolt hit

### Query Types (queries.rs)
- `BoltCollisionData` — `#[derive(QueryData)]` named struct used by collision systems. Composes `SpatialData` (from rantzsoft_spatial2d) + `BoltCollisionParams` (radius, piercings, damage/speed boosts, scale, attribution, last_impact)
- `ResetBoltData` — `#[derive(QueryData)]` named struct used by reset_bolt
- `LostBoltData` — `#[derive(QueryData)]` named struct used by bolt_lost
- `apply_velocity_formula(spatial, active_speed_boosts)` — free function; single source of truth for bolt speed clamping

**NOTE: `CollisionQueryBolt` (old tuple alias), `EffectivePiercing`, `EffectiveDamageMultiplier`,
`EffectiveSpeedMultiplier`, and all `recalculate_*` systems are GONE (cache-removal refactor).**

### Collision Systems
All in `systems/` as directory modules (mod.rs + system.rs + tests/ or tests.rs):
- `bolt_breaker_collision` — CCD sweep + overlap resolution, reflects via BreakerSurface::reflect_top_hit, sends BoltImpactBreaker, resets PiercingRemaining
- `bolt_cell_collision` — CCD loop (up to MAX_BOUNCES=4), sends BoltImpactCell + DamageCell, handles piercing pass-through
- `bolt_wall_collision` — overlap detection via quadtree, reflects off nearest face, sends BoltImpactWall, resets PiercingRemaining

### System Sets (sets.rs)
BoltSystems: Reset, CellCollision, WallCollision, BreakerCollision, BoltLost

**NOTE: `InitParams` and `PrepareVelocity` set variants are GONE** (eliminated in builder migration).
`prepare_bolt_velocity` system is GONE — velocity clamping is now inline via `apply_velocity_formula()`
at each mutation site (collision, bolt_lost, launch_bolt, reset_bolt).

### Scheduling (plugin.rs)
- OnEnter(GameState::Playing): spawn_bolt (exclusive world fn), apply_entity_scale_to_bolt, reset_bolt
- FixedUpdate under NodeState::Playing
- Order: CellCollision → WallCollision (after Cell), BreakerCollision (after Cell) → clamp → BoltLost

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
