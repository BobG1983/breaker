---
name: Invariant substitution for unavailable variants
description: When a spec requests non-existent InvariantKind variants, how to substitute with existing ones
type: feedback
---

The spec may request invariants that don't exist in `InvariantKind`. Only use variants listed in `InvariantKind::ALL`.

As of 2026-03-30, the available invariants are (25 total):
`BoltInBounds`, `BoltSpeedInRange`, `BoltCountReasonable`, `BreakerInBounds`,
`NoEntityLeaks`, `NoNaN`, `TimerNonNegative`, `ValidStateTransitions`,
`ValidBreakerState`, `TimerMonotonicallyDecreasing`, `BreakerPositionClamped`,
`PhysicsFrozenDuringPause`, `OfferingNoDuplicates`, `MaxedChipNeverOffered`,
`ChipStacksConsistent`, `RunStatsMonotonic`, `ChipOfferExpected`,
`SecondWindWallAtMostOne`, `ShieldChargesConsistent`, `PulseRingAccumulation`,
`EffectiveSpeedConsistent`, `ChainArcCountReasonable`,
`AabbMatchesEntityDimensions`, `GravityWellCountReasonable`, `SizeBoostInRange`

**Why:** The spec may be written before invariants are implemented. Using a non-existent variant causes a RON parse error.

**How to apply:** When a spec lists an unavailable invariant:
- Any domain-specific invariant → find the closest structural invariant (entity leaks, NaN, bounds)

Their self-test mutation kinds:
- `AabbMatchesEntityDimensions` → `InjectMismatchedBoltAabb` (no fields)
- `GravityWellCountReasonable` → `SpawnExtraGravityWells(N)` — also requires `invariant_params: (max_gravity_well_count: M)` when lowering threshold below default 10
- `SizeBoostInRange` → `InjectWrongSizeMultiplier(wrong_value: 99.0)` (named field)

`InvariantParams` now has 4 fields: `max_bolt_count`, `max_pulse_ring_count`, `max_chain_arc_count`, `max_gravity_well_count` (defaults: 8, 20, 50, 10).
