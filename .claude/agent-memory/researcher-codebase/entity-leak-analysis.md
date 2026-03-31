---
name: entity-leak-analysis
description: Root cause analysis of NoEntityLeaks invariant failures in long chaos scenarios. Two confirmed entity leaks with file/line specifics.
type: project
---

## Entity Leak Root Causes

**Why:** `NoEntityLeaks` fires when entity count exceeds 2x baseline after 10000+ frames in 5 chaos scenarios.

**How to apply:** Fix `ChainBolt` constraint leak and `SecondWindWall` missing cleanup first. These are the only confirmed no-`CleanupOnNodeExit` spawned entities found across all effect implementations.

### Confirmed Leak 1: ChainBolt DistanceConstraint entity

- File: `breaker-game/src/effect/effects/chain_bolt/effect.rs` lines 22-28
- `world.spawn(DistanceConstraint { ... })` — NO `CleanupOnNodeExit` on spawned entity
- The chain bolt (spawned via `spawn_extra_bolt`) HAS `CleanupOnNodeExit` → despawned on `OnExit(Playing)`
- The `DistanceConstraint` entity does NOT → survives across node transitions and run restarts
- `reverse()` cleans it up correctly, but `reverse()` only fires when the `Until` trigger resolves
- In chaos runs with rapid cell destruction, the node clears before the `Until` trigger, so `reverse()` is never called
- Fix: add `CleanupOnNodeExit` to the `DistanceConstraint` spawn in `chain_bolt::fire()`

### Confirmed Leak 2: SecondWindWall missing CleanupOnNodeExit

- File: `breaker-game/src/effect/effects/second_wind/system.rs` lines 39-53
- `world.spawn((SecondWindWall, Wall, WallSize, ...))` — NO `CleanupOnNodeExit`
- `despawn_second_wind_on_contact` despawns it when bolt impacts the wall
- `reverse()` despawns all `SecondWindWall` entities
- But if the node clears WITHOUT the bolt hitting the wall (cell cascade kills all cells), the wall survives into the next node
- Fix: add `CleanupOnNodeExit` to the `SecondWindWall` spawn in `second_wind::fire()`

### "Entity despawned" warnings (separate from leaks)

The "Entity despawned: invalid; generation X" warnings come from stale Entity IDs stored across frames:
- `ChainState::ArcTraveling { arc_entity, target }` stores entity IDs
- If `PlayingState` changes while an arc is mid-flight, `cleanup_entities::<CleanupOnNodeExit>` (run on `OnExit(GameState::Playing)`) despawns the arc entity
- Next FixedUpdate tick: `tick_chain_lightning` (run_if `PlayingState::Active`) should NOT run after state change, so in practice the ordering protects against double-despawn within same frame
- The multi-generation IDs (v3, v6, v10) indicate the same entity INDEX was recycled many times across `restart_run_on_end` cycles — these are warnings from the double-despawn protection in Bevy 0.18, co-incident with the leak but NOT the cause

### Cleanup architecture summary

- `cleanup_entities::<CleanupOnNodeExit>` runs on `OnExit(GameState::Playing)` — registered in `ScreenPlugin`
- `CleanupOnNodeExit` is added by: `spawn_extra_bolt` (covers: ChainBolt, SpawnPhantom, TetherBolt, SpawnBolts, MirrorProtocol), all effect spawn sites (ShockwaveSource, ChainLightningChain, ChainLightningArc, PulseRing, GravityWellMarker, TetherBeamComponent, ExplodeRequest, PiercingBeamRequest)
- Missing `CleanupOnNodeExit`: `DistanceConstraint` in `chain_bolt::fire()` and `SecondWindWall` in `second_wind::fire()`

**Why:** Per project investigation into NoEntityLeaks invariant failures on feature/scenario-coverage branch (2026-03-30).
