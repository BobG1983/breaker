//! Group G — RON asset shape (Behaviors 61–63).
//!
//! Pins that `assets/hazards/sympathy.hazard.ron` parses into a
//! `HazardDefinition` with the canonical Sympathy tuning and the exact
//! name, description, and unlock tier — drift guard.

use crate::mutators::hazards::definition::{HazardDefinition, HazardKind, HazardTuning};

// ── Behavior 61 — sympathy.hazard.ron parses to HazardDefinition ────────────

#[test]
fn sympathy_ron_asset_deserializes_to_hazard_definition() {
    let ron_str = include_str!("../../../../../assets/hazards/sympathy.hazard.ron");
    let def: HazardDefinition =
        ron::de::from_str(ron_str).expect("sympathy.hazard.ron should parse");

    assert_eq!(def.kind(), HazardKind::Sympathy);
}

// ── Behavior 62 — tuning fields match canonical values ──────────────────────

#[test]
fn sympathy_ron_tuning_fields_match_canonical_values() {
    let ron_str = include_str!("../../../../../assets/hazards/sympathy.hazard.ron");
    let def: HazardDefinition =
        ron::de::from_str(ron_str).expect("sympathy.hazard.ron should parse");

    let HazardTuning::Sympathy {
        base_heal_frac,
        per_level_heal_frac,
        depth_every_levels,
    } = def.tuning.clone()
    else {
        panic!("expected HazardTuning::Sympathy, got {:?}", def.tuning);
    };

    assert!(
        (base_heal_frac - 0.25).abs() < f32::EPSILON,
        "base_heal_frac expected 0.25, got {base_heal_frac}"
    );
    assert!(
        (per_level_heal_frac - 0.05).abs() < f32::EPSILON,
        "per_level_heal_frac expected 0.05, got {per_level_heal_frac}"
    );
    assert_eq!(depth_every_levels, 5);
}

// ── Behavior 63 — name / description / unlock_tier pinned exactly ───────────

#[test]
fn sympathy_ron_name_description_unlock_tier_pinned_exactly() {
    let ron_str = include_str!("../../../../../assets/hazards/sympathy.hazard.ron");
    let def: HazardDefinition =
        ron::de::from_str(ron_str).expect("sympathy.hazard.ron should parse");

    assert_eq!(def.name, "Sympathy", "name drift guard");
    assert_eq!(
        def.description, "When a cell takes damage, its neighbors heal.",
        "description drift guard"
    );
    assert_eq!(def.unlock_tier, 0, "unlock_tier drift guard");
}
