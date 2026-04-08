---
name: entropy_engine_stress birthing regression
description: entropy_engine_stress self-test fails after .birthed() was added to SpawnBolts::fire — bolt accumulation dynamics changed, peak count never exceeds 12
type: project
---

`entropy_engine_stress` is a stress self-test (`allowed_failures: [BoltCountReasonable]`) that expects
`BoltCountReasonable` to fire at least once. The scenario uses `EntropyEngine` + `SpawnBolts(count: 1, lifespan: 2.0)`
on a Dense layout with `invariant_params: (max_bolt_count: 12)`.

**Root cause:** Commit `3c920353` added `.birthed()` to `SpawnBolts::fire`. This causes all effect-spawned bolts
to enter a 0.3s birthing animation with:
- `CollisionLayers` zeroed (cannot collide)
- `Scale2D` zeroed (invisible, zero size)
- `Without<Birthing>` excludes them from `bolt_lost`, `tick_bolt_lifespan`, `ActiveFilter`

Effect: Birthing bolts live slightly longer (lifespan timer doesn't tick during 0.3s birth), but they also
cannot interact with anything. The simulation path diverges enough that with seed 4242 over 4000 frames,
the peak simultaneous bolt count never exceeds 12.

**Failure type:** Game bug — the scenario's stress threshold `max_bolt_count: 12` is now too conservative
given birthing dynamics. The scenario needs its `max_bolt_count` lowered (e.g. to 8 or 10) OR more cells
destroyed per run OR a higher `max_effects` OR a longer `max_frames` to reliably exceed the threshold.

**Key files:**
- `breaker-scenario-runner/scenarios/stress/entropy_engine_stress.scenario.ron` — threshold/params to adjust
- `breaker-game/src/effect/effects/spawn_bolts/effect.rs` — where `.birthed()` was added
- `breaker-game/src/bolt/systems/tick_bolt_lifespan.rs` — `Without<Birthing>` guard
- `breaker-game/src/bolt/systems/bolt_lost/system.rs` — `ActiveFilter` excludes birthing bolts

**Why:** Without birthing, extra bolts immediately had full physics and could accumulate to 13+.
With birthing, the peak count drops because some bolts are always in the non-interacting birthing state.
