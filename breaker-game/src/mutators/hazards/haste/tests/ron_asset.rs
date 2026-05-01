//! RON-asset structural shape for the Haste hazard.
//!
//! Pins that `haste.hazard.ron` parses into a `HazardDefinition` with
//! `HazardKind::Haste`, that numeric tuning fields are finite and non-negative,
//! and that the description string matches the design-doc value. Specific
//! numeric values are exercised by design-behaviour tests.

use crate::mutators::hazards::definition::{HazardDefinition, HazardKind, HazardTuning};

#[test]
fn haste_ron_asset_deserializes_to_hazard_definition() {
    let ron_str = include_str!("../../../../../assets/hazards/haste.hazard.ron");
    let def: HazardDefinition = ron::de::from_str(ron_str).expect("haste.hazard.ron should parse");

    assert_eq!(def.kind(), HazardKind::Haste);
}

#[test]
fn haste_ron_tuning_carries_correct_percent_values() {
    let ron_str = include_str!("../../../../../assets/hazards/haste.hazard.ron");
    let def: HazardDefinition = ron::de::from_str(ron_str).expect("haste.hazard.ron should parse");

    let HazardTuning::Haste {
        base_percent,
        per_level_percent,
    } = def.tuning
    else {
        panic!("expected HazardTuning::Haste");
    };
    assert!(
        base_percent.is_finite() && base_percent >= 0.0,
        "base_percent must be finite and non-negative, got {base_percent}"
    );
    assert!(
        per_level_percent.is_finite() && per_level_percent >= 0.0,
        "per_level_percent must be finite and non-negative, got {per_level_percent}"
    );
}

#[test]
fn haste_ron_description_matches_design_doc_drift_guard() {
    let ron_str = include_str!("../../../../../assets/hazards/haste.hazard.ron");
    let def: HazardDefinition = ron::de::from_str(ron_str).expect("haste.hazard.ron should parse");
    assert_eq!(def.name, "Haste", "hazard name should be 'Haste'");
    assert_eq!(
        def.description,
        "Bolts move faster \u{2014} harder to time your bumps."
    );
    assert_eq!(def.unlock_tier, 0);
}
