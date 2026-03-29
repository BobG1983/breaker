---
name: effect-domain-inventory
description: Complete inventory of effect implementations and trigger bridges — which are real vs placeholders, what components each exposes. Last verified against Phase 5 (feature/runtime-effects).
type: project
---

## Effect Domain Architecture

The effect domain lives in `breaker-game/src/effect/`. It is a data-driven trigger→effect pipeline.

### Core Components

- `BoundEffects(Vec<(String, EffectNode)>)` — permanent effect trees on an entity. Never consumed by trigger evaluation.
- `StagedEffects(Vec<(String, EffectNode)>)` — working set of partially-resolved chains. Consumed when matched.

### Effect Node Types

- `EffectNode::When { trigger, then }` — gate node; matches a trigger
- `EffectNode::Do(EffectKind)` — terminal; fires a leaf effect
- `EffectNode::Once(children)` — one-shot; consumed when any child matches
- `EffectNode::On { target, permanent, then }` — redirects to another entity
- `EffectNode::Until { trigger, then }` — duration-scoped; desugared by `desugar_until` system
- `EffectNode::Reverse { effects, chains }` — internal node created by Until desugaring

### Trigger Bridge Status (all placeholders — "Wave 8")

All 18 trigger bridges in `src/effect/triggers/` are empty placeholder bodies:
- bump, perfect_bump, early_bump, late_bump, bump_whiff, no_bump (global bump)
- bumped, perfect_bumped, early_bumped, late_bumped (targeted on bolt)
- impact, impacted (collision)
- death, died, bolt_lost (lifecycle)
- node_start, node_end (node lifecycle)

Note: `NodeTimerThreshold(f32)` variant exists in `Trigger` enum but has NO bridge system registered.

### Trigger Systems with REAL implementations

- `timer.rs` — `tick_time_expires`: ticks `When(TimeExpires(f32), ...)` nodes in StagedEffects. Has 5 tests.
- `until.rs` — `desugar_until`: desugars `Until` nodes to `When+Reverse`. Has 6 tests including overclock integration test.
- `evaluate.rs` — `evaluate_bound_effects` / `evaluate_staged_effects`: shared helpers called by bridge systems. Has 8 tests.

### Effect Implementations — Status Summary

#### REAL (component + fire/reverse + optional systems)

| Effect | Component(s) | fire() | reverse() | Runtime systems | Tests |
|---|---|---|---|---|---|
| SpeedBoost | `ActiveSpeedBoosts(Vec<f32>)` | push multiplier | remove matching | `recalculate_speed` (placeholder body) | 7 |
| DamageBoost | `ActiveDamageBoosts(Vec<f32>)` | push multiplier | remove matching | `recalculate_damage` (placeholder body) | 7 |
| Piercing | `ActivePiercings(Vec<u32>)` | push count | remove matching | `recalculate_piercing` (placeholder body) | 7 |
| SizeBoost | `ActiveSizeBoosts(Vec<f32>)` | push value | remove matching | `recalculate_size` (placeholder body) | 7 |
| BumpForce | `ActiveBumpForces(Vec<f32>)` | push force | remove matching | `recalculate_bump_force` (placeholder body) | 7 |
| QuickStop | `ActiveQuickStops(Vec<f32>)` | push multiplier | remove matching | none registered | 7 |
| LoseLife | `LivesCount(u32)` | decrement | increment | none | 3 |
| Shockwave | `ShockwaveSource`, `ShockwaveRadius`, `ShockwaveMaxRadius`, `ShockwaveSpeed` | spawns entity | noop | tick+despawn in FixedUpdate, run_if Active | 5 |
| Shield | `ShieldActive { remaining, owner }` | insert/extend | remove | tick+remove expired in FixedUpdate, run_if Active | 5 |
| Attraction | `ActiveAttractions(Vec<AttractionEntry>)` | insert entry | remove matching | `apply_attraction` + `manage_attraction_types` (both placeholder bodies) | 4 |
| SpawnPhantom | `PhantomBoltMarker`, `PhantomTimer`, `PhantomOwner` | spawn + cap enforcement | noop | tick+despawn in FixedUpdate, run_if Active | 4 |
| GravityWell | `GravityWellMarker`, `GravityWellConfig` | spawn + cap enforcement | noop | tick+despawn + `apply_gravity_pull` in FixedUpdate, run_if Active | 5 |
| RampingDamage | `RampingDamageState { bonus_per_hit, accumulated, hits }` | insert fresh state | remove | none | 3 |
| ChainBolt | `ChainBoltMarker`, `ChainBoltAnchor` | spawn 2 entities | despawn all + remove anchor | none | 3 |
| SecondWind | `SecondWindWall` | spawn wall entity | despawn all SecondWindWall | none | 2 |
| Pulse | `PulseEmitter`, `PulseRing`, `PulseSource`, `PulseRadius`, `PulseMaxRadius`, `PulseSpeed`, `PulseDamaged` | adds PulseEmitter to entity | removes PulseEmitter | tick_pulse_emitter + tick_pulse_ring + apply_pulse_damage in FixedUpdate, run_if Active | 18 |

#### PLACEHOLDERS (log only, no real behavior)

| Effect | Notes |
|---|---|
| ChainLightning | debug log only; spatial query + chaining not implemented |
| PiercingBeam | debug log only; beam cast not implemented |
| SpawnBolts | info log only; needs SpawnAdditionalBolt message from bolt domain |
| TimePenalty | info log only; needs NodeTimer resource from run domain |
| RandomEffect | picks first element deterministically; no weighted random; no actual firing |
| EntropyEngine | completely empty body; "Wave 8" |
| Explode | debug log only; spatial query not implemented |
| TetherBeam | completely empty body; "Wave 8" |

### "Recalculate" systems pattern note

SpeedBoost/DamageBoost/Piercing/SizeBoost/BumpForce all register `recalculate_*` systems
in FixedUpdate but each has a placeholder body ("Wave 6"). These components hold the
stacked values correctly but don't yet propagate them back to the game (bolt speed, etc.).

**Why:** the `ActiveXxx` component → game behavior wiring is pending Wave 6 integration.

### Key File Paths

- `src/effect/core/types.rs` — all types: Trigger, Target, EffectNode, EffectKind, BoundEffects, StagedEffects
- `src/effect/commands.rs` — EffectCommandsExt trait + FireEffectCommand/ReverseEffectCommand/TransferCommand
- `src/effect/triggers/evaluate.rs` — evaluate_bound_effects / evaluate_staged_effects helpers (REAL + tested)
- `src/effect/triggers/timer.rs` — tick_time_expires (REAL + tested)
- `src/effect/triggers/until.rs` — desugar_until (REAL + tested)
- `src/effect/effects/<name>.rs` — one file per EffectKind variant
