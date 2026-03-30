---
name: Effect domain Velocity2D cross-domain writes
description: gravity_well and attraction effects write Velocity2D on bolt entities via FixedUpdate systems — undocumented cross-domain write exception
type: project
---

Two effect domain runtime systems write `Velocity2D` on bolt entities:
- `apply_gravity_pull` in `effect/effects/gravity_well.rs` (line 89)
- `apply_attraction` in `effect/effects/attraction/effect.rs` (line 88)

These are FixedUpdate systems (not fire() functions), running each tick to continuously modify bolt velocity. Velocity2D is a rantzsoft_spatial2d component carried by bolt entities.

**Why this matters:** `docs/architecture/plugins.md` says "Writes to other domains only through messages — no direct mutation of another domain's components or resources." The documented exceptions are ShieldActive (bolt and cells) and the debug domain. These velocity writes are not documented as exceptions.

**Why it's acceptable (pattern rationale):** The purpose of GravityWell and Attraction effects is to apply forces to bolts. Adding message indirection (effect writes `GravityForce`, bolt reads and applies) would add complexity without benefit — the force application is simple arithmetic on velocity. Both systems run `.before(BoltSystems::PrepareVelocity)` so the bolt domain's speed clamping still applies.

**How to apply:** Accept as a pragmatic exception. If documenting, add to `docs/architecture/plugins.md` alongside the ShieldActive exception.
