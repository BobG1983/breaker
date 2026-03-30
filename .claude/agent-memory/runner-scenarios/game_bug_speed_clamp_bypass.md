---
name: BoltSpeedInRange violations from velocity-modifying systems bypassing speed clamp
description: Systems that modify Velocity2D after prepare_bolt_velocity has clamped speed can push bolt speed above BoltMaxSpeed
type: project
---

Two confirmed game bugs where velocity-modifying systems bypass the speed clamp:

## Pattern

`prepare_bolt_velocity` (in `BoltSystems::PrepareVelocity`) clamps `Velocity2D` to `[BoltMinSpeed, BoltMaxSpeed]`. Any system that modifies `Velocity2D` *after* this clamp runs can push bolt speed out of range, causing `BoltSpeedInRange` violations.

## Bug 1: gravity_well.rs — apply_gravity_pull ordering

`apply_gravity_pull` adds a force vector to `Velocity2D` in FixedUpdate with no ordering constraint relative to `BoltSystems::PrepareVelocity`. When it runs after `prepare_bolt_velocity`, the gravitational pull increases bolt speed above max.

- File: `breaker-game/src/effect/effects/gravity_well.rs`
- System: `apply_gravity_pull`
- Fix needed: Add `.before(BoltSystems::PrepareVelocity)` constraint so clamp runs after the pull

## Bug 2: enforce_distance_constraints — velocity redistribution ordering

`enforce_distance_constraints` (in `PhysicsSystems::EnforceDistanceConstraints`, `rantzsoft_physics2d/src/systems/enforce_distance_constraints.rs`) redistributes `Velocity2D` along the tether axis. This redistribution can change bolt speed magnitude. The system has no ordering constraint relative to `BoltSystems::PrepareVelocity`.

- File: `rantzsoft_physics2d/src/systems/enforce_distance_constraints.rs` + `rantzsoft_physics2d/src/plugin.rs`
- Fix needed: Either ensure `prepare_bolt_velocity` runs after `PhysicsSystems::EnforceDistanceConstraints`, or add a speed re-clamp after constraint enforcement
- Affects: `tether_chain_bolt_stress` scenario

## Detection

Both bugs fail in `--all` runs. Both fail individually (`-s scenario_name`). Not load-dependent.
Confirmed 2026-03-30.
