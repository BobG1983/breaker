---
name: Bolt Velocity2D system map
description: All systems that read or write Velocity2D on bolt entities, their sets, and ordering — updated for builder migration (feature/chip-evolution-ecosystem); prepare_bolt_velocity and BoltSystems::PrepareVelocity eliminated
type: project
---

# Bolt Velocity2D System Map

Bevy 0.18. Analyzed from breaker-game/src/bolt/ and breaker-game/src/effect/effects/.

**NOTE (feature/chip-evolution-ecosystem): `prepare_bolt_velocity` system and
`BoltSystems::PrepareVelocity` set are ELIMINATED. Velocity clamping is now inline via
`apply_velocity_formula()` in `bolt/queries.rs` at each mutation site (collision, bolt_lost,
launch_bolt, reset_bolt). The gravity_well and attraction ordering anchors
(`.before(BoltSystems::PrepareVelocity)`) are now stale — verify current ordering in plugin.**

## Systems That Write &mut Velocity2D on Bolt Entities

| System | File | Set | Writes Velocity2D? |
|---|---|---|---|
| `enforce_distance_constraints` | rantzsoft_physics2d/src/systems/enforce_distance_constraints.rs | `PhysicsSystems::EnforceDistanceConstraints` | YES (any entity with Velocity2D+Position2D, no Bolt filter) |
| `apply_gravity_pull` | effect/effects/gravity_well/effect.rs | (none, bare system) | YES — `With<Bolt>` filter |
| `apply_attraction` | effect/effects/attraction/effect.rs | (none, bare system) | YES — entities with ActiveAttractions |
| `bolt_cell_collision` | bolt/systems/bolt_cell_collision/system.rs | `BoltSystems::CellCollision` | YES via BoltCollisionData |
| `bolt_wall_collision` | bolt/systems/bolt_wall_collision/system.rs | `BoltSystems::WallCollision` | YES via BoltCollisionData |
| `bolt_breaker_collision` | bolt/systems/bolt_breaker_collision/system.rs | `BoltSystems::BreakerCollision` | YES via BoltCollisionData |
| `clamp_bolt_to_playfield` | bolt/systems/clamp_bolt_to_playfield/system.rs | (none, bare system) | YES — ActiveFilter |
| `launch_bolt` | bolt/systems/launch_bolt.rs | (none, bare system) | YES — ServingFilter only |

## Ordering constraints (FixedUpdate only)

### BoltPlugin (plugin.rs — current state)
- `bolt_cell_collision` .after(PhysicsSystems::EnforceDistanceConstraints) .after(BreakerSystems::Move) .after(PhysicsSystems::MaintainQuadtree) .in_set(BoltSystems::CellCollision)
- `bolt_wall_collision` .after(BoltSystems::CellCollision) .in_set(BoltSystems::WallCollision)
- `bolt_breaker_collision` .after(BoltSystems::CellCollision) .in_set(BoltSystems::BreakerCollision)
- `clamp_bolt_to_playfield` .after(bolt_breaker_collision)
- `bolt_lost` .after(PhysicsSystems::EnforceDistanceConstraints) .after(clamp_bolt_to_playfield)
- `launch_bolt` — no ordering constraints (only fires on ServingFilter entities)

### gravity_well/effect.rs and attraction/effect.rs
- **WARNING**: Ordering anchors `.before(BoltSystems::PrepareVelocity)` are STALE.
  `BoltSystems::PrepareVelocity` no longer exists. Current ordering behavior of
  `apply_gravity_pull` and `apply_attraction` relative to collision sets is UNKNOWN —
  re-verify against current plugin registration before relying on this map.

## Known Concerns

- `apply_gravity_pull` and `apply_attraction` ordering relative to collision sets is UNVERIFIED
  after the builder migration (PrepareVelocity anchor no longer exists).
- `enforce_distance_constraints` has no ordering relative to `MaintainQuadtree` within RantzPhysics2dPlugin (same add_systems call, no .before/.after between them).
- `launch_bolt` has no ordering constraints but is safe — it only writes on ServingFilter (BoltServing) entities; once launched, BoltServing is removed, so active-flight bolts are never touched by it.
- `mirror_protocol::fire` writes Velocity2D on newly spawned bolts via world.entity_mut().insert() — one-time spawn operation, not a repeating system. Not in scope for ordering analysis.
