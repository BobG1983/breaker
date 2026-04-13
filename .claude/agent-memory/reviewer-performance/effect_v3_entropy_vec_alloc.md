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

**fire_dispatch inside inner loop:** Called up to `max_effects * bump_count` times per entity
per frame. At max_effects=5 and 1 bump, that's 5 `fire_dispatch` calls. Each call does a
`world.get_mut` or `world.spawn` (for shockwave effects). This is the real cost driver for
EntropyEngine at high escalation — not the Vec alloc.
