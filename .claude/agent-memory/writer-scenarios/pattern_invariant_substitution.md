---
name: Invariant substitution for unavailable variants
description: When a spec requests non-existent InvariantKind variants, how to substitute with existing ones
type: feedback
---

The spec may request invariants that don't exist in `InvariantKind`. Only use variants listed in `InvariantKind::ALL`.

As of 2026-04-02 (Shield refactor), the available invariants are (23 total):
`BoltInBounds`, `BoltSpeedAccurate`, `BoltCountReasonable`, `BreakerInBounds`,
`NoEntityLeaks`, `NoNaN`, `TimerNonNegative`, `ValidStateTransitions`,
`ValidBreakerState`, `TimerMonotonicallyDecreasing`, `BreakerPositionClamped`,
`PhysicsFrozenDuringPause`, `OfferingNoDuplicates`, `MaxedChipNeverOffered`,
`ChipStacksConsistent`, `RunStatsMonotonic`, `ChipOfferExpected`,
`SecondWindWallAtMostOne`, `ShieldWallAtMostOne`, `PulseRingAccumulation`,
`ChainArcCountReasonable`, `AabbMatchesEntityDimensions`, `GravityWellCountReasonable`

**REMOVED**: `BoltSpeedInRange` (renamed to `BoltSpeedAccurate`),
`EffectiveSpeedConsistent` (removed with Effective* cache removal),
`SizeBoostInRange` (removed with Effective* cache removal)

**RENAMED**: `ShieldChargesConsistent` → `ShieldWallAtMostOne` (Shield refactor, 2026-04-02;
checks `ShieldWall` entity count <= 1, not charge consistency — `ShieldActive` eliminated)

**Why:** The spec may be written before invariants are implemented. Using a non-existent variant causes a RON parse error.

**How to apply:** When a spec lists an unavailable invariant:
- Any domain-specific invariant → find the closest structural invariant (entity leaks, NaN, bounds)

Their self-test mutation kinds:
- `AabbMatchesEntityDimensions` → `InjectMismatchedBoltAabb` (no fields)
- `GravityWellCountReasonable` → `SpawnExtraGravityWells(N)` — also requires `invariant_params: (max_gravity_well_count: M)` when lowering threshold below default 10
- `BoltSpeedAccurate` → `InjectWrongBoltSpeed` (verify exact variant name in MutationKind)

**REMOVED mutations**: `InjectWrongSizeMultiplier` and `InjectWrongEffectiveSpeed` were removed with the Effective* cache removal. Do NOT reference these variants.

`InvariantParams` now has 4 fields: `max_bolt_count`, `max_pulse_ring_count`, `max_chain_arc_count`, `max_gravity_well_count` (defaults: 8, 20, 50, 10).
