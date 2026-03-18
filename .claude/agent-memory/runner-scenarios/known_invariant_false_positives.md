---
name: known_invariant_false_positives
description: Invariant checkers that fire on valid gameplay — root causes and fix directions
type: project
---

# Known Invariant False Positives

These invariants fire on valid/intentional gameplay. Confirmed by source reading
and verbose scenario output.

## BoltInBounds — fires on every life-loss event

**Root cause:** No floor wall exists by design (`spawn_walls.rs` spawns only
left, right, ceiling). The bolt exits through the bottom to model life loss.
The invariant threshold (`bottom - radius - 1`) and `bolt_lost` threshold
(`bottom - radius`) are 1 px apart. The invariant fires once per life-loss event
at the single frame when the bolt is between -554 and -555 (for 14px radius, 60px
half-height playfield).

**Every scenario** that includes `BoltInBounds` will fail on any scenario that
allows bolt/life-loss mechanics. All 21 failing scenarios are affected.

**Fix direction:** Exclude the bottom wall check from `check_bolt_in_bounds`
in `breaker-scenario-runner/src/invariants.rs:251`, OR raise the margin to
exactly `radius` (not `radius + 1`) to match the bolt_lost threshold.

---

## ValidBreakerState — Dashing → Settling is a legal transition

**Root cause:** `perfect_bump_dash_cancel` system in
`breaker-game/src/breaker/systems/bump.rs:177` forcibly sets state to `Settling`
when a perfect bump fires during `Dashing`. This is the "dash cancel" mechanic.
The invariant only allows `Dashing → Braking`.

**Fix direction:** Add `(BreakerState::Dashing, BreakerState::Settling)` to
the legal transitions in `check_valid_breaker_state` at
`breaker-scenario-runner/src/invariants.rs:53-61`.

---

## TimerMonotonicallyDecreasing — same-duration node transitions not detected

**Root cause:** The fix tracks `(remaining, total)` and resets tracking when
`total` changes. But all nodes in Aegis/Chrono/Prism use the same timer
duration (60s), so `total` does not change across node transitions. The
invariant fires on the first tick of a new node when `remaining` resets from
~40-54s back to 59.984s (60s minus one fixed tick).

**Fix direction:** Instead of detecting node transitions via `total` change,
add a node generation counter (new `NodeGeneration` resource that increments
on each `OnEnter(Playing)`), and track `(remaining, generation)` in the Local.
Alternatively: reset tracking whenever `current > prev_remaining` — simpler
but loses subtle increase-bug detection.

See: `breaker-scenario-runner/src/invariants.rs:86-117`,
`breaker-game/src/run/node/systems/tick_node_timer.rs:36-38`

---

## Note: `invariants` field in RON is documentation-only

All invariant check systems run for every scenario unconditionally. The
`invariants: [...]` field in `.scenario.ron` files does NOT filter which
checks are active. Confirmed by reading `lifecycle.rs:138-165`.
