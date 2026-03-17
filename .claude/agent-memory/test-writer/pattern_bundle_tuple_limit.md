---
name: pattern_bundle_tuple_limit
description: Bevy Bundle impl has a max tuple size (~15 items). When spawning entities with many components in tests, split into spawn + insert.
type: project
---

# Bevy Bundle Tuple Size Limit

Bevy's `Bundle` derive only covers tuples up to a certain size (~15 elements). When spawning entities with many components (e.g., the full breaker component set, which has 25+ items), a single large tuple passed to `world.spawn((...))` will fail to compile with:

```
error[E0277]: `(...large tuple...)` is not a `Bundle`
```

## Fix: split into spawn + entity_mut().insert()

```rust
let entity = world.spawn((
    Comp1, Comp2, ... Comp12,  // first batch
)).id();
world.entity_mut(entity).insert((
    Comp13, Comp14, ... Comp25,  // second batch
));
```

Each insert tuple must itself be within the tuple size limit.

## When this matters

This only comes up in test setup helpers that spawn entities with the full component set from a config. Production code uses multiple `.insert()` calls (as in `init_breaker_params.rs`) which each have their own reasonably-sized tuples.
