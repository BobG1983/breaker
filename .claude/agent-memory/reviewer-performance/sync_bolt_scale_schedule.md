---
name: sync_bolt_scale schedule
description: sync_bolt_scale runs in FixedUpdate every tick with no change-detection guard — runs unconditionally but is cheap at 1 bolt
type: project
---

`sync_bolt_scale` is registered in `FixedUpdate` with `run_if(in_state(PlayingState::Active))` in `BoltPlugin` (bolt/plugin.rs line 88). Tests register it in `FixedUpdate` for convenience.

The system runs unconditionally every `FixedUpdate` tick. There is no change-detection guard (e.g., `Changed<BaseRadius>`, `Changed<ActiveSizeBoosts>`). The iteration body is cheap: a few multiplications and two float assignments.

At 1 bolt entity, this is unmeasurably cheap per tick. The `ActiveSizeBoosts` `Vec<f32>` that feeds `multiplier()` calls `.iter().product()` in the loop body — again, at O(1) boost entries and 1 entity, negligible.

**Compared to sync_breaker_scale**: breaker runs in `Update` (visual), bolt runs in `FixedUpdate`. Both are correct for their respective domains — bolt physics need the fixed timestep; breaker scale is visual-only. This asymmetry is acceptable.

**How to apply:** Do not flag the unconditional run or the FixedUpdate placement as issues. The system is correctly placed and has acceptable cost at current entity scale. If bolt count ever grows to dozens simultaneously (e.g., a multiball mechanic), consider a `Changed<>` guard as an optimization.
