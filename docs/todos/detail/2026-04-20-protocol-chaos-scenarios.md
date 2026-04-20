# Custom-system protocols — upgrade smoke scenarios to chaos coverage

**Status:** ready
**Found during:** Burnout protocol Full Verification Tier (2026-04-20, reviewer-scenarios)

## The gap

Every custom-system protocol has a single `*_smoke.scenario.ron` file that was authored when the protocol was a stub. Pattern:
- `disable_physics: true`
- `Scripted(actions: [])`
- `InjectProtocol` at frame 10, idle for 500 frames
- Comment: "When runtime behaviour lands, this scenario will start exercising it without spec changes" — aspirational, never upgraded

This pattern exists for all 8 custom-system protocols whose runtimes have now landed:
- Burnout (just merged — highest urgency)
- Reckless Dash
- Echo Strike
- Fission
- Iron Curtain
- Debt Collector
- Siphon
- Greed

None of these smoke scenarios exercise the actual protocol behavior. Chaos/stress coverage is zero.

## What's missing per protocol

Each protocol needs (at minimum):
1. Physics-enabled playthrough with `Perfect(AlwaysPerfect)` or `Chaos` input
2. A layout with cells the bolt can hit
3. `InjectProtocol` at frame N (existing pattern)
4. The full standard invariant set: `NoNaN`, `BoltInBounds`, `BreakerInBounds`, `NoEntityLeaks`, `BoltSpeedAccurate`
5. Burnout additionally: multi-node variant to exercise `burnout_cleanup_node`

## New invariants to consider

- **`BurnoutHeatClamped`** (MEDIUM priority): catches `BurnoutHeat.heat` leaving `[0.0, 1.0]`. Clamping is unit-tested but scenarios are the only path against the running system. `NoNaN` only fires on IEEE NaN, not on out-of-range finite values.
- Similar clamping invariants may be needed per-protocol as their state expands.

## Priority

Batch this after one more protocol lands so we upgrade all scaffolds together rather than piecemeal. Start with the three that have the most runtime surface: Burnout, Afterimage (once it lands), Conductor (once it lands).

## Per-protocol scope estimate

- Write chaos scenario RON file: ~30 min each
- Add any new invariant checkers (BurnoutHeatClamped etc.): ~2h each
- Self-test scenarios for new invariants: ~30 min each
- Verification: run `cargo scenario -- --all` and triage

Total for the current 8 protocols: ~6-8 hours if no surprises, likely more if chaos surfaces regressions (which is the point).
