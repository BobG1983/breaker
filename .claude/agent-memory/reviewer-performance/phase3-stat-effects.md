---
name: Phase 3 stat-effects performance analysis
description: Analysis of Active*/Effective* components and recalculate systems (feature/stat-effects through cache-removal refactor)
type: project
---

## Phase 3 Entity Scale

The Active* components target bolt and breaker entities only:
- Bolt: 1 normally, up to ~4 with chain-bolt
- Breaker: 1

At most 5 entities in worst case.

## Cache Removal Refactor (2026-03-30)

All 6 `Effective*` cache components (`EffectiveSpeedMultiplier`, `EffectiveDamageMultiplier`,
`EffectiveSizeMultiplier`, `EffectiveBumpForceMultiplier`, `EffectiveQuickStopMultiplier`,
`EffectivePiercingTotal`) and all 6 `recalculate_*` systems have been REMOVED.

Consumers now call `.multiplier()` inline at point of use. EffectPlugin no longer has
`EffectSystems::Recalculate` — the `sets.rs` file only defines `EffectSystems::Bridge`.

## Hot-Path Call Sites for .multiplier()

These are the FixedUpdate call sites (all confirmed by code inspection):

- `prepare_bolt_velocity` — `active_boosts.map_or(1.0, ActiveSpeedBoosts::multiplier)` — once per bolt
- `bolt_cell_collision` — `damage_mult.map_or(1.0, ActiveDamageBoosts::multiplier)` — once per bolt BEFORE the CCD bounce loop (not inside it)
- `bolt_breaker_collision` — `active_speed_boosts.map_or(1.0, ActiveSpeedBoosts::multiplier)` — once per bolt; `size_mult.map_or(1.0, ActiveSizeBoosts::multiplier)` — once per breaker
- `move_breaker` — `ActiveSpeedBoosts::multiplier` once, `ActiveSizeBoosts::multiplier` once — 1 breaker entity
- `update_breaker_state` (dash) — `ActiveSpeedBoosts::multiplier` once, `ActiveSizeBoosts::multiplier` once — 1 breaker entity

Performance verdict: CLEAN. `.multiplier()` is `iter().product()` on a `&[f32]` — no allocation.
Vec typically has 0-3 elements. Total compute per frame: ~10 multiplications across all call sites.

## Net Cost vs Prior Architecture

PRIOR: 6 FixedUpdate systems × query iteration × write to Effective* components + subsequent read
NOW: Direct `Option<&Active*>::map_or(1.0, ...)` calls at consumption point

The refactor eliminates 6 system calls, 6 query iterations, and 6 component write passes per
FixedUpdate frame. Consumers incur the product computation directly but this is cheaper than the
prior round-trip through the scheduler and component table. Net win at all scales.

## Archetype Impact

Removing 6 `Effective*` components strictly reduces fragmentation:
- Bolt entities no longer straddle archetypes based on which Effective* they have
- `CollisionQueryBolt` and `MovementQuery` already used `Option<&Active*>` — these queries are
  unchanged since Effective* was never in them

The only remaining Optional component pattern: `CollisionQueryBolt` has
`Option<&ActiveDamageBoosts>`, `Option<&ActivePiercings>`, `Option<&ActiveSpeedBoosts>` plus
existing optional fields. At 1-4 bolts, this is not a fragmentation concern.

## Vec<f32> Allocation Pattern

Each Active* stores a `Vec<f32>` (or `Vec<u32>` for piercing). `.multiplier()` / `.total()`
iterate this vec inline — no allocation. The Vec is heap-allocated per entity at chip dispatch
time (lazy init in fire()), not per frame. Clean.

## How Active* Components Are Inserted

Lazy init in `fire()` — inserted on first chip activation, not at entity spawn. Entities without
any stat chip activated have none of these components. `query.get()` returns None → `map_or(1.0, ...)`
→ returns the default multiplier. Zero overhead when no chip has fired.

`quick_stop.rs` does NOT have lazy init — assumes `ActiveQuickStops` is pre-inserted.
This is intentional (different registration pattern).
