//! Group I — RON asset shape (Behaviors 58–60).
//!
//! Pins that `assets/hazards/momentum.hazard.ron` parses into a
//! `HazardDefinition` with the canonical Momentum tuning values and the exact
//! name, description, and unlock tier — drift guard.

use crate::hazard::definition::{HazardDefinition, HazardKind, HazardTuning};

// ── Behavior 58 — momentum.hazard.ron parses to HazardDefinition ────────────

#[test]
fn momentum_ron_asset_deserializes_to_hazard_definition() {
    let ron_str = include_str!("../../../../../assets/hazards/momentum.hazard.ron");
    let def: HazardDefinition =
        ron::de::from_str(ron_str).expect("momentum.hazard.ron should parse");

    assert_eq!(def.kind(), HazardKind::Momentum);
}

// ── Behavior 59 — tuning fields match canonical values ──────────────────────

#[test]
fn momentum_ron_tuning_fields_match_canonical_values() {
    let ron_str = include_str!("../../../../../assets/hazards/momentum.hazard.ron");
    let def: HazardDefinition =
        ron::de::from_str(ron_str).expect("momentum.hazard.ron should parse");

    let HazardTuning::Momentum {
        base_hp_per_hit,
        per_level_hp_per_hit,
    } = def.tuning.clone()
    else {
        panic!("expected HazardTuning::Momentum, got {:?}", def.tuning);
    };

    assert!(
        (base_hp_per_hit - 10.0).abs() < f32::EPSILON,
        "base_hp_per_hit expected 10.0, got {base_hp_per_hit}"
    );
    assert!(
        (per_level_hp_per_hit - 10.0).abs() < f32::EPSILON,
        "per_level_hp_per_hit expected 10.0, got {per_level_hp_per_hit}"
    );
}

// ── Behavior 60 — name / description / unlock_tier pinned exactly ───────────

#[test]
fn momentum_ron_name_description_unlock_tier_pinned_exactly() {
    let ron_str = include_str!("../../../../../assets/hazards/momentum.hazard.ron");
    let def: HazardDefinition =
        ron::de::from_str(ron_str).expect("momentum.hazard.ron should parse");

    assert_eq!(def.name, "Momentum", "name drift guard");
    assert_eq!(
        def.description,
        "Non-lethal damage bolsters cell HP. A second hit with double damage is required to break through.",
        "description drift guard"
    );
    assert_eq!(def.unlock_tier, 0, "unlock_tier drift guard");
}
