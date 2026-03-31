---
name: effect-system-domain-map
description: Complete map of the effect/trigger pipeline — triggers, effects, dispatch, evaluation flow, and system ordering. Last verified against feature/source-chip-shield-absorption (all bridges real).
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

ALL trigger bridges are REAL as of feature/source-chip-shield-absorption (2026-03-29). No stubs remain.

- bump, perfect_bump, early_bump, late_bump, bump_whiff, no_bump — REAL: consume BumpPerformed (with BumpGrade), fire respective Trigger variants globally on all BoundEffects entities
- bumped, perfect_bumped, early_bumped, late_bumped — REAL: fire respective Trigger variants targeted on the bolt entity that was bumped
- impact — REAL: 6 bridge functions consuming all 6 collision message types, fire Impact(X) globally on all BoundEffects entities
- impacted — REAL: 6 bridge functions, fire Impacted(X) targeted on each participant entity
- death — REAL: bridge_death reads RequestCellDestroyed/RequestBoltDestroyed, fires Trigger::Death globally
- died — REAL: bridge_died reads same messages, fires Trigger::Died targeted on the dying entity
- bolt_lost — REAL: bridge_bolt_lost reads BoltLost message, fires Trigger::BoltLost globally
- cell_destroyed — REAL: bridge_cell_destroyed reads CellDestroyedAt, fires Trigger::CellDestroyed globally
- node_start — REAL: OnEnter(PlayingState::Active), fires Trigger::NodeStart globally
- node_end — REAL: fires Trigger::NodeEnd globally when node is cleared
- timer — REAL: tick_time_expires (ticks When(TimeExpires) nodes)
- until — REAL: desugar_until (desugars Until nodes to When+Reverse)

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
all trigger bridges are REAL as of feature/source-chip-shield-absorption.
