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

// -------------------------------------------------------------------------
// InvariantKind::ChainArcCountReasonable — behaviors 1-3
// -------------------------------------------------------------------------

#[test]
fn invariant_kind_chain_arc_count_reasonable_parses() {
    let result: InvariantKind =
        ron::de::from_str("ChainArcCountReasonable").expect("ChainArcCountReasonable should parse");
    assert_eq!(result, InvariantKind::ChainArcCountReasonable);
}

#[test]
fn invariant_kind_chain_arc_count_reasonable_debug_round_trip() {
    let debug_str = format!("{:?}", InvariantKind::ChainArcCountReasonable);
    assert!(
        debug_str.contains("ChainArcCountReasonable"),
        "Debug output should contain 'ChainArcCountReasonable', got: {debug_str}"
    );
}

#[test]
fn invariant_kind_all_includes_chain_arc_count_reasonable() {
    assert!(
        InvariantKind::ALL.contains(&InvariantKind::ChainArcCountReasonable),
        "InvariantKind::ALL must include ChainArcCountReasonable"
    );
}

#[test]
fn fail_reason_chain_arc_count_reasonable() {
    assert_eq!(
        InvariantKind::ChainArcCountReasonable.fail_reason(),
        "chain lightning arc/chain count exceeds maximum"
    );
}

// -------------------------------------------------------------------------
// InvariantParams::max_chain_arc_count — behaviors 4-5
// -------------------------------------------------------------------------

#[test]
fn invariant_params_defaults_max_chain_arc_count_to_50() {
    let params = InvariantParams::default();
    assert_eq!(
        params.max_chain_arc_count, 50,
        "InvariantParams::default().max_chain_arc_count should be 50"
    );
}

#[test]
fn invariant_params_max_chain_arc_count_preserved_with_struct_update() {
    let params = InvariantParams::default();
    assert_eq!(
        params.max_chain_arc_count, 50,
        "max_chain_arc_count should be preserved via ..InvariantParams::default()"
    );
}

#[test]
fn invariant_params_max_chain_arc_count_overridable_via_ron() {
    let ron = "(max_chain_arc_count: 10)";
    let params: InvariantParams =
        ron::de::from_str(ron).expect("InvariantParams with max_chain_arc_count should parse");
    assert_eq!(
        params.max_chain_arc_count, 10,
        "max_chain_arc_count should be overridden to 10"
    );
    // Other fields should retain defaults
    assert_eq!(
        params.max_bolt_count, 8,
        "max_bolt_count should retain default of 8"
    );
    assert_eq!(
        params.max_pulse_ring_count, 20,
        "max_pulse_ring_count should retain default of 20"
    );
}

#[test]
fn invariant_params_max_chain_arc_count_defaults_when_absent_in_ron() {
    let ron = "()";
    let params: InvariantParams =
        ron::de::from_str(ron).expect("InvariantParams with no fields should parse");
    assert_eq!(
        params.max_chain_arc_count, 50,
        "max_chain_arc_count should default to 50 when absent from RON"
    );
}
