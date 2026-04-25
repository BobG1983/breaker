//! RON asset drift guard — pins `assets/protocols/conductor.protocol.ron`
//! against the canonical `ProtocolDefinition` values.

use crate::mutators::protocols::definition::{ProtocolDefinition, ProtocolKind, ProtocolTuning};

// ── Behavior 27 — RON parses to the exact tuning variant ────────────────────

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

    assert!(
        matches!(def.tuning, ProtocolTuning::Conductor),
        "expected ProtocolTuning::Conductor, got {:?}",
        def.tuning
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
        def.description,
        "Perfect-bump an extra bolt to swap primary with it — the bumped bolt becomes the new \
         primary and inherits the old primary's chip effects.",
        "conductor description drift guard"
    );
    assert_eq!(def.unlock_tier, 0, "conductor unlock_tier drift guard");
}
