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

## NoEntityLeaks — fires in parallel subprocess mode when assets load slowly

**Root cause:** `check_no_entity_leaks` samples the entity count at fixed-update
frame 60 as baseline. In parallel subprocess mode (`--all`, 27 processes on macOS
under I/O contention), `bevy_asset_loader` can take >60 fixed-update frames to
finish loading RON assets. The baseline is captured in `GameState::Loading` with
only ~7 system entities. After `Playing` is entered (post-frame-60), ~49 game
entities spawn. Every 120-frame check then fires `count=49 > baseline×2=14`.

In `--serial` and single-scenario (`-s`) mode this does not occur — assets load
within frame 60 reliably when only one process has disk I/O.

**Observed violation message:** `NoEntityLeaks FAIL frame=120 count=49 baseline=7 (>14 threshold)`

**Fix direction:** Defer baseline sampling until `GameState::Playing` has been
entered. In `check_no_entity_leaks` (`invariants.rs:448-477`), add a check for
`stats.entered_playing` (or read `GameState` resource) before setting the
baseline, rather than using a fixed frame-60 sample.

**Confidence: HIGH**

---

## BoltInBounds — fires in parallel subprocess mode under I/O contention (two manifestations)

**Also caused by the same asset-loading race as NoEntityLeaks above.**

**Manifestation A — self-test scenario fails health checks (bolt_oob_detection):**
When asset loading races in a subprocess, `ScenarioTagBolt` / `ScenarioTagBreaker` are never
attached (the lifecycle plugin hasn't run tagging yet). The verdict health checks report
`bolts=0, breakers=0` and fire "no bolts were tagged", "no invariant checks ran", and
"expected violation BoltInBounds never fired" (since `check_bolt_in_bounds` found no tagged
entities). Passes reliably with `-s bolt_oob_detection`.

**Manifestation B — stress scenario fires real BoltInBounds (prism_concurrent_hits, chrono_penalty_stress):**
Under I/O contention, asset loading delays push game initialization into a transient state
where bolt physics simulate briefly before all walls/layout are ready. Bolts escape the
playfield boundary during this window. Violations are sustained for hundreds of frames
(e.g. x2420 frames 13613..15000, x3729 frames 1232..3207). Always passes with `-s <name>`.

**Non-deterministic:** A different scenario fails on each `--all` run. All pass when run
individually. Confirmed across 3 separate `--all` runs on 2026-03-19.

**Fix direction (same as NoEntityLeaks):** Defer scenario lifecycle initialization until
`GameState::Playing` has been confirmed entered; do not allow invariant checks to run
during the `Loading` phase when entities may not yet exist.

---

## BoltInBounds — prism_scatter sustained violation (pre-existing, unrelated to asset race)

**First observed:** 2026-03-19 (Local<Vec> refactor run)
**Violation:** BoltInBounds x1570 frames 6830..7689 (~1.8/frame for 859 frames)
**Scenario:** Prism + Scatter layout + Chaos(seed=47, action_prob=0.3)

Multiple extra bolts simultaneously outside playfield bounds for sustained period.
The `Local<Vec>` change in `bolt_lost` / `handle_cell_hit` is NOT the cause — both
systems call `.clear()` at frame start (fully equivalent to per-frame allocation).

Root cause not yet confirmed. Working hypothesis: under Scatter layout with many cells
destroyed, extra Prism bolts encounter a degenerate reflection state (velocity
compounding or wall-exit without `bolt_lost` triggering). Unique to Scatter + Prism +
seed 47; does not appear in prism_stress, prism_fortress, prism_concurrent_hits,
prism_accumulation, or prism_bolt_stabilization.

**Confidence: MEDIUM** — physics root cause not yet read; `bolt_cell_collision` and
wall-reflection math in `shared::math` are the most likely suspects.

---

## Note: `invariants` field in RON is documentation-only

All invariant check systems run for every scenario unconditionally. The
`invariants: [...]` field in `.scenario.ron` files does NOT filter which
checks are active. Confirmed by reading `lifecycle.rs:138-165`.
