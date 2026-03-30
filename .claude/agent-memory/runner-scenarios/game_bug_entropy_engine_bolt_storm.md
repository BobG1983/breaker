---
name: BoltCountReasonable violation — EntropyEngine + SpawnBolts has no global bolt cap
description: EntropyEngine escalates to max_effects per cell destruction; combined with SpawnBolts and no cap, produces bolt storms in Dense layouts
type: project
---

## Bug: entropy_engine + spawn_bolts — no bolt count cap

`EntropyEngine.fire()` scales effects from 1 to `max_effects` as cells_destroyed grows. Once cells_destroyed >= max_effects (quickly in Dense layouts), every cell destruction fires `max_effects` random effects. With a 50/50 pool of SpawnBolts(count:1) and Shockwave, bursts of 2-3 new bolts can spawn per destruction.

`SpawnBolts.fire()` (`breaker-game/src/effect/effects/spawn_bolts/effect.rs`) has no check for total active bolt count. It spawns unconditionally.

In `entropy_engine_stress`, the Dense layout destroys cells so rapidly that the spawn rate overwhelms the 2-second lifespan drain rate, producing >12 active bolts (the configured `max_bolt_count` threshold) for 2266 consecutive frames.

Secondary symptom: `"Entity despawned: Entity ... is invalid"` Bevy warnings from double-despawn — a bolt can receive two `RequestBoltDestroyed` messages (lifespan expiry + other trigger), leading to a second `commands.entity(...).despawn()` on an already-despawned entity.

## Fix needed

Either:
1. Add a global bolt count cap in `spawn_extra_bolt()` or `SpawnBolts.fire()` — refuse to spawn if bolt count >= some threshold
2. Or accept that `entropy_engine_stress` needs a higher `max_bolt_count` threshold if the scenario intent is to test bolt cleanup, not bolt count

Decision: Needs design input — is "too many bolts" a game bug or an intended stress scenario that needs a higher threshold?

File: `breaker-game/src/effect/effects/spawn_bolts/effect.rs`, `breaker-game/src/effect/effects/mod.rs` (`spawn_extra_bolt`)

Confirmed 2026-03-30.
