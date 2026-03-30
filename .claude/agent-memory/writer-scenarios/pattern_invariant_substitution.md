---
name: Invariant substitution for unavailable variants
description: When a spec requests non-existent InvariantKind variants, how to substitute with existing ones
type: feedback
---

The spec may request invariants that don't exist in `InvariantKind`. Only use variants listed in `InvariantKind::ALL`.

As of 2026-03-29, the available invariants are:
`BoltInBounds`, `BoltSpeedInRange`, `BoltCountReasonable`, `BreakerInBounds`,
`NoEntityLeaks`, `NoNaN`, `TimerNonNegative`, `ValidStateTransitions`,
`ValidBreakerState`, `TimerMonotonicallyDecreasing`, `BreakerPositionClamped`,
`PhysicsFrozenDuringPause`, `OfferingNoDuplicates`, `MaxedChipNeverOffered`,
`ChipStacksConsistent`, `RunStatsMonotonic`, `ChipOfferExpected`,
`SecondWindWallAtMostOne`, `ShieldChargesConsistent`, `PulseRingAccumulation`,
`EffectiveSpeedConsistent`

**Why:** The spec was written before the invariants were implemented. Using a non-existent variant causes a RON parse error.

**How to apply:** When a spec lists an unavailable invariant:
- Any domain-specific invariant → find the closest structural invariant (entity leaks, NaN, bounds)

**Note:** `ShieldChargesConsistent`, `SecondWindWallAtMostOne`, `PulseRingAccumulation`, and
`EffectiveSpeedConsistent` ARE real `InvariantKind` variants as of 2026-03-29. The earlier
memory entry saying they needed substitution was stale — they were added to the enum.
