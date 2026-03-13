---
name: known-conflicts
description: Known query conflicts, ordering issues, and missing constraints identified in the brickbreaker system map (as of 2026-03-13 full re-scan)
type: reference
---

# Known Conflicts and Ordering Issues

Last updated: 2026-03-13 (full re-scan, Bevy 0.18.1)

---

## CONFIRMED CONFLICT — apply_bump_velocity ordering

**File:** `src/bolt/plugin.rs`

`apply_bump_velocity` is registered as:
```rust
apply_bump_velocity.after(PhysicsSystems::BreakerCollision)
```

This means it runs AFTER `bolt_breaker_collision`. But `bolt_lost` runs AFTER `bolt_breaker_collision` too, and there is no ordering between `apply_bump_velocity` and `bolt_lost`.

The ordering chain is:
```
prepare_bolt_velocity (BoltSystems::PrepareVelocity)
  → bolt_cell_collision (after PrepareVelocity)
    → bolt_breaker_collision (after bolt_cell_collision, in_set BreakerCollision)
      → bolt_lost (after bolt_breaker_collision)
```

But `apply_bump_velocity` only says `.after(PhysicsSystems::BreakerCollision)`. This means it could run concurrently with `bolt_lost`. Both write to `BoltVelocity` on active bolts. If the bolt is lost and respawned in `bolt_lost` on the same tick as a bump, the velocity set by `apply_bump_velocity` and the respawn velocity in `bolt_lost` could conflict.

**Severity:** Low in practice (bolt lost and bump contact on the same tick is extremely unlikely), but is a formal Bevy ordering conflict on `BoltVelocity`.

**Fix:** Add `.before(bolt_lost)` or `.after(bolt_lost)` to `apply_bump_velocity` in `BoltPlugin::build`. Most correct: `.after(bolt_lost)` so bump velocity is applied last to the (possibly respawned) bolt.

---

## CONFIRMED CONFLICT — animate_bump_visual and animate_tilt_visual both write Transform on Breaker in Update

**Files:** `src/breaker/systems/bump_visual.rs`, `src/breaker/systems/tilt_visual.rs`

Both systems run in `Update`, both write `&mut Transform` on entities `With<Breaker>`. There is no ordering constraint between them.

- `animate_bump_visual` writes `transform.translation.y`
- `animate_tilt_visual` writes `transform.rotation`

**Severity:** Low — they write different fields of Transform (translation vs rotation). Bevy does not split Transform fields for conflict detection; both access `&mut Transform`. However since they actually modify different fields, there is no logical conflict, only a formal one. Bevy will serialize these (not run them in parallel) unless explicitly marked `ambiguous_with`.

**Note:** This is expected behavior for two visual-only Update systems on the same entity. No fix needed unless you want to suppress ambiguity warnings.

---

## POTENTIAL: launch_bolt has no ordering relative to hover_bolt or prepare_bolt_velocity

**File:** `src/bolt/plugin.rs`

`launch_bolt` is registered with no ordering constraints. Both `hover_bolt` and `prepare_bolt_velocity` run `.after(BreakerSystems::Move)`. `launch_bolt` runs without any explicit ordering.

On the frame the bolt is launched:
- `launch_bolt` sets velocity and removes `BoltServing`
- If `prepare_bolt_velocity` runs first, the serving bolt is skipped (filtered by `ActiveBoltFilter = Without<BoltServing>`) — no conflict
- If `hover_bolt` runs after `launch_bolt`, the newly-launched bolt (now `Without<BoltServing>`) is skipped by `ServingBoltFilter` — no conflict

**Conclusion:** Not actually a conflict because the filter predicates (`ServingBoltFilter` / `ActiveBoltFilter`) make the queries disjoint. The bolt is either serving or active, never both. No fix needed.

---

## CONFIRMED MISSING CONSUMER — CellDestroyed has no active receiver

`handle_cell_hit` sends `CellDestroyed` on every cell destruction, but no system currently reads it. This is expected for Phase 0/1 — RunPlugin (node completion detection) and UpgradesPlugin will consume it in future phases.

**Action required (future phase):** Wire RunPlugin to read CellDestroyed and send NodeCleared when all cells are gone.

---

## REGISTERED BUT UNUSED MESSAGES

These messages are registered but have neither senders nor receivers yet:
- `NodeCleared` (RunPlugin)
- `TimerExpired` (RunPlugin)
- `UpgradeSelected` (UiPlugin)

All expected. Will be wired in future phases.

---

## ORDERING REFERENCE — Full FixedUpdate Chain (PlayingState::Active)

Implicit ordering (no parallelism possible due to constraints):
```
update_bump  (BreakerPlugin, no ordering from anything)
  → move_breaker (.after(update_bump), BreakerSystems::Move)
    → update_breaker_state (.after(move_breaker))
    → hover_bolt (.after(BreakerSystems::Move))
    → prepare_bolt_velocity (.after(BreakerSystems::Move), BoltSystems::PrepareVelocity)
      → bolt_cell_collision (.after(BoltSystems::PrepareVelocity))
        → bolt_breaker_collision (.after(bolt_cell_collision), PhysicsSystems::BreakerCollision)
          → bolt_lost (.after(bolt_breaker_collision))
          → grade_bump (.after(update_bump) AND .after(PhysicsSystems::BreakerCollision))
          → apply_bump_velocity (.after(PhysicsSystems::BreakerCollision))
          → track_bump_result (.after(PhysicsSystems::BreakerCollision), dev only)
            → perfect_bump_dash_cancel (.after(grade_bump))
            → spawn_bump_grade_text (.after(grade_bump))
            → spawn_whiff_text (.after(grade_bump))
```

Systems with NO explicit ordering (run in parallel with each other unless resource/component conflicts force serialization):
- `launch_bolt` — reads InputActions (shared read with move_breaker, update_breaker_state), writes BoltVelocity on ServingBoltFilter
- `spawn_bolt_lost_text` — reads BoltLost message only (written by bolt_lost)
- `trigger_bump_visual` — reads InputActions, Commands only
- `handle_cell_hit` — reads BoltHitCell message, no overlap with physics query
```
