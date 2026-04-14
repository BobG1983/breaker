---
name: entropy_engine and circuit_breaker exclusive system Vec alloc pattern
description: Both collect() all counter entities into Vec every time BumpPerformed fires
type: project
---

Both `tick_entropy_engine` and `tick_circuit_breaker` use the same exclusive system pattern:
1. Count bumps via `.count()` on message iterator (no alloc, early exit on 0)
2. Collect all `EntropyCounter`/`CircuitBreakerCounter` entities into a `Vec<(Entity, Component)>`
3. Process bumps per entity, calling `fire_dispatch` in the inner loop

The `collect()` at step 2 allocates a Vec. This only runs when `bump_count > 0`, so it's
bump-gated, not frame-gated. At current scale (1–2 counter entities), the Vec is tiny.

**EntropyCounter.pool clone:** `pick_weighted_effect` clones the final `EffectType` from the
pool on every call. `EffectType` contains configs with `OrderedFloat` fields — clone is cheap
(no heap allocation unless the effect is `ChainLightning` which contains a `Vec` in its config).

**fire_dispatch inside inner loop (circuit_breaker now fires TWO effects):** Wave 5 updated
`tick_circuit_breaker` to call `fire_dispatch` twice per counter-zero-reach (shockwave + spawn_bolts).
SpawnBoltsConfig::fire allocates a `Vec<f32>` of random angles (one entry per bolt spawned),
gated by `count > 0` early-exit. Both `fire_dispatch` calls are event-driven (only when
counter hits zero), not per-frame. At current scale (1 circuit breaker chip, typical
bumps_required=3–5), this fires rarely and the double dispatch is negligible.

**GameRng access in SpawnBoltsConfig::fire:** Takes `world.resource_mut::<GameRng>()` inside
the exclusive system body — no parallelism conflict since exclusive systems own the World.
