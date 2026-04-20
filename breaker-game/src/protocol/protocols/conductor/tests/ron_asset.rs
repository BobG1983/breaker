//! RON asset drift guard — pins `assets/protocols/conductor.protocol.ron`
//! against the canonical `ProtocolDefinition` values.

use crate::protocol::definition::{ProtocolDefinition, ProtocolKind, ProtocolTuning};

// ── Behavior 27 — RON parses with exact tuning variant + pinned value ───────

#[test]
fn conductor_ron_asset_deserializes_to_protocol_definition() {
    let ron_str = include_str!("../../../../../assets/protocols/conductor.protocol.ron");
    let def: ProtocolDefinition =
        ron::de::from_str(ron_str).expect("conductor.protocol.ron should parse");

    assert_eq!(
        def.kind(),
        ProtocolKind::Conductor,
        "parsed definition must report ProtocolKind::Conductor"
    );

    let ProtocolTuning::Conductor {
        primary_swap_window,
    } = &def.tuning
    else {
        panic!("expected ProtocolTuning::Conductor, got {:?}", def.tuning);
    };

    assert!(
        (primary_swap_window - 0.2).abs() < f32::EPSILON,
        "primary_swap_window expected 0.2 (RON file literal), got {primary_swap_window}"
    );
}

// ── Behavior 28 — name / description / unlock_tier pinned exactly ───────────

#[test]
fn conductor_ron_name_description_unlock_tier_pinned_exactly() {
    let ron_str = include_str!("../../../../../assets/protocols/conductor.protocol.ron");
    let def: ProtocolDefinition =
        ron::de::from_str(ron_str).expect("conductor.protocol.ron should parse");

    assert_eq!(def.name, "Conductor", "conductor name drift guard");
    assert_eq!(
        def.description, "Perfect-bump an extra bolt to inherit the primary bolt's chip effects.",
        "conductor description drift guard"
    );
    assert_eq!(def.unlock_tier, 0, "conductor unlock_tier drift guard");
}
