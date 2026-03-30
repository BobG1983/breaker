---
name: effect-domain-inventory
description: Complete inventory of effect implementations and trigger bridges. Last verified against feature/source-chip-shield-absorption (all bridges real, all effects real).
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

### Trigger Bridge Status

**REAL implementations:**
- `death` — `bridge_death`: reads `RequestCellDestroyed`/`RequestBoltDestroyed`, fires `Trigger::Death` globally on all `BoundEffects` entities
- `died` — `bridge_died`: same messages, fires `Trigger::Died` targeted on the dying entity only
- `bolt_lost` — `bridge_bolt_lost`: reads `BoltLost` message, fires `Trigger::BoltLost` globally on all `BoundEffects` entities
- `timer.rs` — `tick_time_expires`: ticks `When(TimeExpires(f32), ...)` nodes in StagedEffects. Has 5 tests.
- `until.rs` — `desugar_until`: desugars `Until` nodes to `When+Reverse`. Has 6 tests.
- `evaluate.rs` — `evaluate_bound_effects` / `evaluate_staged_effects`: shared helpers. Has 8 tests.

**All trigger bridges are REAL as of feature/source-chip-shield-absorption:**
- bump, perfect_bump, early_bump, late_bump, bump_whiff, no_bump — REAL: consume BumpPerformed messages, fire respective Trigger variants globally
- bumped, perfect_bumped, early_bumped, late_bumped — REAL: fire respective Trigger variants targeted on the bolt entity
- impact, impacted — REAL: each has 6 bridge functions consuming all 6 collision message types
- node_start — REAL: OnEnter(PlayingState::Active), fires Trigger::NodeStart globally
- node_end — REAL: fires Trigger::NodeEnd on node cleared
- cell_destroyed — REAL: consumes CellDestroyedAt, fires Trigger::CellDestroyed globally

Note: `NodeTimerThreshold(f32)` variant exists in `Trigger` enum. Bridge system may be registered (not explicitly confirmed).

### Effect Implementations — Status Summary

#### REAL (component + fire/reverse + optional systems)

| Effect | Component(s) | fire() | reverse() | Runtime systems | Tests |
|---|---|---|---|---|---|
| SpeedBoost | `ActiveSpeedBoosts(Vec<f32>)` | push multiplier | remove matching | `recalculate_speed` (REAL) | 7 |
| DamageBoost | `ActiveDamageBoosts(Vec<f32>)` | push multiplier | remove matching | `recalculate_damage` (REAL) | 7 |
| Piercing | `ActivePiercings(Vec<u32>)` | push count | remove matching | `recalculate_piercing` (REAL) | 7 |
| SizeBoost | `ActiveSizeBoosts(Vec<f32>)` | push value | remove matching | `recalculate_size` (REAL) | 7 |
| BumpForce | `ActiveBumpForces(Vec<f32>)` | push force | remove matching | `recalculate_bump_force` (REAL) | 7 |
| QuickStop | `ActiveQuickStops(Vec<f32>)` | push multiplier | remove matching | none registered | 7 |
| LoseLife | `LivesCount(u32)` | decrement | increment | none | 3 |
| Shockwave | `ShockwaveSource`, `ShockwaveRadius`, `ShockwaveMaxRadius`, `ShockwaveSpeed` | spawns entity | noop | tick+despawn in FixedUpdate, run_if Active | 5 |
| Shield | `ShieldActive { charges: u32 }` | insert/add charges | remove | tick+remove expired in FixedUpdate, run_if Active | 5 |
| Attraction | `ActiveAttractions(Vec<AttractionEntry>)` | insert entry | remove matching | `apply_attraction` + `manage_attraction_types` (REAL) | 10+ |
| GravityWell | `GravityWellMarker`, `GravityWellConfig` | spawn + cap enforcement | noop | tick+despawn + `apply_gravity_pull` in FixedUpdate, run_if Active | 5 |
| RampingDamage | `RampingDamageState { bonus_per_hit, accumulated, hits }` | insert fresh state | remove | none | 3 |
| ChainBolt | `ChainBoltMarker`, `ChainBoltAnchor` | spawn 2 entities | despawn all + remove anchor | none | 3 |
| SecondWind | `SecondWindWall` | spawn wall entity | despawn all SecondWindWall | none | 2 |
| Pulse | `PulseEmitter`, `PulseRing`, `PulseSource`, `PulseRadius`, `PulseMaxRadius`, `PulseSpeed`, `PulseDamaged` | adds PulseEmitter to entity | removes PulseEmitter | tick_pulse_emitter + tick_pulse_ring + apply_pulse_damage in FixedUpdate, run_if Active | 18 |
| SpawnPhantom | `PhantomBoltMarker`, `PhantomOwner` | spawns full bolt via spawn_extra_bolt + inserts PhantomBoltMarker, BoltLifespan(Timer), PiercingRemaining(u32::MAX); cap enforcement | noop | none (lifespan via tick_bolt_lifespan in bolt domain) | 10+ |
| ChainLightning | `ChainLightningChain` (sequential arc model), `ChainLightningArc` | spawns ChainLightningChain entity; DamageCell to first target | noop | tick_chain_lightning in FixedUpdate, run_if Active | 20+ |
| PiercingBeam | `PiercingBeamRequest` (deferred) | spawns PiercingBeamRequest with pre-computed geometry | noop | process_piercing_beam in FixedUpdate | 10+ |
| TetherBeam | `TetherBeamComponent`, `TetherBoltMarker` | spawns 2 tether bolts + beam entity | despawns beam + tether bolts | tick_tether_beam in FixedUpdate, run_if Active | 15+ |
| SpawnBolts | (no persistent component) | spawns N full bolt entities via spawn_extra_bolt | noop | none | 15+ |
| EntropyEngine | `EntropyEngineState { cells_destroyed: u32 }` | increments cells_destroyed, fires N random effects | noop | reset_entropy_engine on OnEnter(PlayingState::Active) | 20+ |
| Explode | `ExplodeRequest` (deferred, CleanupOnNodeExit) | spawns ExplodeRequest entity | noop | process_explode in FixedUpdate | 5+ |

#### All Effects are REAL as of feature/runtime-effects

- `TimePenalty` — real: writes `ApplyTimePenalty`/`ReverseTimePenalty` messages to the run domain's node timer
- `RandomEffect` — real: uses `WeightedIndex` + `GameRng` to select from pool; fires selected node via `StagedEffects`

No known placeholder effect implementations remain as of feature/runtime-effects.

### "Recalculate" systems pattern note

SpeedBoost/DamageBoost/Piercing/SizeBoost/BumpForce all register `recalculate_*` systems
in FixedUpdate. These systems are REAL — they query `(&ActiveXxx, &mut EffectiveXxx)` and
propagate the stacked multiplier/total into the `Effective*` component.

The `Active*` → `Effective*` propagation is wired. However, the bolt domain's consumer systems
(`prepare_bolt_velocity`, `bolt_cell_collision`) may still have the 1-frame-stale issue
(see reviewer-correctness/bug-patterns.md for ordering gap details).

### Key File Paths

- `src/effect/core/types.rs` — all types: Trigger, Target, EffectNode, EffectKind, BoundEffects, StagedEffects
- `src/effect/commands.rs` — EffectCommandsExt trait + FireEffectCommand/ReverseEffectCommand/TransferCommand
- `src/effect/triggers/evaluate.rs` — evaluate_bound_effects / evaluate_staged_effects helpers (REAL + tested)
- `src/effect/triggers/timer.rs` — tick_time_expires (REAL + tested)
- `src/effect/triggers/until.rs` — desugar_until (REAL + tested)
- `src/effect/effects/<name>.rs` — one file per EffectKind variant
