//! RON-asset structural shape for the Cascade hazard.
//!
//! Pins that `cascade.hazard.ron` parses into a `HazardDefinition` with
//! `HazardKind::Cascade` and that numeric tuning fields are finite and
//! non-negative. Specific values are exercised by design-behaviour tests.

use crate::mutators::hazards::definition::{HazardDefinition, HazardKind, HazardTuning};

#[test]
fn cascade_ron_asset_deserializes_with_correct_tuning() {
    let ron_str = include_str!("../../../../../assets/hazards/cascade.hazard.ron");
    let def: HazardDefinition =
        ron::de::from_str(ron_str).expect("cascade.hazard.ron should parse");

    assert_eq!(def.kind(), HazardKind::Cascade);

    let HazardTuning::Cascade {
        base_heal,
        per_level_heal,
    } = def.tuning
    else {
        panic!("expected HazardTuning::Cascade");
    };
    assert!(
        base_heal.is_finite() && base_heal >= 0.0,
        "base_heal must be finite and non-negative, got {base_heal}"
    );
    assert!(
        per_level_heal.is_finite() && per_level_heal >= 0.0,
        "per_level_heal must be finite and non-negative, got {per_level_heal}"
    );
}

#[test]
fn cascade_ron_name_description_unlock_tier_pinned_exactly() {
    let ron_str = include_str!("../../../../../assets/hazards/cascade.hazard.ron");
    let def: HazardDefinition =
        ron::de::from_str(ron_str).expect("cascade.hazard.ron should parse");

    assert_eq!(def.name, "Cascade", "cascade name drift guard");
    assert_eq!(
        def.description, "Every cell clear heals the remaining cells a little.",
        "cascade description drift guard"
    );
    assert_eq!(def.unlock_tier, 0, "cascade unlock_tier drift guard");
}
