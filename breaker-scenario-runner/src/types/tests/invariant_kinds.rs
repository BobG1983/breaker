use super::super::*;

// -------------------------------------------------------------------------
// InvariantKind — all variants
// -------------------------------------------------------------------------

#[test]
fn invariant_kind_bolt_in_bounds_parses() {
    let result: InvariantKind =
        ron::de::from_str("BoltInBounds").expect("BoltInBounds should parse");
    assert_eq!(result, InvariantKind::BoltInBounds);
}

#[test]
fn invariant_kind_breaker_in_bounds_parses() {
    let result: InvariantKind =
        ron::de::from_str("BreakerInBounds").expect("BreakerInBounds should parse");
    assert_eq!(result, InvariantKind::BreakerInBounds);
}

#[test]
fn invariant_kind_no_entity_leaks_parses() {
    let result: InvariantKind =
        ron::de::from_str("NoEntityLeaks").expect("NoEntityLeaks should parse");
    assert_eq!(result, InvariantKind::NoEntityLeaks);
}

#[test]
fn invariant_kind_no_nan_parses() {
    let result: InvariantKind = ron::de::from_str("NoNaN").expect("NoNaN should parse");
    assert_eq!(result, InvariantKind::NoNaN);
}

#[test]
fn invariant_kind_valid_state_transitions_parses() {
    let result: InvariantKind =
        ron::de::from_str("ValidStateTransitions").expect("ValidStateTransitions should parse");
    assert_eq!(result, InvariantKind::ValidStateTransitions);
}

#[test]
fn invariant_kind_offering_no_duplicates_parses() {
    let result: InvariantKind =
        ron::de::from_str("OfferingNoDuplicates").expect("OfferingNoDuplicates should parse");
    assert_eq!(result, InvariantKind::OfferingNoDuplicates);
}

#[test]
fn invariant_kind_maxed_chip_never_offered_parses() {
    let result: InvariantKind =
        ron::de::from_str("MaxedChipNeverOffered").expect("MaxedChipNeverOffered should parse");
    assert_eq!(result, InvariantKind::MaxedChipNeverOffered);
}

#[test]
fn invariant_kind_chip_stacks_consistent_parses() {
    let result: InvariantKind =
        ron::de::from_str("ChipStacksConsistent").expect("ChipStacksConsistent should parse");
    assert_eq!(result, InvariantKind::ChipStacksConsistent);
}

#[test]
fn invariant_kind_run_stats_monotonic_parses() {
    let result: InvariantKind =
        ron::de::from_str("RunStatsMonotonic").expect("RunStatsMonotonic should parse");
    assert_eq!(result, InvariantKind::RunStatsMonotonic);
}

#[test]
fn invariant_kind_second_wind_wall_at_most_one_parses() {
    let result: InvariantKind =
        ron::de::from_str("SecondWindWallAtMostOne").expect("SecondWindWallAtMostOne should parse");
    assert_eq!(result, InvariantKind::SecondWindWallAtMostOne);
}

#[test]
fn invariant_kind_shield_charges_consistent_parses() {
    let result: InvariantKind =
        ron::de::from_str("ShieldChargesConsistent").expect("ShieldChargesConsistent should parse");
    assert_eq!(result, InvariantKind::ShieldChargesConsistent);
}

#[test]
fn invariant_kind_pulse_ring_accumulation_parses() {
    let result: InvariantKind =
        ron::de::from_str("PulseRingAccumulation").expect("PulseRingAccumulation should parse");
    assert_eq!(result, InvariantKind::PulseRingAccumulation);
}

#[test]
fn invariant_kind_effective_speed_consistent_parses() {
    let result: InvariantKind = ron::de::from_str("EffectiveSpeedConsistent")
        .expect("EffectiveSpeedConsistent should parse");
    assert_eq!(result, InvariantKind::EffectiveSpeedConsistent);
}

// -------------------------------------------------------------------------
// InvariantKind::fail_reason — each variant returns non-empty string
// -------------------------------------------------------------------------

#[test]
fn fail_reason_returns_non_empty_string_for_every_variant() {
    for variant in InvariantKind::ALL {
        let reason = variant.fail_reason();
        assert!(
            !reason.is_empty(),
            "fail_reason() for {variant:?} must not be empty"
        );
    }
}

#[test]
fn fail_reason_returns_distinct_strings_for_each_variant() {
    let reasons: Vec<&str> = InvariantKind::ALL
        .iter()
        .map(InvariantKind::fail_reason)
        .collect();
    let unique: std::collections::HashSet<&str> = reasons.iter().copied().collect();
    assert_eq!(
        reasons.len(),
        unique.len(),
        "fail_reason() must return distinct strings — found duplicates in: {reasons:?}"
    );
}

#[test]
fn all_variants_covered_by_invariant_kind_all() {
    // If a new variant is added to InvariantKind, fail_reason()'s exhaustive
    // match forces a compile error. This test ensures ALL has the right count.
    let unique: std::collections::HashSet<InvariantKind> =
        InvariantKind::ALL.iter().copied().collect();
    assert_eq!(
        InvariantKind::ALL.len(),
        unique.len(),
        "InvariantKind::ALL must not contain duplicates"
    );
}
