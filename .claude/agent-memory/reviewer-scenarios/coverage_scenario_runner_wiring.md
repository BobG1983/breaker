---
name: Scenario Runner Wiring Branch Coverage Map
description: What was added on feature/scenario-runner-wiring — new scenarios for Prism/Aegis/Chrono baseline coverage, new effects (CircuitBreaker, MirrorProtocol, FlashStep, Anchor, SplitDecision, NovaLance), BreakerBuilder node-scale+boost scenarios, multi-node reuse mechanic test
type: project
---

## What Was Added (feature/scenario-runner-wiring)

### New Invariant Checkers Added
- `check_aabb_matches_entity_dimensions` (new checker file, self-test aabb_matches_entity_dimensions)
- `breaker_count_reasonable` (new checker file, self-test breaker_count_reasonable)
- `breaker_position_clamped` updated — now reads `ActiveSizeBoosts` and uses effective_half_width

### New Stress Scenarios (baseline breaker coverage)
- `aegis_chaos`, `aegis_bolt_stress`, `aegis_fortress`, `aegis_scatter`, `aegis_multinode` — Aegis baseline; only crash guards (BoltInBounds, BreakerCountReasonable, BreakerInBounds, NoEntityLeaks, NoNaN), no effect-correctness invariants
- `prism_stress`, `prism_accumulation`, `prism_bolt_stabilization`, `prism_concurrent_hits`, `prism_fortress`, `prism_scatter`, `prism_scatter_stress` — Prism multi-bolt baseline; most use BoltCountReasonable but NOT BoltSpeedAccurate
- `chrono_fortress`, `chrono_scatter` — Chrono baseline; only crash guards
- `dense_stress`, `dense_chaos`, `boss_arena_chaos`, `gauntlet_chaos` — layout-baseline scenarios

### New Effect Scenarios (previously MISSING from evolution ecosystem branch)
- `circuit_breaker_chaos` — uses chip_selections ["Circuit Breaker"], Perfect(AlwaysPerfect), Dense. Invariants: crash guards only. Does NOT include ChainArcCountReasonable or a counter-correctness invariant.
- `flashstep_chaos` — uses chip_selections ["FlashStep"], Chaos(0.9), Scatter. Invariants: BoltInBounds, BreakerCountReasonable, BreakerInBounds, NoNaN. Does NOT include ValidDashState or BoltSpeedAccurate.
- `mirror_protocol_chaos` — uses chip_selections ["Mirror Protocol"], stress(32), Scatter. Invariants: crash guards + BoltCountReasonable + BoltSpeedAccurate. Good.
- `nova_lance_chaos` — uses chip_selections ["Nova Lance"], Perfect(AlwaysPerfect), Dense. Invariants: crash guards + BoltSpeedAccurate. Does NOT include BoltCountReasonable.
- `split_decision_cascade` — uses chip_selections ["Split Decision"], Prism, Dense, stress(4). Invariants: crash guards + BoltCountReasonable. Correct — no BoltSpeedAccurate (Split Decision doesn't target speed).
- `anchor_plant_chaos` — uses chip_selections ["Anchor"], Chrono, Fortress, Chaos(0.7). Invariants: crash guards + BoltSpeedAccurate. Does NOT include BreakerPositionClamped or ValidDashState.

### New Builder/Scale Scenarios
- `node_scale_entity_chaos` — BossArena (entity_scale: 0.7) + SizeBoost(2.0) per bump on Breaker. Tests interaction. Invariants: BreakerPositionClamped, AabbMatchesEntityDimensions, NoNaN, BoltInBounds, BreakerInBounds, BreakerCountReasonable. Good.
- `bolt_radius_clamping_chaos` — BossArena + SizeBoost(3.0) per bump on Bolt. Invariants: AabbMatchesEntityDimensions, BoltInBounds, BreakerCountReasonable, BreakerInBounds, NoNaN. Good.

### New Mechanic Scenarios
- `multi_node_breaker_reuse` — quick_clear layout, godmode breaker, allow_early_end: true, 5000 frames. Tests spawn_or_reuse_breaker reuse path across node transitions. Invariants: BreakerCountReasonable, NoNaN, BreakerInBounds, BreakerPositionClamped, BoltInBounds. This RESOLVES the spawn_or_reuse gap from coverage_breaker_builder_pattern.md.

### New Self-Test Scenarios
- `aabb_matches_entity_dimensions` — InjectMismatchedBoltAabb at frame 30. Correct.
- `breaker_count_reasonable` — SpawnExtraPrimaryBreakers(1) at frame 5. Correct.

## Remaining Gaps After This Branch

### Still MISSING (from prior memory maps, not resolved by this branch)
- TetherBeam chain mode (Arcwelder) — no scenario, per coverage_effect_system.md
- LoseLife via initial_effects — no scenario
- NoBump, Died, Impact(Bolt), Impacted(Bolt on breaker), Impacted(Breaker) triggers — no scenarios
- RampingDamage monotonicity invariant — still no invariant
- BreakerPositionClamped self-test scenario — no scenario in self_tests/ intentionally violates this invariant
- Scale2D coherence invariant — no invariant checks Scale2D matches effective size (visual-only, LOW priority)
- Evolution chips via chip_selections (not initial_effects): Second Wind, Entropy Engine, Gravity Well, Phantom Bolt, Voltchain, Resonance Cascade, Shock Chain — all use initial_effects
- Evolution offering path: multiple eligible evolutions — no scenario

### New Gaps Introduced by This Branch
- `flashstep_chaos` missing ValidDashState — FlashStep should exercise settling→teleport state machine transition; ValidDashState would catch illegal state transitions from the teleport
- `circuit_breaker_chaos` missing BoltCountReasonable — CircuitBreaker spawns bolts on counter completion; the bolt count should be bounded
- `nova_lance_chaos` missing BoltCountReasonable — PiercingBeam is spawning beams per hit; BoltCountReasonable should be in the invariant list
- Anchor missing BreakerPositionClamped and ValidDashState — plant/unplant cycle interacts with the breaker state machine and position clamping
- Prism scenarios (prism_stress, prism_fortress, prism_scatter) missing BoltSpeedAccurate — Prism spawns extra bolts but many scenarios don't verify speed accuracy on any of them
- BreakerPositionClamped has no self-test scenario in self_tests/ — the checker exists and was updated to track ActiveSizeBoosts, but there is no self-test that intentionally triggers it

## How to apply
- When reviewing Prism scenarios, flag the ones that omit BoltSpeedAccurate as adversarial quality issues.
- BreakerPositionClamped self-test gap is HIGH — it's a new checker (with boost logic) with no self-test.
- FlashStep missing ValidDashState is HIGH — the whole mechanic is about dash state transitions.
