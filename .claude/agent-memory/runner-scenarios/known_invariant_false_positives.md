---
name: resolved_checker_bugs
description: Past scenario runner bugs that caused incorrect invariant results — fixed, kept as regression reference
type: project
---

# Resolved Checker Bugs

Past bugs in the scenario runner's invariant checkers that caused incorrect results.
All resolved. Kept as reference to prevent regression.

## BoltInBounds — bottom boundary check (RESOLVED)

**Bug:** Checker flagged bolts exiting through the open bottom (life-loss mechanic).
**Root cause:** Bottom boundary was checked, but the game has no floor wall by design.
**Fix:** Removed bottom boundary check entirely from `check_bolt_in_bounds`.
**File:** `breaker-scenario-runner/src/invariants/checkers/bolt_in_bounds.rs`

## ValidBreakerState — Braking → Dashing ordering race (RESOLVED)

**Bug:** Checker sampled breaker state before `update_breaker_state` ran, then on the
next frame saw the final state after a double-transition (`Braking → Settling → Idle → Dashing`),
recording an illegal `Braking → Dashing` transition.
**Root cause:** No ordering constraint between checker and `update_breaker_state`.
**Fix:** Added `.after(update_breaker_state)` to invariant block in lifecycle plugin.
**File:** `breaker-scenario-runner/src/lifecycle/mod.rs`

## TimerMonotonicallyDecreasing — same-duration node transitions (RESOLVED)

**Bug:** When consecutive nodes had the same timer duration, `remaining` jumped back near
`total` on the new node but `total` didn't change, so the checker saw an illegal increase.
**Root cause:** Checker only detected node transitions via `total` change.
**Fix:** Added `near_total` detection — if `remaining` jumps to within 1.0 of `total`,
treat it as a node transition reset.
**File:** `breaker-scenario-runner/src/invariants/checkers/timer_monotonically_decreasing.rs`

## NoEntityLeaks — baseline sampled during Loading (RESOLVED)

**Bug:** Baseline entity count was sampled at a fixed frame, which could be during
`GameState::Loading` when few entities existed. Post-Loading entity spawns looked like leaks.
**Root cause:** Frame-based baseline sampling, not gameplay-aware.
**Fix:** Baseline now sampled on `SpawnNodeComplete` message (all gameplay entities spawned).
**File:** `breaker-scenario-runner/src/invariants/checkers/no_entity_leaks.rs`

## BreakerPositionClamped — checker ran before enforce_frozen_positions (RESOLVED)

**Bug:** In self-test scenarios with `disable_physics: true`, the checker could run before
`enforce_frozen_positions` restored the teleported position, seeing the clamped position instead.
**Root cause:** Invariant block was an unordered tuple — no guaranteed execution order.
**Fix:** Changed invariant block to `.chain()` with `enforce_frozen_positions` first.
**File:** `breaker-scenario-runner/src/lifecycle/mod.rs`

## Invariant checkers firing during Loading state (RESOLVED)

**Bug:** Under parallel I/O contention, checkers ran before `GameState::Playing` was entered,
finding uninitialized or missing entities and producing spurious violations.
**Root cause:** Invariant block had no `run_if` guard for gameplay readiness.
**Fix:** Added `.run_if(|stats| stats.entered_playing)` to invariant block.
**File:** `breaker-scenario-runner/src/lifecycle/mod.rs`

## tick_scenario_frame counting during Loading (RESOLVED)

**Bug:** Frame counter ticked from app startup. In parallel mode, frame_mutations at
frame 30 were missed because Playing wasn't entered yet.
**Root cause:** `tick_scenario_frame` had no `entered_playing` gate.
**Fix:** Added `.run_if(entered_playing)` to `(tick_scenario_frame, check_frame_limit).chain()`.
**File:** `breaker-scenario-runner/src/lifecycle/mod.rs`

---

## Unresolved Game Bugs (correctly detected by scenario runner)

### BoltInBounds — prism_scatter sustained violation

**Scenario:** Prism + Scatter layout + Chaos(seed=47, action_prob=0.3)
**Violation:** BoltInBounds x1570 frames 6830..7689
**Status:** Real game physics bug. Multiple extra bolts simultaneously outside playfield
bounds for sustained period. Root cause not yet confirmed — likely degenerate reflection
state in bolt physics under Scatter layout with many cells destroyed.
**This is NOT a scenario runner bug.** The runner correctly detects it.
