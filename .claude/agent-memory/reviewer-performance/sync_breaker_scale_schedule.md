---
name: sync_breaker_scale schedule placement
description: sync_breaker_scale runs in Update (visual sync) — FixedUpdate only in tests
type: project
---

`sync_breaker_scale` is registered in `Update` in `BreakerPlugin` (plugin.rs line 95), not `FixedUpdate`. The test harness uses `FixedUpdate` for convenience (tick helper) but the production schedule is `Update`.

Running in `Update` is correct: this system syncs Scale2D for rendering. It queries `&mut Scale2D` (mutable) which is needed, and has a `run_if(in_state(PlayingState::Active))` guard.

The query has 7 optional components (`ActiveSizeBoosts`, `NodeScalingFactor`, `MinWidth`, `MaxWidth`, `MinHeight`, `MaxHeight`) on a single Breaker entity. At entity count = 1, this is a non-issue.

**How to apply:** Do not flag the FixedUpdate test / Update production mismatch as a bug — it's intentional. Do not flag the optional components on a 1-entity query as fragmentation.
