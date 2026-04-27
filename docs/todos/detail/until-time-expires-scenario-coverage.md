# Until(TimeExpires) scenario coverage gaps

## Summary
Five coverage gaps surfaced by `reviewer-scenarios` after the `Until(TimeExpires)` wiring fix landed. None blocked the merge; all are follow-up work.

## Context
The `feature/until-time-expires-wiring` branch (merged 2026-04-27) wired `Until(TimeExpires(d), ...)` end-to-end — including direct-evaluate routing for `When(_, Until(...))` / `Once(_, Until(...))` and source-keyed timer entries via `EffectTimerExpired.source: SourceId`. The two existing Until(TimeExpires) scenarios (`overclock_until_speed`, `damage_boost_until_reversal`) use `initial_effects`, which bypasses chip dispatch entirely. Several runtime paths now have working code but no scenario exercising them under chaos input.

## Scope
- In:
  - HIGH: `surge_chip_dispatch.scenario.ron` — chip-selections-driven Surge with `AlwaysPerfect` input, exercising `chip_selections → ChipSelected → chip dispatch → Stamp(Bolt, ...)` path
  - HIGH: `overclock_chip_dispatch.scenario.ron` — Overclock chip selection driving `Until(TimeExpires, Sequence([SpeedBoost, DamageBoost]))` reversal (Shape 2 path in `reverse_scoped_tree`)
  - MEDIUM: `concurrent_same_duration_untils.scenario.ron` — two chips with identical `TimeExpires(d)` durations on the same bolt, validating that `on_time_expires` source-filter correctly disambiguates
  - MEDIUM: `surge_stack_max_taken.scenario.ron` — pre-seed 3 Surge stacks via `initial_chips`, exercise `(duration, source)` idempotency in `arm_time_expires_timer`
  - LOW: scenario for `Once(_, Until(TimeExpires(d), ...))` delegation path — load-bearing `remove_effect` ordering before `ensure_until_bound`
- Out:
  - New invariants (the existing `BoltSpeedAccurate + NoEntityLeaks + NoNaN` set adequately covers Shape 1; the missing `DamageBoostClamped` invariant for Shape 2 is a separate pre-existing gap, not scope here)
  - Changes to existing scenarios (`overclock_until_speed`, `damage_boost_until_reversal` are adequate for the `initial_effects` injection path)

## Dependencies
- Depends on: feature/until-time-expires-wiring (merged 2026-04-27)
- Blocks: nothing

## Notes
- Existing scenarios test correct behavior (boost reverses, speed returns to base) — quality is adequate for the `initial_effects` path.
- The chip dispatch path (`auto_skip_chip_select` → `ChipSelected` → chip dispatch wiring) has zero scenario coverage today. A regression there would not be caught.
- `node_end_speed_purge` is the regression scenario for the originally-fixed bug; remains the primary correctness signal.
- Source: reviewer-scenarios run on 2026-04-27, pre-merge full-tier audit of `feature/until-time-expires-wiring`.

## Status
`ready`
