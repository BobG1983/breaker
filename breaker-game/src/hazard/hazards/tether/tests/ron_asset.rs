//! Section G — RON asset shape (Behaviours 66–67).
//!
//! Pins that `assets/hazards/tether.hazard.ron` parses into a
//! `HazardDefinition` with the canonical Tether tuning values and exact name,
//! description, and unlock tier — guards against drift.

use crate::hazard::definition::{HazardDefinition, HazardKind, HazardTuning};

// ── Behavior 66 — tether.hazard.ron parses to HazardDefinition ───────────────

#[test]
fn tether_ron_asset_deserializes_to_hazard_definition() {
    let ron_str = include_str!("../../../../../assets/hazards/tether.hazard.ron");
    let def: HazardDefinition = ron::de::from_str(ron_str).expect("tether.hazard.ron should parse");

    assert_eq!(def.kind(), HazardKind::Tether);

    let HazardTuning::Tether {
        base_share_frac,
        per_level_share_frac,
        base_coverage_frac,
        per_level_coverage_frac,
    } = def.tuning.clone()
    else {
        panic!("expected HazardTuning::Tether, got {:?}", def.tuning);
    };
    assert!(
        (base_share_frac - 0.25).abs() < f32::EPSILON,
        "base_share_frac expected 0.25, got {base_share_frac}"
    );
    assert!(
        (per_level_share_frac - 0.10).abs() < f32::EPSILON,
        "per_level_share_frac expected 0.10, got {per_level_share_frac}"
    );
    assert!(
        (base_coverage_frac - 0.40).abs() < f32::EPSILON,
        "base_coverage_frac expected 0.40, got {base_coverage_frac}"
    );
    assert!(
        (per_level_coverage_frac - 0.10).abs() < f32::EPSILON,
        "per_level_coverage_frac expected 0.10, got {per_level_coverage_frac}"
    );
}

// ── Behavior 67 — name / description / unlock_tier pinned exactly ────────────

#[test]
fn tether_ron_name_description_unlock_tier_pinned_exactly() {
    let ron_str = include_str!("../../../../../assets/hazards/tether.hazard.ron");
    let def: HazardDefinition = ron::de::from_str(ron_str).expect("tether.hazard.ron should parse");

    assert_eq!(def.name, "Tether", "tether name drift guard");
    assert_eq!(
        def.description, "Pairs of cells share damage. Kill one, the other takes a slice.",
        "tether description drift guard"
    );
    assert_eq!(def.unlock_tier, 0, "tether unlock_tier drift guard");
}
