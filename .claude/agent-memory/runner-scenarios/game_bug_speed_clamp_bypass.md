---
name: BoltSpeedInRange violations from velocity-modifying systems bypassing speed clamp
description: Systems that modify Velocity2D after prepare_bolt_velocity has clamped speed can push bolt speed above BoltMaxSpeed — previously confirmed bugs, now RESOLVED
type: project
---

Two confirmed game bugs where velocity-modifying systems bypass the speed clamp.
**Both confirmed resolved as of 2026-03-30 scenario run (all scenarios PASS).**

## Pattern

`prepare_bolt_velocity` (in `BoltSystems::PrepareVelocity`) clamps `Velocity2D` to `[BoltMinSpeed, BoltMaxSpeed]`. Any system that modifies `Velocity2D` *after* this clamp runs can push bolt speed out of range, causing `BoltSpeedInRange` violations.

## Bug 1: gravity_well.rs — apply_gravity_pull ordering (RESOLVED)

`apply_gravity_pull` adds a force vector to `Velocity2D` in FixedUpdate with no ordering constraint relative to `BoltSystems::PrepareVelocity`. When it runs after `prepare_bolt_velocity`, the gravitational pull increases bolt speed above max.

- File: `breaker-game/src/effect/effects/gravity_well.rs`
- System: `apply_gravity_pull`
- Fix needed: Add `.before(BoltSystems::PrepareVelocity)` constraint so clamp runs after the pull

**`gravity_well_chaos` stress: 16/16 passed** — resolved as of 2026-03-30 run

## Bug 2: enforce_distance_constraints — velocity redistribution ordering (RESOLVED)

`enforce_distance_constraints` (in `PhysicsSystems::EnforceDistanceConstraints`, `rantzsoft_physics2d/src/systems/enforce_distance_constraints.rs`) redistributes `Velocity2D` along the tether axis. This redistribution can change bolt speed magnitude. The system has no ordering constraint relative to `BoltSystems::PrepareVelocity`.

- File: `rantzsoft_physics2d/src/systems/enforce_distance_constraints.rs` + `rantzsoft_physics2d/src/plugin.rs`
- Fix needed: Either ensure `prepare_bolt_velocity` runs after `PhysicsSystems::EnforceDistanceConstraints`, or add a speed re-clamp after constraint enforcement
- Affects: `tether_chain_bolt_stress` scenario

**`tether_chain_bolt_stress` stress: 16/16 passed** — resolved as of 2026-03-30 run

## Detection

Both bugs failed in `--all` runs. Both failed individually. Not load-dependent.
Confirmed fixed: 2026-03-30 (all stress suites pass).
