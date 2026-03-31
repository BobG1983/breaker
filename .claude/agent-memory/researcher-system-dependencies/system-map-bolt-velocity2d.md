---
name: Bolt Velocity2D system map
description: All systems that read or write Velocity2D on bolt entities, their sets, and ordering relative to BoltSystems::PrepareVelocity
type: project
---

# Bolt Velocity2D System Map

Bevy 0.18. Analyzed from breaker-game/src/bolt/ and breaker-game/src/effect/effects/.

## Systems That Write &mut Velocity2D on Bolt Entities

| System | File | Set | Writes Velocity2D? |
|---|---|---|---|
| `enforce_distance_constraints` | rantzsoft_physics2d/src/systems/enforce_distance_constraints.rs | `PhysicsSystems::EnforceDistanceConstraints` | YES (any entity with Velocity2D+Position2D, no Bolt filter) |
| `apply_gravity_pull` | effect/effects/gravity_well/effect.rs | (none, bare system) | YES — `With<Bolt>` filter |
| `apply_attraction` | effect/effects/attraction/effect.rs | (none, bare system) | YES — entities with ActiveAttractions |
| `bolt_cell_collision` | bolt/systems/bolt_cell_collision/system.rs | `BoltSystems::CellCollision` | YES via CollisionQueryBolt |
| `bolt_wall_collision` | bolt/systems/bolt_wall_collision/system.rs | `BoltSystems::WallCollision` | YES via CollisionQueryBolt |
| `bolt_breaker_collision` | bolt/systems/bolt_breaker_collision/system.rs | `BoltSystems::BreakerCollision` | YES via CollisionQueryBolt |
| `clamp_bolt_to_playfield` | bolt/systems/clamp_bolt_to_playfield/system.rs | (none, bare system) | YES — ActiveFilter |
| `prepare_bolt_velocity` | bolt/systems/prepare_bolt_velocity/system.rs | `BoltSystems::PrepareVelocity` | YES — ActiveFilter |
| `launch_bolt` | bolt/systems/launch_bolt.rs | (none, bare system) | YES — ServingFilter only |

## Ordering constraints (FixedUpdate only)

### BoltPlugin (plugin.rs lines 56–91)
- `bolt_cell_collision` .after(PhysicsSystems::EnforceDistanceConstraints) .after(BreakerSystems::Move) .after(PhysicsSystems::MaintainQuadtree) .in_set(BoltSystems::CellCollision)
- `bolt_wall_collision` .after(BoltSystems::CellCollision) .in_set(BoltSystems::WallCollision)
- `bolt_breaker_collision` .after(BoltSystems::CellCollision) .in_set(BoltSystems::BreakerCollision)
- `clamp_bolt_to_playfield` .after(bolt_breaker_collision)
- `prepare_bolt_velocity` .in_set(BoltSystems::PrepareVelocity) .after(BoltSystems::BreakerCollision) .after(BoltSystems::WallCollision)
- `launch_bolt` — no ordering constraints (only fires on ServingFilter entities)

### gravity_well/effect.rs register() line 155
- `apply_gravity_pull` .before(BoltSystems::PrepareVelocity) — ORDERED (runs before PrepareVelocity)
- No ordering relative to collision sets — UNORDERED relative to CellCollision/WallCollision/BreakerCollision

### attraction/effect.rs register() lines 205-208
- `apply_attraction` .after(PhysicsSystems::MaintainQuadtree) .before(BoltSystems::PrepareVelocity) — ORDERED (runs before PrepareVelocity)
- No ordering relative to collision sets — UNORDERED relative to CellCollision/WallCollision/BreakerCollision

## Known Concerns

- `apply_gravity_pull` and `apply_attraction` have no ordering relative to the three collision sets (CellCollision, WallCollision, BreakerCollision). They could run before OR after collisions in the same tick.
- `enforce_distance_constraints` has no ordering relative to `MaintainQuadtree` within RantzPhysics2dPlugin (same add_systems call, no .before/.after between them).
- `launch_bolt` has no ordering constraints but is safe — it only writes on ServingFilter (BoltServing) entities; once launched, BoltServing is removed, so active-flight bolts are never touched by it.
- `mirror_protocol::fire` writes Velocity2D on newly spawned bolts via world.entity_mut().insert() — this is a one-time spawn operation, not a repeating system. Not in scope for ordering analysis.
