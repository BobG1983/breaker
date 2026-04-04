---
name: Effect domain Velocity2D cross-domain writes
description: gravity_well and attraction effects write Velocity2D on bolt entities via FixedUpdate systems — undocumented cross-domain write exception
type: project
---

Two effect domain runtime systems write `Velocity2D` on bolt entities:
- `apply_gravity_pull` in `effect/effects/gravity_well/effect.rs` (see FixedUpdate registration in gravity_well/mod.rs or plugin.rs)
- `apply_attraction` in `effect/effects/attraction/effect.rs`

These are FixedUpdate systems (not fire() functions), running each tick to continuously modify bolt velocity. Velocity2D is a rantzsoft_spatial2d component carried by bolt entities.

**Why this matters:** `docs/architecture/plugins.md` says "Writes to other domains only through messages — no direct mutation of another domain's components or resources." The only documented exception is the debug domain. ShieldActive was a former exception but was ELIMINATED in the Shield refactor (2026-04-02). These Velocity2D writes are not documented as exceptions.

**Why it's acceptable (pattern rationale):** The purpose of GravityWell and Attraction effects is to apply forces to bolts. Adding message indirection (effect writes `GravityForce`, bolt reads and applies) would add complexity without benefit — the force application is simple arithmetic on velocity. These systems run in FixedUpdate before collision systems so `apply_velocity_formula()` (called inline at each collision site) still applies speed clamping. NOTE: `BoltSystems::PrepareVelocity` was eliminated in builder migration — the ordering anchor is now the collision system sets directly.

**How to apply:** Accept as a pragmatic exception. If documenting, add to `docs/architecture/plugins.md` alongside the ShieldActive exception.
