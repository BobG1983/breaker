# Phantom Bolt — Scenario Coverage Gaps (post-merge follow-up)

## Summary
Two scenario-coverage gaps identified by reviewer-scenarios during the phantom-bolt Full Verification Tier (2026-05-27). Deferred to post-merge (user-approved) because the mutate-real-bolt chain is fully exercised end-to-end by unit tests (T18-T21) and all 176 existing scenarios pass clean under chaos.

## Context
Phantom Bolt feature (todo #1) merged on `feature/phantom-bolt`. The Wave 4A mutate-real-bolt path (`afterimage_spawn_phantom_bolt` calling `Bolt::become_phantom`) is well-covered by isolated unit tests but has no end-to-end physics-enabled chaos scenario exercising the full chain (trigger → mutation → lifespan tick → revert).

Additionally, the `breaker-scenario-runner` invariant catalog has no checker for the failure mode "PhantomBolt component present but Lifespan absent or expired" — a regression in `tick_bolt_lifespan` dispatch could leave a bolt permanently phantom with no scenario catching it.

## Scope

**In:**

1. **Chaos scenario**: `scenarios/stress/afterimage_phantom_bolt_chaos.scenario.ron`
   - `breaker: "Aegis"`, `layout: "Dense"`, `input: Chaos((action_prob: 0.9))`, `max_frames: 4000`
   - `frame_mutations: [(frame: 10, mutation: InjectProtocol(kind_name: "Afterimage"))]`
   - `disallowed_failures: [BoltCountReasonable, BoltInBounds, BreakerInBounds, NoNaN, NoEntityLeaks, BoltSpeedAccurate, ValidDashState, BreakerCountReasonable, ExactlyOnePrimaryBolt, PhantomBoltOrphaned]`
   - `stress: (runs: 16, parallelism: 16)` to maximize Perfect-bump-on-phantom-breaker overlap

2. **`PhantomBoltOrphaned` invariant**:
   - Add `InvariantKind::PhantomBoltOrphaned` variant to `breaker-scenario-runner`
   - Check: for every entity with `(With<Bolt>, With<PhantomBolt>)`, if `Lifespan` is absent OR `Lifespan.remaining <= 0.0`, fire violation
   - Self-test scenario: `scenarios/self_tests/phantom_bolt_orphaned_self_test.scenario.ron` — inject `(PhantomBolt, PhantomDedupKey, PhantomDamagedCells, Lifespan { remaining: 0.0 })` and assert the invariant fires on the next frame before `tick_bolt_lifespan` has a chance to revert

**Out:**
- Tuning sweep on `phantom_bolt_duration` (3.0 → 1.5–2.0s) — Phase 5 polish item, separate concern
- PhantomFlicker visual quality (min_alpha or color-shift effect) — Phase 5 polish item, separate concern
- Upgrading `mechanic/afterimage_smoke.scenario.ron` from `disable_physics: true` to physics-enabled — could be folded in once the chaos scenario lands
- `max_active` boundary scenario for chip-effect SpawnPhantom — LOW priority per reviewer
- Cell double-damage dedup chaos scenario with damage-amount verification — LOW priority per reviewer
- `PhantomBumpFiresNoChipEffect` invariant — inherited gap from phantom-breaker, not in scope here
- `PhantomBreakerOrphaned` invariant — inherited gap from phantom-breaker, not in scope here

## Dependencies
- Depends on: nothing (phantom-bolt merged)
- Blocks: nothing — pure coverage tightening

## Notes
- The chaos scenario should be added FIRST (lower-risk; just a RON file)
- The invariant requires writer-tests → writer-code TDD cycle for `breaker-scenario-runner` crate; treat as a small standalone TDD feature
- Self-test scenario validates the invariant catches its target failure mode (per `.claude/rules/spec-format-tests.md` Scenario Coverage rules)

## Status
`ready`
