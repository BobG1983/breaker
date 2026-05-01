//! RON-asset structural shape for the Overcharge hazard.
//!
//! Pins that `overcharge.hazard.ron` parses into a `HazardDefinition` with
//! `HazardKind::Overcharge` and that numeric tuning fields are finite and
//! non-negative. Specific values are exercised by design-behaviour tests.

use crate::mutators::hazards::definition::{HazardDefinition, HazardKind, HazardTuning};

#[test]
fn overcharge_ron_asset_structural_shape() {
    let ron_str = include_str!("../../../../../assets/hazards/overcharge.hazard.ron");
    let def: HazardDefinition =
        ron::de::from_str(ron_str).expect("overcharge.hazard.ron should parse");

    assert_eq!(def.kind(), HazardKind::Overcharge);

    let HazardTuning::Overcharge {
        base_frac,
        per_level_frac,
    } = def.tuning
    else {
        panic!("expected HazardTuning::Overcharge");
    };
    assert!(
        base_frac.is_finite() && base_frac >= 0.0,
        "base_frac must be finite and non-negative, got {base_frac}"
    );
    assert!(
        per_level_frac.is_finite() && per_level_frac >= 0.0,
        "per_level_frac must be finite and non-negative, got {per_level_frac}"
    );
}

#[test]
fn overcharge_ron_name_description_unlock_tier_pinned_exactly() {
    let ron_str = include_str!("../../../../../assets/hazards/overcharge.hazard.ron");
    let def: HazardDefinition =
        ron::de::from_str(ron_str).expect("overcharge.hazard.ron should parse");

    assert_eq!(def.name, "Overcharge", "overcharge name drift guard");
    assert_eq!(
        def.description,
        "Bolt damage ramps up after each clear \u{2014} but so do the consequences of dropping a bolt.",
        "overcharge description drift guard"
    );
    assert_eq!(def.unlock_tier, 0, "overcharge unlock_tier drift guard");
}
