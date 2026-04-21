//! Group L — RON asset drift guard (Behaviors L1–L2).
//!
//! Pins that `assets/protocols/afterimage.protocol.ron` parses into a
//! `ProtocolDefinition` with `ProtocolKind::Afterimage`, authored tuning
//! `phantom_duration: 2.0, phantom_bolt_duration: 3.0`, name `"Afterimage"`,
//! description `"Dash to leave a phantom of yourself; Perfect-bump the
//! phantom to split off an extra bolt."`, and `unlock_tier: 0`. The
//! earlier "piercing bolt" wording was factually wrong — the phantom
//! bolt inherits base damage only, no piercing component is attached.

use crate::protocol::definition::{ProtocolDefinition, ProtocolKind, ProtocolTuning};

// ── L1 — RON parses with exact tuning variant + values ────────────────────

#[test]
fn afterimage_ron_asset_deserializes_to_protocol_definition() {
    let ron_str = include_str!("../../../../../assets/protocols/afterimage.protocol.ron");
    let def: ProtocolDefinition =
        ron::de::from_str(ron_str).expect("afterimage.protocol.ron should parse");

    assert_eq!(
        def.kind(),
        ProtocolKind::Afterimage,
        "parsed definition must report ProtocolKind::Afterimage"
    );

    let ProtocolTuning::Afterimage {
        phantom_duration,
        phantom_bolt_duration,
    } = &def.tuning
    else {
        panic!("expected ProtocolTuning::Afterimage, got {:?}", def.tuning);
    };

    assert!(
        (phantom_duration - 2.0).abs() < f32::EPSILON,
        "phantom_duration expected 2.0 (design-doc source of truth), got {phantom_duration}"
    );
    assert!(
        (phantom_bolt_duration - 3.0).abs() < f32::EPSILON,
        "phantom_bolt_duration expected 3.0 (design-doc source of truth), got {phantom_bolt_duration}"
    );
}

// ── L2 — name / description / unlock_tier pinned exactly ──────────────────

#[test]
fn afterimage_ron_name_description_unlock_tier_pinned_exactly() {
    let ron_str = include_str!("../../../../../assets/protocols/afterimage.protocol.ron");
    let def: ProtocolDefinition =
        ron::de::from_str(ron_str).expect("afterimage.protocol.ron should parse");

    assert_eq!(def.name, "Afterimage", "afterimage name drift guard");
    assert_eq!(
        def.description,
        "Dash to leave a phantom of yourself; Perfect-bump the phantom to split off an extra bolt.",
        "afterimage description drift guard"
    );
    assert_eq!(def.unlock_tier, 0, "afterimage unlock_tier drift guard");
}
