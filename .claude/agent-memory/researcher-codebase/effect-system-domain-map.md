---
name: effect-system-domain-map
description: Complete map of the effect/trigger pipeline — triggers, effects, dispatch, evaluation flow, and system ordering. Last verified against Phase 5 (feature/runtime-effects).
type: project
---

# Effect System Domain Map (post-rewrite, Bevy 0.18.1)

Architecture docs live at `docs/architecture/effects/index.md`.
Design docs live at `docs/design/effects/index.md` and `docs/design/triggers/index.md`.

## Trigger enum (all variants, src/effect/core/types.rs)

Bump (global): PerfectBump, EarlyBump, LateBump, Bump, BumpWhiff, NoBump
Bump (targeted bolt): PerfectBumped, EarlyBumped, LateBumped, Bumped
Impact (global): Impact(ImpactTarget) — Cell/Bolt/Wall/Breaker
Impact (targeted both participants): Impacted(ImpactTarget)
Death (global): Death, Died (targeted on entity that died)
Destruction (global): BoltLost, CellDestroyed
Node lifecycle (global): NodeStart, NodeEnd
Timer (global): NodeTimerThreshold(f32), TimeExpires(f32)

## EffectKind enum (all variants, src/effect/core/types.rs)

Combat: Shockwave, ChainLightning, PiercingBeam, Pulse, Explode
Bolt spawning: SpawnBolts, ChainBolt, SpawnPhantom
Stat modifiers: SpeedBoost, DamageBoost, Piercing, SizeBoost, BumpForce, RampingDamage, Attraction
Breaker modifiers: QuickStop
Defensive: Shield, SecondWind, GravityWell
Penalties: LoseLife, TimePenalty
Meta: RandomEffect, EntropyEngine, TetherBeam

## EffectNode types (src/effect/core/types.rs)

When { trigger, then } — gate, matches trigger, permanent in BoundEffects
Do(EffectKind) — terminal, fires on current entity
Once(children) — one-shot wrapper, consumed when any child matches
On { target, permanent, then } — redirect to another entity
Until { trigger, then } — desugared by dedicated system
Reverse { effects, chains } — internal only, created by Until desugaring

## Components (src/effect/core/types.rs)

BoundEffects(Vec<(String, EffectNode)>) — permanent effect trees, never consumed
StagedEffects(Vec<(String, EffectNode)>) — working set, consumed when matched

## Trigger bridge system state (src/effect/triggers/)

Most trigger bridges are stubs. "Wired in Wave 8" comments throughout.

- bump, perfect_bump, early_bump, late_bump, bump_whiff, no_bump — stubs
- bumped, perfect_bumped, early_bumped, late_bumped — stubs
- impact, impacted — stubs (these consume the collision messages)
- death — REAL: bridge_death reads RequestCellDestroyed/RequestBoltDestroyed, fires Trigger::Death globally on all BoundEffects entities (in EffectSystems::Bridge, FixedUpdate)
- died — REAL: bridge_died reads same messages, fires Trigger::Died only on the dying entity (targeted, in EffectSystems::Bridge, FixedUpdate)
- bolt_lost — REAL: bridge_bolt_lost reads BoltLost message, fires Trigger::BoltLost globally on all BoundEffects entities
- node_start, node_end — stubs
- timer — REAL: tick_time_expires (was already noted below)
- until — REAL: desugar_until (was already noted below)

## EffectSystems set (src/effect/sets.rs)

EffectSystems::Bridge — the label for all trigger bridge systems

## Confirmed: death/died bridges are live (corrected from earlier memory)

bridge_death and bridge_died both exist in src/effect/triggers/ and are fully implemented
with tests. They consume RequestCellDestroyed and RequestBoltDestroyed and fire Trigger::Death
(global) and Trigger::Died (targeted) respectively into the BoundEffects evaluation pipeline.
cleanup_cell (which emits CellDestroyedAt) runs after EffectSystems::Bridge to ensure bridges
see the entity before it is despawned.

## Dispatch location (NOT in effect domain)

chips/ → dispatch_chip_effects
breaker/ → breaker init (apply_breaker_config_overrides / init_breaker)
cells/ → cell init
All three: for each RootEffect::On { target, then }, resolve target,
fire bare Do nodes or push non-Do to BoundEffects on target entity.

## Collision messages (defined in detecting domain)

BoltImpactCell, BoltImpactWall, BoltImpactBreaker (bolt/messages.rs)
BreakerImpactCell, BreakerImpactWall (breaker/messages.rs)
CellImpactWall (cells/messages.rs)
DamageCell (cells/messages.rs)

One collision message → four triggers (Impact(X) global + Impacted(X) targeted) —
but all trigger bridges are stubs, so no trigger evaluation happens yet.
