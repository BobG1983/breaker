---
name: build-failure-bolt-breaker-query
description: Build failure in bolt_breaker_collision/system.rs — E0308 tuple arity mismatch when destructuring Query<(Entity, CollisionQueryBreaker)>
type: project
---

`breaker-game` fails to compile at `breaker-game/src/bolt/systems/bolt_breaker_collision/system.rs:63`.

**Failure:** E0308 mismatched types — the system destructures `breaker_query.single()` as a flat 9-element tuple, but `Query<(Entity, CollisionQueryBreaker), ...>` returns a 2-tuple `(Entity, CollisionQueryBreaker)` where `CollisionQueryBreaker` is itself an 8-tuple defined in `breaker-game/src/breaker/queries.rs:20`.

**Fix:** The destructure at line 63 must nest the 8 `CollisionQueryBreaker` fields inside a second tuple level:
```rust
let Ok((breaker_entity, (
    breaker_position,
    breaker_tilt,
    breaker_w,
    breaker_h,
    max_angle,
    min_angle,
    width_boost,
    breaker_entity_scale,
))) = breaker_query.single()
```

**Why:** This is a writer-code bug — the query type alias wraps the 8 fields in a tuple, so destructuring must match the two-level structure.

**How to apply:** Route to writer-code with a fix spec hint pointing to `system.rs:63` and `queries.rs:20`.
