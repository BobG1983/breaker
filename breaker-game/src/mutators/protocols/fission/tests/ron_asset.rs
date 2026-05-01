//! RON asset structural tests — pins that `assets/protocols/fission.protocol.ron`
//! parses into a `ProtocolDefinition` with `ProtocolKind::Fission` and carries
//! structurally valid tuning (finite, positive fields). Specific tuning values
//! live in design-behavior tests; name / description / `unlock_tier` are pinned
//! here as stable identifiers.

use crate::mutators::protocols::definition::{ProtocolDefinition, ProtocolKind, ProtocolTuning};

// ── Behavior 40 — RON parses to Fission variant with structurally valid tuning

#[test]
fn fission_ron_asset_deserializes_to_protocol_definition() {
    let ron_str = include_str!("../../../../../assets/protocols/fission.protocol.ron");
    let def: ProtocolDefinition =
        ron::de::from_str(ron_str).expect("fission.protocol.ron should parse");

    assert_eq!(
        def.kind(),
        ProtocolKind::Fission,
        "parsed definition must report ProtocolKind::Fission"
    );

    let ProtocolTuning::Fission {
        kills_per_split, ..
    } = def.tuning.clone()
    else {
        panic!("expected ProtocolTuning::Fission, got {:?}", def.tuning);
    };

    assert!(kills_per_split > 0, "kills_per_split must be > 0");
}

// ── Behavior 41 — divergence_angle_rad is finite and positive ───────────────

#[test]
fn fission_ron_divergence_angle_rad_is_finite_and_positive() {
    let ron_str = include_str!("../../../../../assets/protocols/fission.protocol.ron");
    let def: ProtocolDefinition =
        ron::de::from_str(ron_str).expect("fission.protocol.ron should parse");

    let ProtocolTuning::Fission {
        divergence_angle_rad,
        ..
    } = def.tuning.clone()
    else {
        panic!("expected ProtocolTuning::Fission, got {:?}", def.tuning);
    };

    assert!(
        divergence_angle_rad.is_finite() && divergence_angle_rad > 0.0,
        "divergence_angle_rad must be finite and positive, got {divergence_angle_rad}"
    );
}

// ── Behavior 42 — name / description / unlock_tier pinned ───────────────────

#[test]
fn fission_ron_name_description_unlock_tier_pinned_exactly() {
    let ron_str = include_str!("../../../../../assets/protocols/fission.protocol.ron");
    let def: ProtocolDefinition =
        ron::de::from_str(ron_str).expect("fission.protocol.ron should parse");

    assert_eq!(def.name, "Fission", "fission name drift guard");
    assert_eq!(
        def.description, "Every Nth cell kill splits the active bolt into two.",
        "fission description drift guard"
    );
    assert_eq!(def.unlock_tier, 0, "fission unlock_tier drift guard");
}
