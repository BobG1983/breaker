---
name: known-conflicts
description: Known query conflicts, ordering issues, and missing constraints identified in the brickbreaker system map (as of 2026-03-16 full re-scan)
type: reference
---

# Known Conflicts and Ordering Issues

Last updated: 2026-03-16 (full re-scan, Bevy 0.18.1)

---

## CONFIRMED CONFLICT — apply_bump_velocity has no ordering vs bolt_lost

**File:** `src/bolt/plugin.rs`

`apply_bump_velocity` is registered as `.after(PhysicsSystems::BreakerCollision)`.
`bolt_lost` runs `.after(bolt_breaker_collision)` = `.after(PhysicsSystems::BreakerCollision)`.

Both are `.after(BreakerCollision)` with no ordering between them. Both write `BoltVelocity`
on active bolt entities. On the rare tick a bolt is lost on the same tick as a bump, the
respawn velocity from `bolt_lost` and the bump-amplified velocity from `apply_bump_velocity`
race with each other on `BoltVelocity`.

**Severity:** Low in practice (simultaneous bolt loss + bump contact is extremely rare).
Formal Bevy ordering conflict on `BoltVelocity`.

**Fix:** Add `.before(bolt_lost)` or `.after(bolt_lost)` to `apply_bump_velocity` in
`BoltPlugin::build`. Most correct: `.after(bolt_lost)` so bump velocity is applied last to
the (possibly respawned) bolt.

---

## CONFIRMED CONFLICT — animate_bump_visual and animate_tilt_visual write Transform on Breaker in Update

**Files:** `src/breaker/systems/bump_visual.rs`, `src/breaker/systems/tilt_visual.rs`

Both run in `Update`, both write `&mut Transform` on `With<Breaker>`. No ordering constraint.

- `animate_bump_visual` writes `transform.translation.y`
- `animate_tilt_visual` writes `transform.rotation`

**Severity:** Low — they write different fields (translation vs rotation). No logical conflict.
Bevy will serialize these (no parallel execution) unless marked `ambiguous_with`. Expected
behavior for two visual Update systems. No fix needed unless suppressing ambiguity warnings.

---

## NEW — RunPlugin FixedUpdate systems lack intra-group ordering

**File:** `src/run/plugin.rs` (lines 43–52)

These four systems are registered in one tuple with only `.run_if()`, not `.chain()`:
```
track_node_completion
handle_node_cleared
tick_node_timer
handle_timer_expired
```

Two message-chain dependencies exist within this group:

**Pair 1:** `track_node_completion` sends `NodeCleared` → `handle_node_cleared` reads it.
**Pair 2:** `tick_node_timer` sends `TimerExpired` → `handle_timer_expired` reads it.

Since Bevy 0.18 messages persist across frames (they are NOT drained per-tick like Events),
a one-tick delay is **acceptable** — the message sent on tick N is read on tick N+1 at worst.
This means node transitions and timer loss will be delayed by at most one fixed-update tick
(~16ms at 60Hz fixed rate), which is **imperceptible in gameplay**.

Additionally, `track_node_completion` and `handle_node_cleared` both write to `RunState`
(indirectly — `handle_node_cleared` writes it, `track_node_completion` does not). No write
conflict. `tick_node_timer` and `handle_timer_expired` both write `ResMut<RunState>` — Bevy
will serialize them automatically since they share a mutable resource.

**Severity:** Low — one-tick delay on node transitions and timer expiry is unnoticeable.
However, explicitly chaining these systems would make the intent clearer and guarantee
same-tick propagation if that ever becomes desirable.

**Fix (optional):** Change the registration to:
```rust
(track_node_completion, handle_node_cleared).chain(),
(tick_node_timer, handle_timer_expired).chain(),
```
Or use a single `.chain()` across all four in the order they logically depend.

---

## NEW — handle_cell_hit has no ordering vs track_node_completion

**Files:** `src/cells/plugin.rs`, `src/run/plugin.rs`

`handle_cell_hit` (CellsPlugin, FixedUpdate) sends `CellDestroyed`.
`track_node_completion` (RunPlugin, FixedUpdate) reads `CellDestroyed`.

No cross-plugin ordering constraint. Since messages persist across frames, a one-tick delay
is safe — `track_node_completion` will read the message on tick N+1 if it runs before
`handle_cell_hit` on tick N.

**Severity:** Low — one-tick delay on cell destruction counting. Node clear will be detected
one tick after the last required cell is destroyed. Completely imperceptible.

**Fix (optional):** If same-tick cell-to-completion detection is ever required, add
`handle_cell_hit.before(track_node_completion)` in CellsPlugin or RunPlugin. Not currently needed.

---

## POTENTIAL: launch_bolt has no ordering vs hover_bolt / prepare_bolt_velocity

**File:** `src/bolt/plugin.rs`

`launch_bolt` has no ordering. Analysis shows this is NOT a real conflict because filter
predicates (`ServingBoltFilter` / `ActiveBoltFilter`) make the queries disjoint on launch frame.
No fix needed.

---

## RESOLVED — CellDestroyed had no active receiver

Previously flagged as an orphan message. Now consumed by `track_node_completion` (RunPlugin).

---

## REGISTERED BUT UNUSED MESSAGES

- `UpgradeSelected` (UiPlugin) — no sender or receiver yet. Expected. Future phases.

---

## ORDERING REFERENCE — Full FixedUpdate Chain (PlayingState::Active)

Fully constrained chain (serial):
```
update_bump  (BreakerPlugin)
  → move_breaker (.after(update_bump), BreakerSystems::Move)
    → update_breaker_state (.after(move_breaker))
    → hover_bolt (.after(BreakerSystems::Move))
    → prepare_bolt_velocity (.after(BreakerSystems::Move), BoltSystems::PrepareVelocity)
      → bolt_cell_collision (.after(BoltSystems::PrepareVelocity))
        → bolt_breaker_collision (.after(bolt_cell_collision), PhysicsSystems::BreakerCollision)
          → bolt_lost (.after(bolt_breaker_collision))
          → grade_bump (.after(update_bump) AND .after(PhysicsSystems::BreakerCollision))
          → apply_bump_velocity (.after(PhysicsSystems::BreakerCollision))  ← UNORDERED vs bolt_lost
          → track_bump_result (.after(PhysicsSystems::BreakerCollision), dev only)
            → perfect_bump_dash_cancel (.after(grade_bump))
            → spawn_bump_grade_text (.after(grade_bump))
            → spawn_whiff_text (.after(grade_bump))
```

Parallel/unordered systems (run in indeterminate order each tick):
- `launch_bolt` — writes BoltVelocity on ServingBoltFilter (disjoint from physics chain)
- `spawn_bolt_lost_text` — reads BoltLost message only
- `trigger_bump_visual` — reads InputActions, Commands only
- `handle_cell_hit` — reads BoltHitCell, writes CellHealth, sends CellDestroyed
- `track_node_completion` — reads CellDestroyed, writes ClearRemainingCount, sends NodeCleared
- `handle_node_cleared` — reads NodeCleared, writes RunState
- `tick_node_timer` — writes NodeTimer, sends TimerExpired
- `handle_timer_expired` — reads TimerExpired, writes RunState
