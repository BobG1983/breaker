---
name: ShieldActive cross-domain write exception — ELIMINATED (and pattern reused for SecondWind)
description: ShieldActive no longer exists; Shield (and SecondWind as of 2026-04-13) are wall entities spawned via Wall::builder().floor(&playfield).spawn(&mut commands) from effect_v3
type: project
---

**ELIMINATED as of Shield refactor (2026-04-02, commit e887570).**

`ShieldActive` component NO LONGER EXISTS. Shield is now a wall entity spawned by `effect_v3::effects::shield::ShieldConfig::fire()` via `Wall::builder().floor(&playfield).spawn(&mut commands)`, with lifetime managed via its own `ShieldDuration` component (ticked in `EffectV3Systems::Tick`).

**Wave 1 of the effect_v3 audit (2026-04-13) extended this pattern to SecondWind:**

- `effect_v3/effects/second_wind/config.rs` uses the same `Wall::builder().floor(...).spawn(...)` path.
- `effect_v3/effects/second_wind/systems.rs` (`despawn_on_first_reflection`) reads `BoltImpactWall` like shield's `apply_shield_reflection_cost`.
- `Wall` has `#[require(Spatial2D, CleanupOnExit<NodeState>)]` at `walls/components.rs:11`, so cleanup is automatic via the `require` chain.
- `Lifetime::Timed` / `Lifetime::OneShot` in `walls/builder/core/types.rs` remain `#[cfg(test)]` — effects manage their own lifetime via components (`ShieldDuration`, `SecondWindWall` marker-despawn), not via the builder's `Lifetime` typestate.

**Acceptable cross-domain use of the walls builder from effect_v3**: effect modules may call `Wall::builder().floor(...).spawn(...)` directly. This is NOT a cross-domain write violation — it is a legitimate consumer of the walls domain's public builder API (`pub(crate)` at `walls/builder/core/transitions.rs:13`).

**Do NOT re-flag any absence of ShieldActive cross-domain write exceptions — the entire mechanism was redesigned.**
