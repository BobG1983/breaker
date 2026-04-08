---
name: sync_bolt_scale schedule
description: sync_bolt_scale runs in Update (not FixedUpdate) gated by in_state(NodeState::Playing) — visual sync, correct placement
type: project
---

`sync_bolt_scale` is registered in `Update` with `run_if(in_state(NodeState::Playing))` in `BoltPlugin` (`breaker-game/src/bolt/plugin.rs` line 94). Tests register it in `Update` for convenience.

The system runs unconditionally each `Update` frame during `NodeState::Playing`. There is no change-detection guard (e.g., `Changed<BaseRadius>`, `Changed<ActiveSizeBoosts>`). The iteration body is cheap: a few multiplications and two float assignments.

The `Without<Birthing>` filter (added in the birthing animation branch) means birthing bolts are skipped — their Scale2D is driven by `tick_birthing` in FixedUpdate instead.

At 1 bolt entity, this is unmeasurably cheap per frame. The `ActiveSizeBoosts` `Vec<f32>` that feeds `multiplier()` calls `.iter().product()` — at O(1) boost entries and 1 entity, negligible.

**Compared to sync_breaker_scale**: breaker runs in `Update` (visual), bolt scale also runs in `Update`. Both are visual-sync systems — correct placement.

**How to apply:** Do not flag the unconditional run, the Update placement, or the Without<Birthing> filter as issues. The system is correctly placed. If bolt count ever grows to dozens simultaneously, consider a `Changed<>` guard.
