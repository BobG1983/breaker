//! Section E — RON asset shape.

use crate::hazard::definition::{HazardDefinition, HazardKind, HazardTuning};

// Behavior 47 — diffusion.hazard.ron parses to a HazardDefinition.
#[test]
fn diffusion_ron_asset_deserializes_to_hazard_definition() {
    let ron_str = include_str!("../../../../../assets/hazards/diffusion.hazard.ron");
    let def: HazardDefinition =
        ron::de::from_str(ron_str).expect("diffusion.hazard.ron should parse");

    assert_eq!(def.kind(), HazardKind::Diffusion);

    // Field accessors live inside the definition module; match the tuning
    // variant to extract fractional values and compare.
    let HazardTuning::Diffusion {
        base_share_frac,
        per_level_share_frac,
        depth_every_levels,
    } = match_definition_tuning_for_test(&def)
    else {
        panic!("expected HazardTuning::Diffusion");
    };
    assert!((base_share_frac - 0.2).abs() < f32::EPSILON);
    assert!((per_level_share_frac - 0.1).abs() < f32::EPSILON);
    assert_eq!(depth_every_levels, 5);
}

// Behavior 48 — description matches the design-doc drift guard.
#[test]
fn diffusion_ron_description_matches_design_doc_drift_guard() {
    let ron_str = include_str!("../../../../../assets/hazards/diffusion.hazard.ron");
    let def: HazardDefinition =
        ron::de::from_str(ron_str).expect("diffusion.hazard.ron should parse");
    assert_eq!(
        name_of(&def),
        "Diffusion",
        "hazard name should be 'Diffusion'"
    );
    assert_eq!(
        description_of(&def),
        "Damage spreads to neighboring cells, but each hit leaves the target less damaged."
    );
    assert_eq!(unlock_tier_of(&def), 0);
}

// `HazardDefinition`'s `name`/`description`/`tuning`/`unlock_tier` are
// `pub(crate)` — reachable from this test module because we are inside the
// same crate. Access them through thin helpers kept here so the test
// bodies read as data-oriented assertions.
fn name_of(def: &HazardDefinition) -> &str {
    &def.name
}

fn description_of(def: &HazardDefinition) -> &str {
    &def.description
}

fn unlock_tier_of(def: &HazardDefinition) -> u32 {
    def.unlock_tier
}

fn match_definition_tuning_for_test(def: &HazardDefinition) -> HazardTuning {
    def.tuning.clone()
}
