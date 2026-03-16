---
name: known-conflicts
description: Known query conflicts, ordering issues, and missing constraints identified in the brickbreaker system map (as of 2026-03-16, updated post-cleanup)
type: reference
---

# Known Conflicts and Ordering Issues

Last updated: 2026-03-16 (post-architecture-cleanup re-scan)

---

## RESOLVED — apply_bump_velocity now has correct ordering vs bolt_lost

Previously `apply_bump_velocity` and `bolt_lost` were both `.after(BreakerCollision)` with no
ordering between them, creating a `BoltVelocity` write race.

**Current state (bolt/plugin.rs lines 40–42):**
```rust
apply_bump_velocity
    .after(PhysicsSystems::BreakerCollision)
    .before(PhysicsSystems::BoltLost),
```

`bolt_lost` is now in `PhysicsSystems::BoltLost` set. `apply_bump_velocity` runs
`.before(PhysicsSystems::BoltLost)`, so it always completes before `bolt_lost` executes.
The fix is in place and correct. No remaining conflict here.

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

## RESOLVED — RunPlugin FixedUpdate ordering

Previous state: all four run systems in a flat unordered tuple.

**Current state (run/plugin.rs):**
```rust
handle_node_cleared.after(NodeSystems::TrackCompletion),
handle_timer_expired
    .after(NodeSystems::TickTimer)
    .after(handle_node_cleared),
handle_run_lost,
```

And in node/plugin.rs:
```rust
track_node_completion.in_set(NodeSystems::TrackCompletion),
tick_node_timer.in_set(NodeSystems::TickTimer),
```

Result: `track_node_completion` → `handle_node_cleared` is now guaranteed same-tick.
`tick_node_timer` → `handle_timer_expired` is now guaranteed same-tick.
`handle_timer_expired` also runs `.after(handle_node_cleared)` — prevents both firing the
same tick from interfering. No remaining ordering concern in the run group.

---

## LOW SEVERITY — handle_cell_hit has no ordering vs track_node_completion

**Files:** `src/cells/plugin.rs`, `src/run/node/plugin.rs`

`handle_cell_hit` (CellsPlugin, FixedUpdate) sends `CellDestroyed`.
`track_node_completion` (NodePlugin, FixedUpdate, `NodeSystems::TrackCompletion`) reads it.

No cross-plugin ordering constraint. Since messages persist across frames, a one-tick delay
is safe — `track_node_completion` reads the message on tick N+1 if it runs before
`handle_cell_hit` on tick N.

**Severity:** Low — one-tick delay on cell destruction counting. Node clear detection delayed
by at most one fixed-update tick (~16ms at 60Hz). Completely imperceptible.

**Fix (optional):** Add `handle_cell_hit.before(NodeSystems::TrackCompletion)` in CellsPlugin.
Not currently needed.

---

## NEW — handle_run_lost has no ordering vs handle_node_cleared / handle_timer_expired

**File:** `src/run/plugin.rs`

`handle_run_lost` is registered in the same `.run_if(in_state(PlayingState::Active))` tuple as
`handle_node_cleared` and `handle_timer_expired`, but with no `.after()` constraints.

All three write `ResMut<RunState>` and `ResMut<NextState<GameState>>`. Bevy will serialize
them automatically on shared mutable resource access, but execution order is non-deterministic.

**Practical concern:** If a `RunLost` and a `NodeCleared`/`TimerExpired` message arrive on the
same tick (e.g., last bolt lost on the same tick as the last cell destroyed), the outcome
depends on which system runs first:
- `handle_node_cleared` first → `RunState::Won`, then `handle_run_lost` checks
  `run_state.outcome == InProgress` → false, skips. Correct — win takes priority.
- `handle_run_lost` first → `RunState::Lost`, then `handle_node_cleared` overwrites
  `RunState::Won` and sets `GameState::NodeTransition`. Incorrect — loss then falsely wins.

**Severity:** Low in practice — simultaneous last-cell-cleared + bolt-lost is an edge case.
`handle_life_lost` is an observer (runs immediately on `LoseLifeRequested`) and writes
`RunLost` as a message. `handle_cell_hit` sends `CellDestroyed`, which `track_node_completion`
reads next tick, then `handle_node_cleared` reads `NodeCleared` the tick after. The
multi-tick propagation delay makes the simultaneous scenario very unlikely.

**Fix (recommended):** Add `.after(handle_node_cleared)` and `.after(handle_timer_expired)` to
`handle_run_lost`, so a win always takes priority over a loss if both resolve on the same tick.
In `run/plugin.rs`:
```rust
handle_run_lost
    .after(handle_node_cleared)
    .after(handle_timer_expired),
```

---

## POTENTIAL: launch_bolt has no ordering vs hover_bolt / prepare_bolt_velocity

**File:** `src/bolt/plugin.rs`

`launch_bolt` has no ordering. Analysis shows this is NOT a real conflict because filter
predicates (`ServingBoltFilter` / `ActiveBoltFilter`) make the queries disjoint on launch frame.
No fix needed.

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
          → apply_bump_velocity (.after(BreakerCollision), .before(BoltLost))  ← FIXED
          → grade_bump (.after(update_bump) AND .after(PhysicsSystems::BreakerCollision))
          → bridge_bump (.after(PhysicsSystems::BreakerCollision), conditional)
          → track_bump_result (.after(PhysicsSystems::BreakerCollision), dev only)
          → bolt_lost (.after(bolt_breaker_collision), PhysicsSystems::BoltLost)
            → bridge_bolt_lost (.after(PhysicsSystems::BoltLost), conditional)
              → [observer: handle_life_lost] (immediate on LoseLifeRequested trigger)
                → sends RunLost message
          → grade_bump continuations: perfect_bump_dash_cancel, spawn_bump_grade_text, spawn_whiff_text (.after(grade_bump))
```

NodePlugin FixedUpdate (ordered chains):
```
track_node_completion (NodeSystems::TrackCompletion)  ← handle_cell_hit unordered vs this
  → [message: NodeCleared] → handle_node_cleared (.after(NodeSystems::TrackCompletion))
tick_node_timer (NodeSystems::TickTimer)
  → [message: TimerExpired] → handle_timer_expired (.after(NodeSystems::TickTimer), .after(handle_node_cleared))
```

RunPlugin FixedUpdate (unordered vs above node chain):
```
handle_run_lost  ← UNORDERED vs handle_node_cleared / handle_timer_expired (low severity)
```

Parallel/unordered systems (run in indeterminate order each tick):
- `launch_bolt` — writes BoltVelocity on ServingBoltFilter (disjoint from physics chain)
- `spawn_bolt_lost_text` — reads BoltLost message only
- `trigger_bump_visual` — reads InputActions, Commands only
- `handle_cell_hit` — reads BoltHitCell, writes CellHealth, sends CellDestroyed
