//! RON asset drift guard — pins `assets/protocols/burnout.protocol.ron`
//! against the canonical `ProtocolDefinition` values.

use crate::mutators::protocols::definition::{ProtocolDefinition, ProtocolKind, ProtocolTuning};

// ── J1 — RON parses with exact tuning variant + design-doc values ──────────-

#[test]
fn burnout_ron_asset_deserializes_to_protocol_definition() {
    let ron_str = include_str!("../../../../../assets/protocols/burnout.protocol.ron");
    let def: ProtocolDefinition =
        ron::de::from_str(ron_str).expect("burnout.protocol.ron should parse");

    assert_eq!(
        def.kind(),
        ProtocolKind::Burnout,
        "parsed definition must report ProtocolKind::Burnout"
    );

    let ProtocolTuning::Burnout {
        fill_duration,
        drain_duration,
        still_threshold,
        full_heat_damage_multiplier,
        speed_boost_duration,
    } = &def.tuning
    else {
        panic!("expected ProtocolTuning::Burnout, got {:?}", def.tuning);
    };

    assert!(
        (fill_duration - 4.0).abs() < f32::EPSILON,
        "fill_duration expected 4.0 (design-doc source of truth), got {fill_duration}"
    );
    assert!(
        (drain_duration - 2.0).abs() < f32::EPSILON,
        "drain_duration expected 2.0, got {drain_duration}"
    );
    assert!(
        (still_threshold - 1.5).abs() < f32::EPSILON,
        "still_threshold expected 1.5, got {still_threshold}"
    );
    assert!(
        (full_heat_damage_multiplier - 4.0).abs() < f32::EPSILON,
        "full_heat_damage_multiplier expected 4.0, got {full_heat_damage_multiplier}"
    );
    assert!(
        (speed_boost_duration - 2.0).abs() < f32::EPSILON,
        "speed_boost_duration expected 2.0, got {speed_boost_duration}"
    );
}

// ── J2 — name / description / unlock_tier pinned exactly ───────────────────-

#[test]
fn burnout_ron_name_description_unlock_tier_pinned_exactly() {
    let ron_str = include_str!("../../../../../assets/protocols/burnout.protocol.ron");
    let def: ProtocolDefinition =
        ron::de::from_str(ron_str).expect("burnout.protocol.ron should parse");

    assert_eq!(def.name, "Burnout", "burnout name drift guard");
    assert_eq!(
        def.description,
        "Move to build heat, stop to drain it into a bump-window speed and damage surge.",
        "burnout description drift guard"
    );
    assert_eq!(def.unlock_tier, 0, "burnout unlock_tier drift guard");
}
