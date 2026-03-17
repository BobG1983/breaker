---
name: FixedUpdate Testing Patterns
description: Verified API for testing FixedUpdate systems in Bevy 0.18 — accumulate_overstep vs advance_by, schedule registration, and input clearing
type: reference
---

# Testing FixedUpdate Systems (verified v0.18.0 source)

## The Only Correct Pattern

Use `accumulate_overstep` to trigger FixedUpdate ticks in tests. This is the Bevy-documented
test helper. Register systems in `FixedUpdate` in tests (matching production) — do NOT move
them to `Update` as a workaround.

```rust
let timestep = app.world().resource::<Time<Fixed>>().timestep();
app.world_mut().resource_mut::<Time<Fixed>>().accumulate_overstep(timestep);
app.update(); // FixedUpdate fires exactly once
```

To trigger N ticks: `accumulate_overstep(timestep * N)` before a single `app.update()`.

To have a frame where FixedUpdate does NOT tick: call `app.update()` without accumulating overstep.

## Why NOT advance_by

- `advance_by` moves the clock forward but does NOT deposit into the overstep accumulator
- `run_fixed_main_schedule` reads the overstep accumulator to decide how many ticks to run
- Using `advance_by` alone will NOT cause FixedUpdate to fire — the tick is silently skipped
- This was the root cause of a bug where bump inputs were lost when FixedUpdate skipped a frame

## Key API Facts

- `accumulate_overstep(&mut self, delta: Duration)` — documented test helper; scheduler reads this
- `advance_by(&mut self, delta: Duration)` — sets delta/elapsed on the clock; does NOT trigger FixedUpdate
- Default timestep: 64 Hz (15625 microseconds) — `Time::<Fixed>::DEFAULT_TIMESTEP`
- `delta()` on `Time<Fixed>` always equals `timestep()` when a tick fires (not variable)
- Both `Time<Fixed>` and `Time<Virtual>` must exist in world; `MinimalPlugins` includes `TimePlugin`

## Related Fix: Input Clearing Schedule

Input state that must survive a frame where FixedUpdate does NOT tick should be cleared in
`FixedPostUpdate`, NOT `PreUpdate`. Clearing in `PreUpdate` loses inputs on frames where
FixedUpdate is skipped.

## Sources

- `crates/bevy_time/src/fixed.rs` v0.18.0
- `crates/bevy_app/src/main_schedule.rs` v0.18.0
- Verified via debugging: bump inputs lost when FixedUpdate skipped a frame
