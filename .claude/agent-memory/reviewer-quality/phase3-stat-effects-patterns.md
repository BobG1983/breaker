---
name: Phase 3 Stat Effects — Intentional Patterns
description: Patterns established in the Phase 3 feature/stat-effects branch that look like violations but are correct for this codebase. Updated after cache-removal refactor.
type: project
---

## CURRENT: Active*-only pattern (post cache-removal refactor)

The `Effective*` components and their `recalculate_*` systems have been removed.
All consumers now read `Active*` types directly and call `.multiplier()` inline:

```rust
let mult = active_boosts.map_or(1.0, ActiveSpeedBoosts::multiplier);
```

The old `Option<&Effective*>` pattern with `map_or(1.0, |e| e.0)` is GONE.
Do not flag `Active*::multiplier` method-reference syntax as unusual — it is the established pattern.

## `multiplier()` method identical across Active* types

`ActiveSpeedBoosts`, `ActiveDamageBoosts`, `ActiveSizeBoosts`, `ActiveBumpForces`, `ActiveQuickStops`
all have a `multiplier()` method with the same empty-returns-1.0 / product logic. This is intentional
repetition per the plugin-per-domain convention rather than a shared trait. Do not flag without confirmation
a shared trait is desired.

## "Recalculation:" doc comment in effect files is stale

`speed_boost.rs`, `size_boost.rs`, `quick_stop.rs`, `bump_force.rs` all have struct-level doc comments
that say `/// Recalculation: base_X * product(all_boosts)`. This language implies a separate caching
step that no longer exists. Consumers compute on demand. These comments should be updated to say
something like `/// The effective multiplier is the product of all entries (default 1.0).`
This is a documentation quality issue flagged in the cache-removal review.

## `dispatch_breaker_effects` is a REAL system (FIXED — no longer a stub)

`dispatch_breaker_effects` is now a real system at `breaker/systems/dispatch_breaker_effects/system.rs`.

## `unwrap()` in `bolt_wall_collision` face selection is safe

`faces.into_iter().min_by(...).unwrap()` is safe: `faces` is a fixed 4-element array literal,
so `min_by` on a non-empty iterator cannot return `None`. An invariant comment would strengthen
this but it is not a bug.
