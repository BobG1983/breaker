---
name: Domain Inventory — Bolt, Breaker, Chips, Effect
description: Key types, queries, and system locations for the bolt, breaker, chips, and effect domains — used for spec writing
type: project
---

## Bolt Domain

**Components** (`src/bolt/components.rs`): Bolt, BoltServing, BoltBaseSpeed, BoltMinSpeed, BoltMaxSpeed, BoltRadius, BoltSpawnOffsetY, BoltRespawnOffsetY, BoltRespawnAngleSpread, BoltInitialAngle, ExtraBolt, SpawnedByEvolution(String), BoltLifespan(Timer)

**Queries** (`src/bolt/queries.rs`):
- `CollisionQueryBolt` — 10-field tuple, used by bolt_cell_collision, bolt_wall_collision, bolt_breaker_collision
- `ResetBoltQuery` — 6-field tuple, used by reset_bolt
- `LostQuery` — 8-field tuple, used by bolt_lost detection

**Key Systems**:
- `bolt_cell_collision` — `src/bolt/systems/bolt_cell_collision/system.rs` (CCD sweep, damage formula, pierce logic)
- `bolt_wall_collision` — `src/bolt/systems/bolt_wall_collision.rs` (overlap detection, pierce reset)
- `bolt_breaker_collision` — `src/bolt/systems/bolt_breaker_collision/system.rs` (CCD sweep, reflection, pierce reset)
- `prepare_bolt_velocity` — `src/bolt/systems/prepare_bolt_velocity/system.rs` (speed clamping, min angle)
- `reset_bolt` — `src/bolt/systems/reset_bolt/system.rs` (node start reset)

**Constants**: `BASE_BOLT_DAMAGE = 10.0` in `src/bolt/resources.rs`

## Breaker Domain

**Queries** (`src/breaker/queries.rs`):
- `CollisionQueryBreaker` — 8-field tuple (includes WidthBoost, EntityScale)
- `MovementQuery` — 10-field tuple (includes BreakerSpeedBoost, WidthBoost)
- `WidthBoostVisualQuery` — 5-field tuple (includes WidthBoost, EntityScale, Scale2D)
- `DashQuery`, `ResetQuery`, `BumpTimingQuery`, `BumpGradingQuery` — not affected by migration

**Key Systems**:
- `move_breaker` — `src/breaker/systems/move_breaker.rs` (speed formula, width clamping)
- `width_boost_visual` — `src/breaker/systems/width_boost_visual.rs` (Scale2D from width)

## Chips Domain

**Components** (`src/chips/components.rs`): Piercing, PiercingRemaining, DamageBoost, BoltSpeedBoost, ChainHit, BoltSizeBoost, WidthBoost, BreakerSpeedBoost, BumpForceBoost

## Effect Domain

**Active* Components** (in respective effect files):
- `ActiveDamageBoosts(Vec<f32>)` — `src/effect/effects/damage_boost.rs`
- `ActiveSpeedBoosts(Vec<f32>)` — `src/effect/effects/speed_boost.rs`
- `ActiveSizeBoosts(Vec<f32>)` — `src/effect/effects/size_boost.rs`
- `ActivePiercings(Vec<u32>)` — `src/effect/effects/piercing.rs`
- `ActiveBumpForces(Vec<f32>)` — `src/effect/effects/bump_force.rs`

Each has `fire()`, `reverse()`, `register()`, `multiplier()`/`total()` method, and a placeholder `recalculate_*` system.

**Why:** Fast lookup during spec writing for any feature touching bolt/breaker/chips/effect.
**How to apply:** Reference this when planning migrations, identifying shared prerequisites, or naming types in specs.
