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

## ValidBreakerState — Dashing → Settling is a legal transition (FIXED)

This is already resolved: `check_valid_breaker_state` lists `(Dashing, Braking | Settling)` as legal.

---

## ValidBreakerState — Braking → Dashing appears illegal due to system ordering race (NEW, 2026-03-19)

**Invariant fires:** `prism_scatter_stress` copy 25/32, frame 3257
**Scenario:** Prism + Scatter + Chaos(seed=47, action_prob=0.3)

**Root cause:** No explicit ordering between `check_valid_breaker_state` (invariant
checker group, before `PhysicsSystems::BoltLost`) and `update_breaker_state` (after
`BreakerSystems::Move`). `handle_idle_or_settling` in `dash.rs` performs TWO state
transitions in one call: timer expiry gives `Settling → Idle`, then dash input immediately
gives `Idle → Dashing`. Both happen synchronously with no frame boundary.

When the checker samples before `update_breaker_state` on frame N (stores `Braking`),
then `update_breaker_state` runs (`Braking → Settling`), then on frame N+1 the checker
samples after `update_breaker_state` (which ran `Settling → Idle → Dashing` in one call),
the checker records: previous=`Braking`, current=`Dashing` → `Braking → Dashing` is not
in the legal transition set → VIOLATION.

**Intermittency:** 1/32 copies fail (stress parallelism causes non-deterministic scheduling).
Does not fire in single-scenario (`-s`) mode or the non-stress `prism_scatter` scenario.

**Fix direction:** Add explicit ordering in lifecycle/mod.rs:
`check_valid_breaker_state.after(update_breaker_state)` (or after a new
`BreakerSystems::StateMachine` set that `update_breaker_state` is placed in).

**Confidence: HIGH** — full code path traced:
`breaker-game/src/breaker/systems/dash.rs:110-146` (double-transition in single call),
`breaker-scenario-runner/src/lifecycle/mod.rs:149-177` (checker ordering),
`breaker-game/src/breaker/plugin.rs:46-68` (no mutual ordering between checker and state machine)

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

## BreakerPositionClamped — never fires in breaker_oob_detection self-test (UPDATED analysis 2026-03-20)

**Scenario:** `breaker_oob_detection` — Aegis + Corridor, breaker teleported to x=2000.0 via `debug_setup`, `disable_physics=true`. Expects BOTH `BreakerInBounds` AND `BreakerPositionClamped` to fire.

**Observed:** `BreakerInBounds` fires 83 times (frames 38-120). `BreakerPositionClamped` never fires.

**Root cause (corrected 2026-03-20):** System ordering ambiguity in the `FixedUpdate` invariant block. The block is registered as an UNORDERED tuple (no `.chain()`):

```
(enforce_frozen_positions, ..., check_breaker_in_bounds, ..., check_breaker_position_clamped, ...)
```

Both `enforce_frozen_positions` (writes `&mut Transform`) and the two checkers (read `&Transform`) have a data conflict. Bevy's ambiguity resolver may schedule `check_breaker_position_clamped` BEFORE `enforce_frozen_positions`.

When `check_breaker_position_clamped` runs first:
- `move_breaker` already ran (before the invariant block, via `.after(update_breaker_state)`)
- `move_breaker` clamped x=2000 to `max_x = right() - half_width = 720 - 108 = 612`
- checker tests: `612 > 612 + 1.0 = 613` → FALSE → no violation

When `check_breaker_in_bounds` runs AFTER `enforce_frozen_positions` (Bevy happened to order it correctly for this checker):
- position is restored to x=2000
- `2000 > 720 + 50 = 770` → TRUE → fires

**Why violations start at frame 38:** `apply_debug_setup` (which sets x=2000 and inserts `ScenarioPhysicsFrozen`) runs in `OnEnter(Playing)` AFTER `tag_game_entities` in the same `.chain()`. But `tag_game_entities` inserts `ScenarioTagBreaker` via deferred `Commands` — no `ApplyDeferred` between them. So `apply_debug_setup` finds no breaker on the FIRST `OnEnter(Playing)`. The game cycles MainMenu → Playing → ChipSelect → NodeTransition → Playing (second entry). On the second `OnEnter(Playing)`, tags are already flushed from pass 1, so `apply_debug_setup` succeeds. This second entry occurs ~frame 38.

**Evidence:**
- `breaker-scenario-runner/src/lifecycle/mod.rs:149-177` — unordered invariant tuple (no `.chain()`)
- `breaker-scenario-runner/src/invariants/checkers/breaker_position_clamped.rs:22` — tolerance check `x > max_x + 1.0`
- `breaker-game/src/breaker/systems/move_breaker.rs:75-78` — clamp to exactly `max_x`
- `breaker-game/assets/config/defaults.playfield.ron` — `width: 1440.0`, so `right()=720`
- `breaker-game/assets/config/defaults.breaker.ron` — `width: 216.0`, so `half_width=108`, `max_x=612`

**Note:** Previous diagnosis (2026-03-19) claimed `BreakerWidth` was never inserted due to missing `ApplyDeferred` in `breaker/plugin.rs`. This is INCORRECT — `ApplyDeferred` IS present in the chain (plugin.rs lines 38). The real cause is system ordering.

**Fix direction:** Add `.chain()` to the FixedUpdate invariant block in `breaker-scenario-runner/src/lifecycle/mod.rs:159-173` so `enforce_frozen_positions` is guaranteed to run before all checkers. Also add `ApplyDeferred` between `tag_game_entities` and `apply_debug_setup` in the `OnEnter(Playing)` chain (lifecycle/mod.rs:133-143) so debug teleport works on the first `OnEnter` instead of requiring a second entry.

**Confidence: HIGH**

---

## Note: `invariants` field in RON is documentation-only

All invariant check systems run for every scenario unconditionally. The
`invariants: [...]` field in `.scenario.ron` files does NOT filter which
checks are active. Confirmed by reading `lifecycle.rs:138-165`.
