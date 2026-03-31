---
name: ChainArcCountReasonable self-test scenarios fail — threshold too high for current game
description: chain_lightning_arc_lifecycle and chain_lightning_chaos self-test scenarios expect ChainArcCountReasonable to fire, but most stress copies never trigger it (violations=0); max_chain_arc_count=50 is too high relative to what the current game generates per seed.
type: project
---

`chain_lightning_arc_lifecycle` and `chain_lightning_chaos` have `expected_violations: Some([ChainArcCountReasonable])`. Both fail in most stress copies with "expected violation ChainArcCountReasonable never fired" (violations=0).

**Why:** The self-test scenarios were designed so that arc accumulation would reliably exceed max=50. With the current game:
- `chain_lightning_chaos`: arc_speed=200.0 (default), arcs=3, range=96.0 — arcs complete in ~0.5 ticks each at 200 units/sec; chains clean up before accumulating past 50
- `chain_lightning_arc_lifecycle`: arc_speed=50.0 (slow), arcs=5, range=112.0 — longer arc lifetime, but `allow_early_end: false` causes node resets (CleanupOnNodeExit) to clear all chains/arcs when all cells die
- Dense layout with aggressive chaos (action_prob 0.6-0.8) destroys cells fast enough that node resets prevent sustained accumulation
- The threshold=50 is only reached in some seed variations; most seeds complete node cycles before enough arcs accumulate

**Observed across multiple --all runs:**
- `chain_lightning_chaos`: 8/16 copies fail with "never fired" (violations=0 for all failing copies)
- `chain_lightning_arc_lifecycle`: 3/16 or 13/16 copies fail (highly variable between runs) with "never fired"
- Copies that DO fire the violation show counts in the thousands (1270-2704) once accumulation starts
- Individual `-s` runs also fail: 7/16 and 13/16 in successive individual runs

**The "1 failure(s)" mystery in first --all run was resolved:**
The copies showing `violations=1543` with `FAIL: 1 failure(s)` in the first --all run were different copies from the same stress scenario. The `ChainArcCountReasonable` that fired (1543 times) was expected and generated no fail reason. The actual fail reason for those 4 copies was `NoEntityLeaks` (despawned entity errors from `tick_chain_lightning` trying to despawn arc entities already cleaned up by `CleanupOnNodeExit` during node reset). The despawned-entity WARN from `bevy_ecs::error::handler` co-occurs with these failures — it's a real entity lifecycle bug where the chain's `ArcTraveling` state holds a stale `arc_entity` reference across a node reset.

**Two distinct bugs:**
1. **Self-test threshold**: `max_chain_arc_count=50` not reliably triggered — scenario RON needs tuning
2. **Entity lifecycle bug**: `tick_chain_lightning` doesn't handle arc entity despawn on node reset; the chain's `ArcTraveling { arc_entity, ... }` state persists across `CleanupOnNodeExit`, causing double-despawn attempts on the stale `arc_entity` reference. This triggers `NoEntityLeaks` in copies where arcs happen to be in-flight at node end.

**Category:** Game bugs (both bugs are in game code, not runner)

**Resolution needed for bug 1:** Lower `max_chain_arc_count` in scenario RON files OR increase `arcs`/decrease `arc_speed` to reliably exceed threshold; alternatively, remove `ChainArcCountReasonable` from `expected_violations` and add `allow_early_end: true` so cells-cleared end counts as a pass.

**Resolution needed for bug 2:** `tick_chain_lightning` must check if `arc_entity` still exists before attempting despawn in the `ArcTraveling` arrival branch. Use `commands.entity(arc_entity).try_despawn()` or check existence via `world.get_entity(arc_entity).is_ok()`.

**Files:**
- `/Users/bgardner/dev/brickbreaker/breaker-scenario-runner/scenarios/mechanic/chain_lightning_arc_lifecycle.scenario.ron`
- `/Users/bgardner/dev/brickbreaker/breaker-scenario-runner/scenarios/stress/chain_lightning_chaos.scenario.ron`
- `/Users/bgardner/dev/brickbreaker/breaker-game/src/effect/effects/chain_lightning/effect.rs` — `tick_chain_lightning` ArcTraveling arrival branch (line ~253)
