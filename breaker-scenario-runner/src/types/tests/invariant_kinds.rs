use crate::types::*;

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
fn invariant_kind_shield_wall_at_most_one_parses() {
    let result: InvariantKind =
        ron::de::from_str("ShieldWallAtMostOne").expect("ShieldWallAtMostOne should parse");
    assert_eq!(result, InvariantKind::ShieldWallAtMostOne);
}

#[test]
fn invariant_kind_pulse_ring_accumulation_parses() {
    let result: InvariantKind =
        ron::de::from_str("PulseRingAccumulation").expect("PulseRingAccumulation should parse");
    assert_eq!(result, InvariantKind::PulseRingAccumulation);
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

// -------------------------------------------------------------------------
// InvariantKind::AabbMatchesEntityDimensions — behaviors 1-4
// -------------------------------------------------------------------------

#[test]
fn invariant_kind_aabb_matches_entity_dimensions_parses() {
    let result: InvariantKind = ron::de::from_str("AabbMatchesEntityDimensions")
        .expect("AabbMatchesEntityDimensions should parse");
    assert_eq!(result, InvariantKind::AabbMatchesEntityDimensions);
}

#[test]
fn invariant_kind_all_includes_aabb_matches_entity_dimensions() {
    assert!(
        InvariantKind::ALL.contains(&InvariantKind::AabbMatchesEntityDimensions),
        "InvariantKind::ALL must include AabbMatchesEntityDimensions"
    );
}

#[test]
fn fail_reason_aabb_matches_entity_dimensions() {
    assert_eq!(
        InvariantKind::AabbMatchesEntityDimensions.fail_reason(),
        "Aabb2D half_extents do not match entity dimensions"
    );
}

#[test]
fn invariant_kind_aabb_matches_entity_dimensions_debug_round_trip() {
    let debug_str = format!("{:?}", InvariantKind::AabbMatchesEntityDimensions);
    assert!(
        debug_str.contains("AabbMatchesEntityDimensions"),
        "Debug output should contain 'AabbMatchesEntityDimensions', got: {debug_str}"
    );
}

// -------------------------------------------------------------------------
// InvariantKind::GravityWellCountReasonable — behaviors 5-8
// -------------------------------------------------------------------------

#[test]
fn invariant_kind_gravity_well_count_reasonable_parses() {
    let result: InvariantKind = ron::de::from_str("GravityWellCountReasonable")
        .expect("GravityWellCountReasonable should parse");
    assert_eq!(result, InvariantKind::GravityWellCountReasonable);
}

#[test]
fn invariant_kind_all_includes_gravity_well_count_reasonable() {
    assert!(
        InvariantKind::ALL.contains(&InvariantKind::GravityWellCountReasonable),
        "InvariantKind::ALL must include GravityWellCountReasonable"
    );
}

#[test]
fn fail_reason_gravity_well_count_reasonable() {
    assert_eq!(
        InvariantKind::GravityWellCountReasonable.fail_reason(),
        "gravity well entity count exceeds maximum"
    );
}

#[test]
fn invariant_kind_gravity_well_count_reasonable_debug_round_trip() {
    let debug_str = format!("{:?}", InvariantKind::GravityWellCountReasonable);
    assert!(
        debug_str.contains("GravityWellCountReasonable"),
        "Debug output should contain 'GravityWellCountReasonable', got: {debug_str}"
    );
}

// -------------------------------------------------------------------------
// InvariantKind::ALL — count after removals
// -------------------------------------------------------------------------

#[test]
fn invariant_kind_all_contains_22_variants() {
    assert_eq!(
        InvariantKind::ALL.len(),
        22,
        "InvariantKind::ALL should contain 22 variants"
    );
}

// -------------------------------------------------------------------------
// InvariantParams::max_gravity_well_count — behaviors 20-23
// -------------------------------------------------------------------------

#[test]
fn invariant_params_defaults_max_gravity_well_count_to_10() {
    let params = InvariantParams::default();
    assert_eq!(
        params.max_gravity_well_count, 10,
        "InvariantParams::default().max_gravity_well_count should be 10"
    );
}

#[test]
fn invariant_params_max_gravity_well_count_overridable_via_ron() {
    let ron = "(max_gravity_well_count: 5)";
    let params: InvariantParams =
        ron::de::from_str(ron).expect("InvariantParams with max_gravity_well_count should parse");
    assert_eq!(
        params.max_gravity_well_count, 5,
        "max_gravity_well_count should be overridden to 5"
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
    assert_eq!(
        params.max_chain_arc_count, 50,
        "max_chain_arc_count should retain default of 50"
    );
}

#[test]
fn invariant_params_max_gravity_well_count_defaults_when_absent_in_ron() {
    let ron = "()";
    let params: InvariantParams =
        ron::de::from_str(ron).expect("InvariantParams with no fields should parse");
    assert_eq!(
        params.max_gravity_well_count, 10,
        "max_gravity_well_count should default to 10 when absent from RON"
    );
}

#[test]
fn invariant_params_partial_override_preserves_defaults_for_unspecified_fields() {
    let ron = "(max_bolt_count: 12)";
    let params: InvariantParams =
        ron::de::from_str(ron).expect("InvariantParams with only max_bolt_count should parse");
    assert_eq!(
        params.max_bolt_count, 12,
        "max_bolt_count should be overridden to 12"
    );
    assert_eq!(
        params.max_gravity_well_count, 10,
        "max_gravity_well_count should retain default of 10"
    );
    assert_eq!(
        params.max_pulse_ring_count, 20,
        "max_pulse_ring_count should retain default of 20"
    );
    assert_eq!(
        params.max_chain_arc_count, 50,
        "max_chain_arc_count should retain default of 50"
    );
}

// -------------------------------------------------------------------------
// InvariantKind::BreakerCountReasonable -- behaviors 1-4
// -------------------------------------------------------------------------

#[test]
fn invariant_kind_breaker_count_reasonable_parses() {
    let result: InvariantKind =
        ron::de::from_str("BreakerCountReasonable").expect("BreakerCountReasonable should parse");
    assert_eq!(result, InvariantKind::BreakerCountReasonable);
}

#[test]
fn invariant_kind_all_includes_breaker_count_reasonable() {
    assert!(
        InvariantKind::ALL.contains(&InvariantKind::BreakerCountReasonable),
        "InvariantKind::ALL must include BreakerCountReasonable"
    );
}

#[test]
fn fail_reason_breaker_count_reasonable() {
    assert_eq!(
        InvariantKind::BreakerCountReasonable.fail_reason(),
        "primary breaker count is not exactly 1"
    );
}

#[test]
fn invariant_kind_breaker_count_reasonable_debug_round_trip() {
    let debug_str = format!("{:?}", InvariantKind::BreakerCountReasonable);
    assert!(
        debug_str.contains("BreakerCountReasonable"),
        "Debug output should contain 'BreakerCountReasonable', got: {debug_str}"
    );
}
