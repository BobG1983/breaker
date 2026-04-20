//! Group E — RON asset drift guard (Behaviors 27–28).
//!
//! Pins that `assets/protocols/iron_curtain.protocol.ron` parses into a
//! `ProtocolDefinition` with `ProtocolKind::IronCurtain`, authored tuning
//! `damage_fraction: 0.25, falloff_start: 0.5`, name `"Iron Curtain"`, the
//! authored description, and `unlock_tier: 0`.
//!
//! NOTE: the RON's `0.25` / `0.5` intentionally differs from the canonical
//! worked-example values (`0.5` / `50.0`) used by the system-behavior tests.

use crate::protocol::definition::{ProtocolDefinition, ProtocolKind, ProtocolTuning};

// ── Behavior 27 — RON parses with exact tuning variant + values ────────────-

#[test]
fn iron_curtain_ron_asset_deserializes_to_protocol_definition() {
    let ron_str = include_str!("../../../../../assets/protocols/iron_curtain.protocol.ron");
    let def: ProtocolDefinition =
        ron::de::from_str(ron_str).expect("iron_curtain.protocol.ron should parse");

    assert_eq!(
        def.kind(),
        ProtocolKind::IronCurtain,
        "parsed definition must report ProtocolKind::IronCurtain"
    );

    let ProtocolTuning::IronCurtain {
        damage_fraction,
        falloff_start,
    } = def.tuning.clone()
    else {
        panic!("expected ProtocolTuning::IronCurtain, got {:?}", def.tuning);
    };

    assert!(
        (damage_fraction - 0.25).abs() < f32::EPSILON,
        "damage_fraction expected 0.25 (RON file literal), got {damage_fraction}"
    );
    assert!(
        (falloff_start - 0.5).abs() < f32::EPSILON,
        "falloff_start expected 0.5 (RON file literal), got {falloff_start}"
    );
}

// ── Behavior 28 — name / description / unlock_tier pinned exactly ──────────-

#[test]
fn iron_curtain_ron_name_description_unlock_tier_pinned_exactly() {
    let ron_str = include_str!("../../../../../assets/protocols/iron_curtain.protocol.ron");
    let def: ProtocolDefinition =
        ron::de::from_str(ron_str).expect("iron_curtain.protocol.ron should parse");

    assert_eq!(def.name, "Iron Curtain", "iron_curtain name drift guard");
    assert_eq!(
        def.description, "Cells farther from the breaker take fractional damage.",
        "iron_curtain description drift guard"
    );
    assert_eq!(def.unlock_tier, 0, "iron_curtain unlock_tier drift guard");
}
